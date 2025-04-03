use std::mem;
use std::sync::Arc;

use http::header;
use http::HeaderMap;
use http::HeaderValue;
use itertools::Itertools;
use p256::ecdsa::signature;
use tracing::info;
use tracing::instrument;
use url::Url;

use crypto::x509::BorrowingCertificateExtension;
use crypto::x509::CertificateError;
use error_category::sentry_capture_error;
use error_category::ErrorCategory;
use jwt::error::JwtError;
use mdoc::utils::cose::CoseError;
use mdoc::utils::issuer_auth::IssuerRegistration;
use openid4vc::credential::MdocCopies;
use openid4vc::credential_payload::CredentialPayloadError;
use openid4vc::issuance_session::HttpIssuanceSession;
use openid4vc::issuance_session::IssuanceSession;
use openid4vc::issuance_session::IssuanceSessionError;
use openid4vc::token::CredentialPreview;
use openid4vc::token::CredentialPreviewError;
use platform_support::attested_key::AttestedKeyHolder;
use sd_jwt_vc_metadata::TypeMetadataError;
use wallet_common::http::TlsPinningConfig;
use wallet_common::reqwest::default_reqwest_client_builder;
use wallet_common::update_policy::VersionState;
use wallet_common::urls;
use wallet_common::vec_at_least::VecNonEmpty;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::account_provider::AccountProviderClient;
use crate::attestation::Attestation;
use crate::attestation::AttestationError;
use crate::attestation::AttestationIdentity;
use crate::config::UNIVERSAL_LINK_BASE_URL;
use crate::errors::ChangePinError;
use crate::errors::UpdatePolicyError;
use crate::instruction::InstructionError;
use crate::instruction::RemoteEcdsaKeyError;
use crate::instruction::RemoteEcdsaKeyFactory;
use crate::issuance::DigidSession;
use crate::issuance::DigidSessionError;
use crate::issuance::HttpDigidSession;
use crate::issuance::PID_DOCTYPE;
use crate::repository::Repository;
use crate::repository::UpdateableRepository;
use crate::storage::Storage;
use crate::storage::StorageError;
use crate::storage::WalletEvent;
use crate::wallet::attestations::AttestationsError;
use crate::wte::WteIssuanceClient;

use super::Wallet;

pub(super) enum PidIssuanceSession<DS = HttpDigidSession, IS = HttpIssuanceSession> {
    Digid(DS),
    Openid4vci(IS),
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum PidIssuanceError {
    #[category(expected)]
    #[error("app version is blocked")]
    VersionBlocked,
    #[error("wallet is not registered")]
    #[category(expected)]
    NotRegistered,
    #[error("wallet is locked")]
    #[category(expected)]
    Locked,
    #[error("issuance session is not in the correct state")]
    #[category(expected)]
    SessionState,
    #[error("PID already present")]
    #[category(expected)]
    PidAlreadyPresent,
    #[error("could not start DigiD session: {0}")]
    DigidSessionStart(#[source] DigidSessionError),
    #[error("could not finish DigiD session: {0}")]
    DigidSessionFinish(#[source] DigidSessionError),
    #[error("could not retrieve PID from issuer: {0}")]
    PidIssuer(#[from] IssuanceSessionError),
    #[error("error sending instruction to Wallet Provider: {0}")]
    Instruction(#[from] InstructionError),
    #[error("invalid signature received from Wallet Provider: {0}")]
    #[category(critical)]
    Signature(#[from] signature::Error),
    #[error("no signature received from Wallet Provider")]
    #[category(critical)]
    MissingSignature,
    #[error("could not insert mdocs in database: {0}")]
    MdocStorage(#[source] StorageError),
    #[error("could not store event in history database: {0}")]
    EventStorage(#[source] StorageError),
    #[error("key '{0}' not found in Wallet Provider")]
    #[category(pd)]
    KeyNotFound(String),
    #[error("invalid issuer certificate: {0}")]
    InvalidIssuerCertificate(#[source] CoseError),
    #[error("issuer not authenticated")]
    #[category(critical)]
    MissingIssuerRegistration,
    #[error("could not read attestations from storage: {0}")]
    Attestations(#[source] AttestationsError),
    #[error("failed to read issuer registration from issuer certificate: {0}")]
    AttestationPreview(#[from] CredentialPreviewError),
    #[error("error finalizing pin change: {0}")]
    ChangePin(#[from] ChangePinError),
    #[error("JWT credential error: {0}")]
    JwtCredential(#[from] JwtError),
    #[error("error fetching update policy: {0}")]
    UpdatePolicy(#[from] UpdatePolicyError),
    #[error("type metadata verification failed: {0}")]
    #[category(critical)]
    TypeMetadataVerification(#[from] TypeMetadataError),
    #[error("error converting mdoc to credential payload: {0}")]
    CredentialPayload(#[from] CredentialPayloadError),
    #[error("error converting credential payload to attestation: {0}")]
    #[category(critical)]
    Attestation(#[from] AttestationError),
    #[error("certificate error: {0}")]
    Certificate(#[from] CertificateError),
}

impl<CR, UR, S, AKH, APC, DS, IS, MDS, WIC> Wallet<CR, UR, S, AKH, APC, DS, IS, MDS, WIC>
where
    CR: Repository<Arc<WalletConfiguration>>,
    UR: Repository<VersionState>,
    AKH: AttestedKeyHolder,
    DS: DigidSession,
    IS: IssuanceSession,
    S: Storage,
    APC: AccountProviderClient,
{
    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn create_pid_issuance_auth_url(&mut self) -> Result<Url, PidIssuanceError> {
        info!("Generating DigiD auth URL, starting OpenID connect discovery");

        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(PidIssuanceError::VersionBlocked);
        }

        info!("Checking if registered");
        if !self.registration.is_registered() {
            return Err(PidIssuanceError::NotRegistered);
        }

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(PidIssuanceError::Locked);
        }

        info!("Checking if there is an active issuance session");
        if self.issuance_session.is_some() {
            return Err(PidIssuanceError::SessionState);
        }

        info!("Checking if a pid is already present");
        let has_pid = self
            .storage
            .write()
            .await
            .has_any_mdocs_with_doctype(PID_DOCTYPE)
            .await
            .map_err(PidIssuanceError::MdocStorage)?;
        if has_pid {
            return Err(PidIssuanceError::PidAlreadyPresent);
        }

        let pid_issuance_config = &self.config_repository.get().pid_issuance;
        let (session, auth_url) = DS::start(
            pid_issuance_config.digid.clone(),
            &pid_issuance_config.digid_http_config,
            urls::issuance_base_uri(&UNIVERSAL_LINK_BASE_URL).as_ref().to_owned(),
        )
        .await
        .map_err(PidIssuanceError::DigidSessionStart)?;

        info!("DigiD auth URL generated");
        self.issuance_session.replace(PidIssuanceSession::Digid(session));

        Ok(auth_url)
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub fn has_active_pid_issuance_session(&self) -> Result<bool, PidIssuanceError> {
        info!("Checking for active PID issuance session");

        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(PidIssuanceError::VersionBlocked);
        }

        info!("Checking if registered");
        if !self.registration.is_registered() {
            return Err(PidIssuanceError::NotRegistered);
        }

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(PidIssuanceError::Locked);
        }

        let has_active_session = self.issuance_session.is_some();

        Ok(has_active_session)
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn cancel_pid_issuance(&mut self) -> Result<(), PidIssuanceError> {
        info!("PID issuance cancelled / rejected");

        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(PidIssuanceError::VersionBlocked);
        }

        info!("Checking if registered");
        if !self.registration.is_registered() {
            return Err(PidIssuanceError::NotRegistered);
        }

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(PidIssuanceError::Locked);
        }

        info!("Checking if there is an active issuance session");
        let issuance_session = self.issuance_session.take().ok_or(PidIssuanceError::SessionState)?;

        if let PidIssuanceSession::Openid4vci(pid_issuer) = issuance_session {
            info!("Rejecting PID");
            pid_issuer.reject_issuance().await?;
        }

        Ok(())
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn continue_pid_issuance(&mut self, redirect_uri: Url) -> Result<Vec<Attestation>, PidIssuanceError> {
        info!("Received DigiD redirect URI, processing URI and retrieving access token");

        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(PidIssuanceError::VersionBlocked);
        }

        info!("Checking if registered");
        if !self.registration.is_registered() {
            return Err(PidIssuanceError::NotRegistered);
        }

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(PidIssuanceError::Locked);
        }

        info!("Checking if there is an active DigiD issuance session");
        if !matches!(self.issuance_session, Some(PidIssuanceSession::Digid(_))) {
            return Err(PidIssuanceError::SessionState);
        }

        // Take ownership of the active session, now that we know that it exists.
        let session = match self.issuance_session.take().unwrap() {
            PidIssuanceSession::Digid(session) => session,
            PidIssuanceSession::Openid4vci(_) => panic!(),
        };

        let token_request = session
            .into_token_request(redirect_uri)
            .await
            .map_err(PidIssuanceError::DigidSessionFinish)?;

        let pid_issuer_http_client = default_reqwest_client_builder()
            .default_headers(HeaderMap::from_iter([(
                header::ACCEPT,
                HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
            )]))
            .build()
            .expect("Could not build reqwest HTTP client");
        let config = self.config_repository.get();

        let (pid_issuer, attestation_previews) = IS::start_issuance(
            pid_issuer_http_client.into(),
            config.pid_issuance.pid_issuer_url.clone(),
            token_request,
            &config.mdoc_trust_anchors(),
        )
        .await?;

        info!("PID received successfully from issuer, returning preview documents");
        let attestations = attestation_previews
            .into_iter()
            .flat_map(|(formats, metadata)| formats.into_iter().zip(metadata))
            .map(|(preview, metadata)| {
                let issuer_registration = preview.issuer_registration()?;
                match preview {
                    CredentialPreview::MsoMdoc { unsigned_mdoc, .. } => {
                        let attestation = Attestation::create_for_issuance(
                            AttestationIdentity::Ephemeral,
                            metadata,
                            issuer_registration.organization,
                            unsigned_mdoc.attributes.into(),
                        )?;

                        Ok(attestation)
                    }
                }
            })
            .collect::<Result<Vec<_>, PidIssuanceError>>()?;

        self.issuance_session
            .replace(PidIssuanceSession::Openid4vci(pid_issuer));

        Ok(attestations)
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn accept_pid_issuance(&mut self, pin: String) -> Result<(), PidIssuanceError>
    where
        UR: UpdateableRepository<VersionState, TlsPinningConfig, Error = UpdatePolicyError>,
        S: Storage,
        APC: AccountProviderClient,
        WIC: WteIssuanceClient + Default,
    {
        info!("Accepting PID issuance");

        let config = &self.config_repository.get().update_policy_server;

        info!("Fetching update policy");
        self.update_policy_repository.fetch(&config.http_config).await?;

        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(PidIssuanceError::VersionBlocked);
        }

        info!("Checking if registered");
        let (attested_key, registration_data) = self
            .registration
            .as_key_and_registration_data()
            .ok_or_else(|| PidIssuanceError::NotRegistered)?;

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(PidIssuanceError::Locked);
        }

        info!("Checking if there is an active PID issuance session");
        let pid_issuer = match self.issuance_session.as_ref().ok_or(PidIssuanceError::SessionState)? {
            PidIssuanceSession::Digid(_) => Err(PidIssuanceError::SessionState)?,
            PidIssuanceSession::Openid4vci(pid_issuer) => pid_issuer,
        };

        let config = self.config_repository.get();

        let instruction_result_public_key = config.account_server.instruction_result_public_key.as_inner().into();

        let remote_instruction = self
            .new_instruction_client(
                pin,
                Arc::clone(attested_key),
                registration_data.clone(),
                config.account_server.http_config.clone(),
                instruction_result_public_key,
            )
            .await?;

        let wte = self
            .wte_issuance_client
            .obtain_wte(
                config.account_server.wte_public_key.as_inner(),
                remote_instruction.clone(),
            )
            .await?;

        let remote_key_factory = RemoteEcdsaKeyFactory::new(remote_instruction);

        info!("Accepting PID by signing mdoc using Wallet Provider");

        let issuance_result = pid_issuer
            .accept_issuance(
                &config.mdoc_trust_anchors(),
                &remote_key_factory,
                Some(wte),
                config.pid_issuance.pid_issuer_url.clone(),
            )
            .await
            .map_err(|error| {
                match error {
                    // We knowingly call unwrap() on the downcast to `RemoteEcdsaKeyError` here because we know
                    // that it is the error type of the `RemoteEcdsaKeyFactory` we provide above.
                    IssuanceSessionError::PrivateKeyGeneration(error)
                    | IssuanceSessionError::Jwt(JwtError::Signing(error)) => {
                        match *error.downcast::<RemoteEcdsaKeyError>().unwrap() {
                            RemoteEcdsaKeyError::Instruction(error) => PidIssuanceError::Instruction(error),
                            RemoteEcdsaKeyError::Signature(error) => PidIssuanceError::Signature(error),
                            RemoteEcdsaKeyError::KeyNotFound(identifier) => PidIssuanceError::KeyNotFound(identifier),
                            RemoteEcdsaKeyError::MissingSignature => PidIssuanceError::MissingSignature,
                        }
                    }
                    _ => PidIssuanceError::PidIssuer(error),
                }
            });

        // Make sure there are no remaining references to the `AttestedKey` value.
        mem::drop(remote_key_factory);

        // If the Wallet Provider returns either a PIN timeout or a permanent block,
        // wipe the contents of the wallet and return it to its initial state.
        if matches!(
            issuance_result,
            Err(PidIssuanceError::Instruction(
                InstructionError::Timeout { .. } | InstructionError::Blocked
            ))
        ) {
            self.reset_to_initial_state().await;
        }
        let issued_mdocs = issuance_result?
            .into_iter()
            .map(|mdocs| mdocs.try_into())
            .collect::<Result<Vec<_>, _>>()?;

        info!("Isuance succeeded; removing issuance session state");
        self.issuance_session.take();

        // Prepare events before storing mdocs, to avoid cloning mdocs
        let event = {
            // Extract first copy from each issued mdoc
            let mdocs: VecNonEmpty<_> = issued_mdocs
                .iter()
                .map(|mdoc: &MdocCopies| mdoc.first().clone())
                .collect_vec()
                .try_into()
                .expect("should have received at least one issued mdoc");

            // Validate all issuer_certificates
            for mdoc in mdocs.as_ref() {
                let certificate = mdoc
                    .issuer_certificate()
                    .map_err(PidIssuanceError::InvalidIssuerCertificate)?;

                // Verify that the certificate contains IssuerRegistration
                if matches!(IssuerRegistration::from_certificate(&certificate), Err(_) | Ok(None)) {
                    return Err(PidIssuanceError::MissingIssuerRegistration);
                }
            }
            WalletEvent::new_issuance(mdocs)
        };

        info!("PID accepted, storing mdoc in database");
        self.storage
            .write()
            .await
            .insert_mdocs(issued_mdocs)
            .await
            .map_err(PidIssuanceError::MdocStorage)?;

        self.store_history_event(event)
            .await
            .map_err(PidIssuanceError::EventStorage)?;

        self.emit_attestations().await.map_err(PidIssuanceError::Attestations)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use mockall::predicate::*;
    use openid4vc::credential_formats::CredentialFormats;
    use rstest::rstest;
    use serial_test::serial;
    use url::Url;

    use mdoc::holder::Mdoc;
    use openid4vc::issuance_session::IssuedCredential;
    use openid4vc::mock::MockIssuanceSession;
    use openid4vc::oidc::OidcError;
    use openid4vc::token::CredentialPreview;
    use openid4vc::token::TokenRequest;
    use openid4vc::token::TokenRequestGrantType;
    use sd_jwt_vc_metadata::TypeMetadataDocuments;
    use wallet_common::http::TlsPinningConfig;
    use wallet_common::vec_at_least::VecNonEmpty;

    use crate::issuance;
    use crate::issuance::MockDigidSession;
    use crate::storage::StorageState;

    use super::super::test;
    use super::super::test::WalletDeviceVendor;
    use super::super::test::WalletWithMocks;
    use super::super::test::ISSUER_KEY;
    use super::*;

    fn mock_issuance_session(mdoc: Mdoc) -> MockIssuanceSession {
        let mut client = MockIssuanceSession::new();
        client.expect_accept().return_once(|| {
            Ok(vec![vec![IssuedCredential::MsoMdoc(Box::new(mdoc))]
                .try_into()
                .unwrap()])
        });
        client
    }

    #[tokio::test]
    #[serial(MockDigidSession)]
    async fn test_create_pid_issuance_auth_url() {
        const AUTH_URL: &str = "http://example.com/auth";

        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        assert!(wallet.issuance_session.is_none());

        // Set up a mock DigiD session.
        let session_start_context = MockDigidSession::start_context();
        session_start_context.expect().returning(|_, _: &TlsPinningConfig, _| {
            let client = MockDigidSession::default();
            Ok((client, Url::parse(AUTH_URL).unwrap()))
        });

        // Have the `Wallet` generate a DigiD authentication URL and test it.
        let auth_url = wallet
            .create_pid_issuance_auth_url()
            .await
            .expect("Could not generate PID issuance auth URL");

        assert_eq!(auth_url.as_str(), AUTH_URL);
        assert!(wallet.issuance_session.is_some());
    }

    #[tokio::test]
    async fn test_create_pid_issuance_auth_url_error_locked() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        wallet.lock();

        // Creating a DigiD authentication URL on
        // a locked wallet should result in an error.
        let error = wallet
            .create_pid_issuance_auth_url()
            .await
            .expect_err("PID issuance auth URL generation should have resulted in error");

        assert_matches!(error, PidIssuanceError::Locked);
    }

    #[tokio::test]
    async fn test_create_pid_issuance_auth_url_error_unregistered() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithMocks::new_unregistered(WalletDeviceVendor::Apple);

        // Creating a DigiD authentication URL on an
        // unregistered wallet should result in an error.
        let error = wallet
            .create_pid_issuance_auth_url()
            .await
            .expect_err("PID issuance auth URL generation should have resulted in error");

        assert_matches!(error, PidIssuanceError::NotRegistered);
    }

    #[tokio::test]
    async fn test_create_pid_issuance_auth_url_error_session_state_digid() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Set up a mock DigiD session.
        wallet.issuance_session = Some(PidIssuanceSession::Digid(MockDigidSession::default()));

        // Creating a DigiD authentication URL on a `Wallet` that
        // has an active DigiD session should return an error.
        let error = wallet
            .create_pid_issuance_auth_url()
            .await
            .expect_err("PID issuance auth URL generation should have resulted in error");

        assert_matches!(error, PidIssuanceError::SessionState);
    }

    #[tokio::test]
    async fn test_create_pid_issuance_auth_url_error_session_state_pid_issuer() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Setup a mock OpenID4VCI session.
        wallet.issuance_session = Some(PidIssuanceSession::Openid4vci(MockIssuanceSession::default()));

        // Creating a DigiD authentication URL on a `Wallet` that has
        // an active OpenID4VCI session should return an error.
        let error = wallet
            .create_pid_issuance_auth_url()
            .await
            .expect_err("PID issuance auth URL generation should have resulted in error");

        assert_matches!(error, PidIssuanceError::SessionState);
    }

    #[tokio::test]
    #[serial(MockDigidSession)]
    async fn test_create_pid_issuance_auth_url_error_digid_session_start() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Make DigiD session starting return an error.
        let session_start_context = MockDigidSession::start_context();
        session_start_context
            .expect()
            .return_once(|_, _: &TlsPinningConfig, _| Err(OidcError::NoAuthCode.into()));

        // The error should be forwarded when attempting to create a DigiD authentication URL.
        let error = wallet
            .create_pid_issuance_auth_url()
            .await
            .expect_err("PID issuance auth URL generation should have resulted in error");

        assert_matches!(error, PidIssuanceError::DigidSessionStart(_));
    }

    #[tokio::test]
    async fn test_cancel_pid_issuance_digid() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Set up a mock DigiD session.
        wallet.issuance_session = Some(PidIssuanceSession::Digid(MockDigidSession::default()));

        assert!(wallet.issuance_session.is_some());

        // Cancelling PID issuance should clear this session.
        wallet
            .cancel_pid_issuance()
            .await
            .expect("Could not cancel PID issuance");

        assert!(wallet.issuance_session.is_none());
    }

    #[tokio::test]
    async fn test_cancel_pid_issuance_pid() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Set up the `PidIssuerClient`
        let pid_issuer = {
            let mut client = MockIssuanceSession::new();
            client.expect_reject().return_once(|| Ok(()));
            client
        };
        wallet.issuance_session = Some(PidIssuanceSession::Openid4vci(pid_issuer));

        // Cancelling PID issuance should not fail.
        wallet
            .cancel_pid_issuance()
            .await
            .expect("Could not cancel PID issuance");

        assert!(wallet.issuance_session.is_none());
    }

    #[tokio::test]
    async fn test_cancel_pid_issuance_error_locked() {
        // Prepare a registered and locked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        wallet.lock();

        // Cancelling PID issuance on a locked wallet should result in an error.
        let error = wallet
            .cancel_pid_issuance()
            .await
            .expect_err("Cancelling PID issuance should have resulted in an error");

        assert_matches!(error, PidIssuanceError::Locked);
    }

    #[tokio::test]
    async fn test_cancel_pid_issuance_error_unregistered() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithMocks::new_unregistered(WalletDeviceVendor::Apple);

        // Cancelling PID issuance on an unregistered wallet should result in an error.
        let error = wallet
            .cancel_pid_issuance()
            .await
            .expect_err("Cancelling PID issuance should have resulted in an error");

        assert_matches!(error, PidIssuanceError::NotRegistered);
    }

    #[tokio::test]
    async fn test_cancel_pid_issuance_error_session_state() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Cancelling PID issuance on a wallet with no
        // active DigiD session should result in an error.
        let error = wallet
            .cancel_pid_issuance()
            .await
            .expect_err("Cancelling PID issuance should have resulted in an error");

        assert_matches!(error, PidIssuanceError::SessionState);
    }

    const REDIRECT_URI: &str = "redirect://here";

    #[tokio::test]
    #[serial(MockIssuanceSession)]
    async fn test_continue_pid_issuance() {
        let mut wallet = setup_wallet_with_digid_session();

        let (unsigned_mdoc, metadata) = issuance::mock::create_example_unsigned_mdoc();
        let (_, _, metadata_documents) = TypeMetadataDocuments::from_single_example(metadata);
        let (normalized_metadata, _) = metadata_documents
            .clone()
            .into_normalized(&unsigned_mdoc.doc_type)
            .unwrap();
        let credential_formats = CredentialFormats::try_new(
            VecNonEmpty::try_from(vec![CredentialPreview::MsoMdoc {
                unsigned_mdoc,
                issuer_certificate: ISSUER_KEY.issuance_key.certificate().clone(),
                type_metadata: metadata_documents.clone(),
            }])
            .unwrap(),
        )
        .unwrap();
        // Set up the `MockIssuanceSession` to return one `AttestationPreview`.
        let start_context = MockIssuanceSession::start_context();
        start_context.expect().return_once(|| {
            Ok((
                MockIssuanceSession::new(),
                vec![(credential_formats, vec![normalized_metadata])],
            ))
        });

        // Continuing PID issuance should result in one preview `Document`.
        let attestations = wallet
            .continue_pid_issuance(Url::parse(REDIRECT_URI).unwrap())
            .await
            .expect("Could not continue PID issuance");

        assert_eq!(attestations.len(), 1);
        assert_matches!(attestations[0].identity, AttestationIdentity::Ephemeral);
    }

    #[tokio::test]
    async fn test_continue_pid_issuance_error_locked() {
        // Prepare a registered and locked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        wallet.lock();

        // Continuing PID issuance on a locked wallet should result in an error.
        let error = wallet
            .continue_pid_issuance(Url::parse(REDIRECT_URI).unwrap())
            .await
            .expect_err("Continuing PID issuance should have resulted in error");

        assert_matches!(error, PidIssuanceError::Locked);
    }

    #[tokio::test]
    async fn test_continue_pid_issuance_error_unregistered() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithMocks::new_unregistered(WalletDeviceVendor::Apple);

        // Continuing PID issuance on an unregistered wallet should result in an error.
        let error = wallet
            .continue_pid_issuance(Url::parse(REDIRECT_URI).unwrap())
            .await
            .expect_err("Continuing PID issuance should have resulted in error");

        assert_matches!(error, PidIssuanceError::NotRegistered);
    }

    #[tokio::test]
    async fn test_continue_pid_issuance_error_session_state() {
        // Prepare a registered wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Continuing PID issuance on a wallet with no active `DigidSession` should result in an error.
        let error = wallet
            .continue_pid_issuance(Url::parse(REDIRECT_URI).unwrap())
            .await
            .expect_err("Continuing PID issuance should have resulted in error");

        assert_matches!(error, PidIssuanceError::SessionState);
    }

    fn setup_wallet_with_digid_session() -> WalletWithMocks {
        // Prepare a registered wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Set up a mock DigiD session that returns a token request.
        let digid_session = {
            let mut session = MockDigidSession::default();

            session.expect_into_token_request().return_once(|_uri| {
                Ok(TokenRequest {
                    grant_type: TokenRequestGrantType::PreAuthorizedCode {
                        pre_authorized_code: "123".to_string().into(),
                    },
                    code_verifier: None,
                    client_id: None,
                    redirect_uri: None,
                })
            });

            session
        };
        wallet.issuance_session = Some(PidIssuanceSession::Digid(digid_session));

        wallet
    }

    #[tokio::test]
    #[serial(MockIssuanceSession)]
    async fn test_continue_pid_issuance_error_pid_issuer() {
        let mut wallet = setup_wallet_with_digid_session();

        // Set up the `MockIssuanceSession` to return an error.
        let start_context = MockIssuanceSession::start_context();
        start_context
            .expect()
            .return_once(|| Err(IssuanceSessionError::MissingNonce));

        // Continuing PID issuance on a wallet should forward this error.
        let error = wallet
            .continue_pid_issuance(Url::parse(REDIRECT_URI).unwrap())
            .await
            .expect_err("Continuing PID issuance should have resulted in error");

        assert_matches!(error, PidIssuanceError::PidIssuer(_));
    }

    #[tokio::test]
    async fn test_cancel_pid_issuance_error_pid_issuer() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Set up a mock OpenID4VCI session that expects to be rejected, which returns an error.
        let pid_issuer = {
            let mut client = MockIssuanceSession::new();
            client
                .expect_reject()
                .return_once(|| Err(IssuanceSessionError::MissingNonce));
            client
        };
        wallet.issuance_session = Some(PidIssuanceSession::Openid4vci(pid_issuer));

        // Canceling PID issuance on a wallet should forward this error.
        let error = wallet
            .cancel_pid_issuance()
            .await
            .expect_err("Rejecting PID issuance should have resulted in an error");

        assert_matches!(error, PidIssuanceError::PidIssuer(_));
    }

    const PIN: &str = "051097";

    #[tokio::test]
    async fn test_accept_pid_issuance() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Register mock document_callback
        let attestations = test::setup_mock_attestations_callback(&mut wallet).await.unwrap();

        // Register mock recent_history_callback
        let events = test::setup_mock_recent_history_callback(&mut wallet).await.unwrap();

        // Create a mock OpenID4VCI session that accepts the PID with a single
        // instance of `MdocCopies`, which contains a single valid `Mdoc`.
        let mdoc = test::create_example_pid_mdoc();
        let pid_issuer = mock_issuance_session(mdoc);
        wallet.issuance_session = Some(PidIssuanceSession::Openid4vci(pid_issuer));

        // Accept the PID issuance with the PIN.
        wallet
            .accept_pid_issuance(PIN.to_owned())
            .await
            .expect("Could not accept PID issuance");

        {
            // Test which `Attestation` instances we have received through the callback.
            let attestations = attestations.lock();

            // The first entry should be empty, because there are no mdocs in the database.
            assert_eq!(attestations.len(), 2);
            assert!(attestations[0].is_empty());

            // The second entry should contain a single attestation with the PID.
            assert_eq!(attestations[1].len(), 1);
            let attestation = &attestations[1][0];
            assert_matches!(attestation.identity, AttestationIdentity::Fixed { id: _ });
            assert_eq!(attestation.attestation_type, "com.example.pid");

            // Test that one successful issuance event is logged
            let events = events.lock();
            assert_eq!(events.len(), 2);
            assert!(events[0].is_empty());
            assert_eq!(events[1].len(), 1);
            assert_matches!(&events[1][0], WalletEvent::Issuance { .. });

            assert!(wallet.has_registration());
            assert!(!wallet.is_locked());
        }

        // Starting another PID issuance should fail
        const AUTH_URL: &str = "http://example.com/auth";
        // Set up a mock DigiD session.
        let session_start_context = MockDigidSession::start_context();
        session_start_context.expect().returning(|_, _: &TlsPinningConfig, _| {
            let client = MockDigidSession::default();
            Ok((client, Url::parse(AUTH_URL).unwrap()))
        });

        let err = wallet
            .create_pid_issuance_auth_url()
            .await
            .expect_err("creating new PID issuance auth URL when there already is a PID should fail");
        assert_matches!(err, PidIssuanceError::PidAlreadyPresent);
    }

    #[tokio::test]
    async fn test_accept_pid_issuance_missing_issuer_registration() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Create a mock OpenID4VCI session that accepts the PID with a single instance of `MdocCopies`, which contains
        // a single valid `Mdoc`, but signed with a Certificate that is missing IssuerRegistration
        let mdoc = test::create_example_pid_mdoc_unauthenticated();
        let pid_issuer = mock_issuance_session(mdoc);
        wallet.issuance_session = Some(PidIssuanceSession::Openid4vci(pid_issuer));

        // Accept the PID issuance with the PIN.
        let error = wallet
            .accept_pid_issuance(PIN.to_owned())
            .await
            .expect_err("Accepting PID issuance should have resulted in an error");

        assert_matches!(error, PidIssuanceError::MissingIssuerRegistration);

        // No issuance event is logged
        let events = wallet.storage.read().await.fetch_wallet_events().await.unwrap();
        assert!(events.is_empty());

        assert!(wallet.has_registration());
        assert!(!wallet.is_locked());
    }

    #[tokio::test]
    async fn test_accept_pid_issuance_unregistered() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithMocks::new_unregistered(WalletDeviceVendor::Apple);

        // Accepting PID issuance on an unregistered wallet should result in an error.
        let error = wallet
            .accept_pid_issuance(PIN.to_owned())
            .await
            .expect_err("Accepting PID issuance should have resulted in an error");

        assert_matches!(error, PidIssuanceError::NotRegistered);
    }

    #[tokio::test]
    async fn test_accept_pid_issuance_locked() {
        // Prepare a registered and locked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        wallet.lock();

        // Accepting PID issuance on a locked wallet should result in an error.
        let error = wallet
            .accept_pid_issuance(PIN.to_owned())
            .await
            .expect_err("Accepting PID issuance should have resulted in an error");

        assert_matches!(error, PidIssuanceError::Locked);

        assert!(wallet.has_registration());
        assert!(wallet.is_locked());
    }

    #[tokio::test]
    async fn test_accept_pid_issuance_session_state() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Accepting PID issuance on a `Wallet` with a `PidIssuerClient`
        // that has no session should result in an error.
        let error = wallet
            .accept_pid_issuance(PIN.to_owned())
            .await
            .expect_err("Accepting PID issuance should have resulted in an error");

        assert_matches!(error, PidIssuanceError::SessionState);

        assert!(wallet.has_registration());
        assert!(!wallet.is_locked());
    }

    async fn test_accept_pid_issuance_error_remote_key(
        key_error: RemoteEcdsaKeyError,
    ) -> (WalletWithMocks, PidIssuanceError) {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Have the mock OpenID4VCI session return a particular `RemoteEcdsaKeyError` upon accepting.
        let pid_issuer = {
            let mut client = MockIssuanceSession::new();
            client
                .expect_accept()
                .return_once(|| Err(IssuanceSessionError::Jwt(JwtError::Signing(Box::new(key_error)))));
            client
        };
        wallet.issuance_session = Some(PidIssuanceSession::Openid4vci(pid_issuer));

        // Accepting PID issuance should result in an error.
        let error = wallet
            .accept_pid_issuance(PIN.to_owned())
            .await
            .expect_err("Accepting PID issuance should have resulted in an error");

        (wallet, error)
    }

    #[rstest]
    #[case(InstructionError::IncorrectPin { attempts_left_in_round: 1, is_final_round: false }, false)]
    #[case(InstructionError::Timeout { timeout_millis: 10_000 }, true)]
    #[case(InstructionError::Blocked, true)]
    #[case(InstructionError::InstructionValidation, false)]
    #[tokio::test]
    async fn test_accept_pid_issuance_error_instruction(
        #[case] instruction_error: InstructionError,
        #[case] expect_reset: bool,
    ) {
        let (wallet, error) =
            test_accept_pid_issuance_error_remote_key(RemoteEcdsaKeyError::from(instruction_error)).await;

        // Test that this error is converted to the appropriate variant of `PidIssuanceError`.
        assert_matches!(error, PidIssuanceError::Instruction(_));

        // Test the state of the Wallet, based on if we expect a reset for this InstructionError.
        if expect_reset {
            assert!(!wallet.has_registration());
            assert!(wallet.is_locked());
            assert_matches!(
                wallet.storage.read().await.state().await.unwrap(),
                StorageState::Uninitialized
            );
        } else {
            assert!(wallet.has_registration());
            assert!(!wallet.is_locked());
            assert_matches!(wallet.storage.read().await.state().await.unwrap(), StorageState::Opened);
        }
    }

    #[tokio::test]
    async fn test_accept_pid_issuance_error_signature() {
        let (wallet, error) =
            test_accept_pid_issuance_error_remote_key(RemoteEcdsaKeyError::from(signature::Error::default())).await;

        // Test that this error is converted to the appropriate variant of `PidIssuanceError`.
        assert_matches!(error, PidIssuanceError::Signature(_));

        assert!(wallet.has_registration());
        assert!(!wallet.is_locked());
    }

    #[tokio::test]
    async fn test_accept_pid_issuance_error_key_not_found() {
        let (wallet, error) =
            test_accept_pid_issuance_error_remote_key(RemoteEcdsaKeyError::KeyNotFound("not found".to_string())).await;

        // Test that this error is converted to the appropriate variant of `PidIssuanceError`.
        assert_matches!(error, PidIssuanceError::KeyNotFound(_));

        assert!(wallet.has_registration());
        assert!(!wallet.is_locked());
    }

    #[tokio::test]
    async fn test_accept_pid_issuance_error_pid_issuer() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Have the mock OpenID4VCI session return an error upon accepting.
        let pid_issuer = {
            let mut client = MockIssuanceSession::new();
            client
                .expect_accept()
                .return_once(|| Err(IssuanceSessionError::MissingNonce));
            client
        };
        wallet.issuance_session = Some(PidIssuanceSession::Openid4vci(pid_issuer));

        // Accepting PID issuance should result in an error.
        let error = wallet
            .accept_pid_issuance(PIN.to_owned())
            .await
            .expect_err("Accepting PID issuance should have resulted in an error");

        assert_matches!(error, PidIssuanceError::PidIssuer(_));

        assert!(wallet.has_registration());
        assert!(!wallet.is_locked());
    }

    #[tokio::test]
    async fn test_accept_pid_issuance_error_storage() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Have the mock OpenID4VCI session report some mdocs upon accepting.
        let mdoc = test::create_example_pid_mdoc();
        let pid_issuer = mock_issuance_session(mdoc);
        wallet.issuance_session = Some(PidIssuanceSession::Openid4vci(pid_issuer));

        // Have the mdoc storage return an error on query.
        wallet.storage.write().await.has_query_error = true;

        // Accepting PID issuance should result in an error.
        let error = wallet
            .accept_pid_issuance(PIN.to_owned())
            .await
            .expect_err("Accepting PID issuance should have resulted in an error");

        assert_matches!(error, PidIssuanceError::MdocStorage(_));

        assert!(wallet.has_registration());
        assert!(!wallet.is_locked());
    }
}
