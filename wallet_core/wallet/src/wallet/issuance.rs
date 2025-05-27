use std::sync::Arc;

use derive_more::Constructor;
use http::header;
use http::HeaderMap;
use http::HeaderValue;
use itertools::Itertools;
use p256::ecdsa::signature;
use rustls_pki_types::TrustAnchor;
use tracing::info;
use tracing::instrument;
use url::Url;

use attestation_data::auth::issuer_auth::IssuerRegistration;
use attestation_data::auth::Organization;
use crypto::x509::BorrowingCertificateExtension;
use crypto::x509::CertificateError;
use error_category::sentry_capture_error;
use error_category::ErrorCategory;
use http_utils::reqwest::default_reqwest_client_builder;
use http_utils::tls::pinning::TlsPinningConfig;
use http_utils::urls;
use http_utils::urls::BaseUrl;
use jwt::error::JwtError;
use mdoc::utils::cose::CoseError;
use openid4vc::credential::CredentialCopies;
use openid4vc::credential::MdocCopies;
use openid4vc::issuance_session::HttpVcMessageClient;
use openid4vc::issuance_session::IssuanceSession as Openid4vcIssuanceSession;
use openid4vc::issuance_session::IssuanceSessionError;
use openid4vc::token::CredentialPreviewError;
use openid4vc::token::TokenRequest;
use platform_support::attested_key::AttestedKeyHolder;
use update_policy_model::update_policy::VersionState;
use utils::vec_at_least::VecNonEmpty;
use wallet_account::NL_WALLET_CLIENT_ID;
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
use crate::issuance::PID_DOCTYPE;
use crate::repository::Repository;
use crate::repository::UpdateableRepository;
use crate::storage::Storage;
use crate::storage::StorageError;
use crate::storage::WalletEvent;
use crate::wallet::attestations::AttestationsError;
use crate::wallet::Session;
use crate::wte::WteIssuanceClient;

use super::Wallet;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum IssuanceError {
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
    #[error("could not retrieve attestations from issuer: {0}")]
    IssuanceSession(#[from] IssuanceSessionError),
    #[error("could not retrieve attestations from issuer: {error}")]
    IssuerServer {
        organization: Box<Organization>,
        #[defer]
        #[source]
        error: IssuanceSessionError,
    },
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
    #[error("error converting credential payload to attestation: {error}")]
    #[category(critical)]
    Attestation {
        organization: Box<Organization>,
        #[source]
        error: AttestationError,
    },
    #[error("certificate error: {0}")]
    Certificate(#[from] CertificateError),
}

#[derive(Debug, Clone, Constructor)]
pub struct IssuanceSession<IS> {
    pub is_pid: bool,
    pub protocol_state: IS,
}

impl<CR, UR, S, AKH, APC, DS, IS, MDS, WIC> Wallet<CR, UR, S, AKH, APC, DS, IS, MDS, WIC>
where
    CR: Repository<Arc<WalletConfiguration>>,
    UR: Repository<VersionState>,
    AKH: AttestedKeyHolder,
    DS: DigidSession,
    IS: Openid4vcIssuanceSession,
    S: Storage,
    APC: AccountProviderClient,
{
    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn create_pid_issuance_auth_url(&mut self) -> Result<Url, IssuanceError> {
        info!("Generating DigiD auth URL, starting OpenID connect discovery");

        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(IssuanceError::VersionBlocked);
        }

        info!("Checking if registered");
        if !self.registration.is_registered() {
            return Err(IssuanceError::NotRegistered);
        }

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(IssuanceError::Locked);
        }

        info!("Checking if there is an active session");
        if self.session.is_some() {
            return Err(IssuanceError::SessionState);
        }

        info!("Checking if a pid is already present");
        let has_pid = self
            .storage
            .write()
            .await
            .has_any_mdocs_with_doctype(PID_DOCTYPE)
            .await
            .map_err(IssuanceError::MdocStorage)?;
        if has_pid {
            return Err(IssuanceError::PidAlreadyPresent);
        }

        let pid_issuance_config = &self.config_repository.get().pid_issuance;
        let (session, auth_url) = DS::start(
            pid_issuance_config.digid.clone(),
            &pid_issuance_config.digid_http_config,
            urls::issuance_base_uri(&UNIVERSAL_LINK_BASE_URL).as_ref().to_owned(),
        )
        .await
        .map_err(IssuanceError::DigidSessionStart)?;

        info!("DigiD auth URL generated");
        self.session.replace(Session::Digid(session));

        Ok(auth_url)
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub fn has_active_issuance_session(&self) -> Result<bool, IssuanceError> {
        info!("Checking for active issuance session");

        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(IssuanceError::VersionBlocked);
        }

        info!("Checking if registered");
        if !self.registration.is_registered() {
            return Err(IssuanceError::NotRegistered);
        }

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(IssuanceError::Locked);
        }

        let has_active_session = matches!(self.session, Some(Session::Digid(..)) | Some(Session::Issuance(..)));

        Ok(has_active_session)
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn cancel_issuance(&mut self) -> Result<(), IssuanceError> {
        info!("Issuance cancelled / rejected");

        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(IssuanceError::VersionBlocked);
        }

        info!("Checking if registered");
        if !self.registration.is_registered() {
            return Err(IssuanceError::NotRegistered);
        }

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(IssuanceError::Locked);
        }

        info!("Checking if there is an active issuance session");
        if !matches!(self.session, Some(Session::Digid(..)) | Some(Session::Issuance(..))) {
            return Err(IssuanceError::SessionState);
        }

        let session = self.session.take().unwrap();
        if let Session::Issuance(issuance_session) = session {
            let organization = issuance_session
                .protocol_state
                .issuer_registration()
                .organization
                .clone();

            info!("Rejecting issuance");
            issuance_session
                .protocol_state
                .reject_issuance()
                .await
                .map_err(|error| IssuanceError::IssuerServer {
                    organization: Box::new(organization),
                    error,
                })?;
        };

        // In the DigiD stage of PID issuance we don't have to do anything with the DigiD session state,
        // so we don't need to match `session` on that arm.

        Ok(())
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn continue_pid_issuance(&mut self, redirect_uri: Url) -> Result<Vec<Attestation>, IssuanceError> {
        info!("Received redirect URI, processing URI and retrieving access token");

        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(IssuanceError::VersionBlocked);
        }

        info!("Checking if registered");
        if !self.registration.is_registered() {
            return Err(IssuanceError::NotRegistered);
        }

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(IssuanceError::Locked);
        }

        info!("Checking if there is an active DigiD issuance session");
        if !matches!(self.session, Some(Session::Digid(..))) {
            return Err(IssuanceError::SessionState);
        }

        // Take ownership of the active session, now that we know that it exists.
        let Some(Session::Digid(session)) = self.session.take() else {
            panic!()
        };

        let token_request = session
            .into_token_request(redirect_uri)
            .await
            .map_err(IssuanceError::DigidSessionFinish)?;

        let config = self.config_repository.get();

        self.issuance_fetch_previews(
            token_request,
            config.pid_issuance.pid_issuer_url.clone(),
            &config.mdoc_trust_anchors(),
            true,
        )
        .await
    }

    #[instrument(skip_all)]
    pub(super) async fn issuance_fetch_previews(
        &mut self,
        token_request: TokenRequest,
        issuer_url: BaseUrl,
        mdoc_trust_anchors: &Vec<TrustAnchor<'_>>,
        is_pid: bool,
    ) -> Result<Vec<Attestation>, IssuanceError> {
        let http_client = default_reqwest_client_builder()
            .default_headers(HeaderMap::from_iter([(
                header::ACCEPT,
                HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
            )]))
            .build()
            .expect("Could not build reqwest HTTP client");

        let issuance_session = IS::start_issuance(
            HttpVcMessageClient::new(NL_WALLET_CLIENT_ID.to_string(), http_client),
            issuer_url,
            token_request,
            mdoc_trust_anchors,
        )
        .await?;

        info!("successfully received token and previews from issuer");
        let attestations = issuance_session
            .normalized_credential_preview()
            .iter()
            .map(|preview_data| {
                let attestation = Attestation::create_from_attributes(
                    AttestationIdentity::Ephemeral,
                    preview_data.normalized_metadata.clone(),
                    preview_data.issuer_registration.organization.clone(),
                    preview_data.content.credential_payload.attributes.clone(),
                )
                .map_err(|error| IssuanceError::Attestation {
                    organization: Box::new(preview_data.issuer_registration.organization.clone()),
                    error,
                })?;

                Ok(attestation)
            })
            .collect::<Result<Vec<_>, IssuanceError>>()?;

        self.session
            .replace(Session::Issuance(IssuanceSession::new(is_pid, issuance_session)));

        Ok(attestations)
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn accept_issuance(&mut self, pin: String) -> Result<(), IssuanceError>
    where
        UR: UpdateableRepository<VersionState, TlsPinningConfig, Error = UpdatePolicyError>,
        S: Storage,
        APC: AccountProviderClient,
        WIC: WteIssuanceClient + Default,
    {
        info!("Accepting issuance");

        let config = &self.config_repository.get().update_policy_server;

        info!("Fetching update policy");
        self.update_policy_repository.fetch(&config.http_config).await?;

        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(IssuanceError::VersionBlocked);
        }

        info!("Checking if registered");
        let (attested_key, registration_data) = self
            .registration
            .as_key_and_registration_data()
            .ok_or_else(|| IssuanceError::NotRegistered)?;

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(IssuanceError::Locked);
        }

        info!("Checking if there is an active issuance session");
        let Some(Session::Issuance(issuance_session)) = &self.session else {
            return Err(IssuanceError::SessionState);
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

        let wte = if issuance_session.is_pid {
            Some(
                self.wte_issuance_client
                    .obtain_wte(
                        config.account_server.wte_public_key.as_inner(),
                        remote_instruction.clone(),
                    )
                    .await?,
            )
        } else {
            None
        };

        let remote_key_factory = RemoteEcdsaKeyFactory::new(remote_instruction);

        info!("Signing nonce using Wallet Provider");

        let organization = issuance_session
            .protocol_state
            .issuer_registration()
            .organization
            .clone();

        let issuance_result = issuance_session
            .protocol_state
            .accept_issuance(&config.mdoc_trust_anchors(), &remote_key_factory, wte)
            .await
            .map_err(|error| {
                match error {
                    // We knowingly call unwrap() on the downcast to `RemoteEcdsaKeyError` here because we know
                    // that it is the error type of the `RemoteEcdsaKeyFactory` we provide above.
                    IssuanceSessionError::PrivateKeyGeneration(error)
                    | IssuanceSessionError::Jwt(JwtError::Signing(error)) => {
                        match *error.downcast::<RemoteEcdsaKeyError>().unwrap() {
                            RemoteEcdsaKeyError::Instruction(error) => IssuanceError::Instruction(error),
                            RemoteEcdsaKeyError::Signature(error) => IssuanceError::Signature(error),
                            RemoteEcdsaKeyError::KeyNotFound(identifier) => IssuanceError::KeyNotFound(identifier),
                            RemoteEcdsaKeyError::MissingSignature => IssuanceError::MissingSignature,
                        }
                    }
                    _ => IssuanceError::IssuerServer {
                        organization: Box::new(organization),
                        error,
                    },
                }
            });

        // Make sure there are no remaining references to the `AttestedKey` value.
        drop(remote_key_factory);

        // If the Wallet Provider returns either a PIN timeout or a permanent block,
        // wipe the contents of the wallet and return it to its initial state.
        if matches!(
            issuance_result,
            Err(IssuanceError::Instruction(
                InstructionError::Timeout { .. } | InstructionError::Blocked
            ))
        ) {
            self.reset_to_initial_state().await;
        }
        let issued_mdocs = issuance_result?
            .into_iter()
            .map(CredentialCopies::from)
            .collect::<Vec<_>>();

        info!("Isuance succeeded; removing issuance session state");
        self.session.take();

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
                    .map_err(IssuanceError::InvalidIssuerCertificate)?;

                // Verify that the certificate contains IssuerRegistration
                if matches!(IssuerRegistration::from_certificate(&certificate), Err(_) | Ok(None)) {
                    return Err(IssuanceError::MissingIssuerRegistration);
                }
            }
            WalletEvent::new_issuance(mdocs)
        };

        info!("Attestations accepted, storing mdoc in database");
        self.storage
            .write()
            .await
            .insert_mdocs(issued_mdocs)
            .await
            .map_err(IssuanceError::MdocStorage)?;

        self.store_history_event(event)
            .await
            .map_err(IssuanceError::EventStorage)?;

        self.emit_attestations().await.map_err(IssuanceError::Attestations)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use mockall::predicate::*;
    use rstest::rstest;
    use serial_test::serial;
    use url::Url;

    use attestation_data::attributes::AttributeValue;
    use attestation_data::auth::issuer_auth::IssuerRegistration;
    use http_utils::tls::pinning::TlsPinningConfig;
    use mdoc::holder::Mdoc;
    use openid4vc::issuance_session::IssuedCredential;
    use openid4vc::mock::MockIssuanceSession;
    use openid4vc::oidc::OidcError;
    use openid4vc::token::TokenRequest;
    use openid4vc::token::TokenRequestGrantType;

    use crate::attestation::AttestationAttributeValue;
    use crate::issuance::MockDigidSession;
    use crate::storage::StorageState;
    use crate::wallet::test::create_example_preview_data;

    use super::super::test;
    use super::super::test::WalletDeviceVendor;
    use super::super::test::WalletWithMocks;
    use super::*;

    fn mock_issuance_session(mdoc: Mdoc) -> MockIssuanceSession {
        let mut client = MockIssuanceSession::new();
        let issuer_certificate = mdoc.issuer_certificate().unwrap();
        client
            .expect_issuer()
            .return_const(match IssuerRegistration::from_certificate(&issuer_certificate) {
                Ok(Some(registration)) => registration,
                _ => IssuerRegistration::new_mock(),
            });

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

        assert!(wallet.session.is_none());

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
        assert!(wallet.session.is_some());
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

        assert_matches!(error, IssuanceError::Locked);
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

        assert_matches!(error, IssuanceError::NotRegistered);
    }

    #[tokio::test]
    async fn test_create_pid_issuance_auth_url_error_session_state_digid() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Set up a mock DigiD session.
        wallet.session = Some(Session::Digid(MockDigidSession::default()));

        // Creating a DigiD authentication URL on a `Wallet` that
        // has an active DigiD session should return an error.
        let error = wallet
            .create_pid_issuance_auth_url()
            .await
            .expect_err("PID issuance auth URL generation should have resulted in error");

        assert_matches!(error, IssuanceError::SessionState);
    }

    #[tokio::test]
    async fn test_create_pid_issuance_auth_url_error_session_state_pid_issuer() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Setup a mock OpenID4VCI session.
        wallet.session = Some(Session::Issuance(IssuanceSession::new(
            true,
            MockIssuanceSession::default(),
        )));

        // Creating a DigiD authentication URL on a `Wallet` that has
        // an active OpenID4VCI session should return an error.
        let error = wallet
            .create_pid_issuance_auth_url()
            .await
            .expect_err("PID issuance auth URL generation should have resulted in error");

        assert_matches!(error, IssuanceError::SessionState);
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

        assert_matches!(error, IssuanceError::DigidSessionStart(_));
    }

    #[tokio::test]
    async fn test_cancel_pid_issuance_digid() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Set up a mock DigiD session.
        wallet.session = Some(Session::Digid(MockDigidSession::default()));

        assert!(wallet.session.is_some());

        // Cancelling PID issuance should clear this session.
        wallet.cancel_issuance().await.expect("Could not cancel PID issuance");

        assert!(wallet.session.is_none());
    }

    #[tokio::test]
    async fn test_cancel_pid_issuance_pid() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Set up the `PidIssuerClient`
        let pid_issuer = {
            let mut client = MockIssuanceSession::new();
            client.expect_reject().return_once(|| Ok(()));
            client.expect_issuer().return_const(IssuerRegistration::new_mock());
            client
        };
        wallet.session = Some(Session::Issuance(IssuanceSession::new(true, pid_issuer)));

        // Cancelling PID issuance should not fail.
        wallet.cancel_issuance().await.expect("Could not cancel PID issuance");

        assert!(wallet.session.is_none());
    }

    #[tokio::test]
    async fn test_cancel_pid_issuance_error_locked() {
        // Prepare a registered and locked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        wallet.lock();

        // Cancelling PID issuance on a locked wallet should result in an error.
        let error = wallet
            .cancel_issuance()
            .await
            .expect_err("Cancelling PID issuance should have resulted in an error");

        assert_matches!(error, IssuanceError::Locked);
    }

    #[tokio::test]
    async fn test_cancel_pid_issuance_error_unregistered() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithMocks::new_unregistered(WalletDeviceVendor::Apple);

        // Cancelling PID issuance on an unregistered wallet should result in an error.
        let error = wallet
            .cancel_issuance()
            .await
            .expect_err("Cancelling PID issuance should have resulted in an error");

        assert_matches!(error, IssuanceError::NotRegistered);
    }

    #[tokio::test]
    async fn test_cancel_pid_issuance_error_session_state() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Cancelling PID issuance on a wallet with no
        // active DigiD session should result in an error.
        let error = wallet
            .cancel_issuance()
            .await
            .expect_err("Cancelling PID issuance should have resulted in an error");

        assert_matches!(error, IssuanceError::SessionState);
    }

    const REDIRECT_URI: &str = "redirect://here";

    #[tokio::test]
    #[serial(MockIssuanceSession)]
    async fn test_continue_pid_issuance() {
        let mut wallet = setup_wallet_with_digid_session();

        let preview_data = create_example_preview_data();

        // Set up the `MockIssuanceSession` to return one `CredentialPreviewState`.
        let start_context = MockIssuanceSession::start_context();
        start_context.expect().return_once(|| {
            let mut client = MockIssuanceSession::new();

            client
                .expect_normalized_credential_previews()
                .return_const(vec![preview_data]);

            Ok(client)
        });

        // Continuing PID issuance should result in one preview `Attestation`.
        let attestations = wallet
            .continue_pid_issuance(Url::parse(REDIRECT_URI).unwrap())
            .await
            .expect("Could not continue PID issuance");

        assert_eq!(attestations.len(), 1);

        let attestation = attestations.into_iter().next().unwrap();
        assert_matches!(attestation.identity, AttestationIdentity::Ephemeral);
        assert_eq!(attestation.attributes.len(), 4);
        assert_eq!(attestation.attributes[0].key, vec!["family_name".to_string()]);
        assert_matches!(
            &attestation.attributes[0].value,
            AttestationAttributeValue::Basic(AttributeValue::Text(string)) if string == "De Bruijn"
        );
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

        assert_matches!(error, IssuanceError::Locked);
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

        assert_matches!(error, IssuanceError::NotRegistered);
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

        assert_matches!(error, IssuanceError::SessionState);
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
        wallet.session = Some(Session::Digid(digid_session));

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

        assert_matches!(error, IssuanceError::IssuanceSession { .. });
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

            client.expect_issuer().return_const(IssuerRegistration::new_mock());

            client
        };
        wallet.session = Some(Session::Issuance(IssuanceSession::new(true, pid_issuer)));

        // Canceling PID issuance on a wallet should forward this error.
        let error = wallet
            .cancel_issuance()
            .await
            .expect_err("Rejecting PID issuance should have resulted in an error");

        assert_matches!(error, IssuanceError::IssuerServer { .. });
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
        wallet.session = Some(Session::Issuance(IssuanceSession::new(true, pid_issuer)));

        // Accept the PID issuance with the PIN.
        wallet
            .accept_issuance(PIN.to_owned())
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
            assert_eq!(attestation.attestation_type, PID_DOCTYPE);

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
        assert_matches!(err, IssuanceError::PidAlreadyPresent);
    }

    #[tokio::test]
    async fn test_accept_pid_issuance_missing_issuer_registration() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Create a mock OpenID4VCI session that accepts the PID with a single instance of `MdocCopies`, which contains
        // a single valid `Mdoc`, but signed with a Certificate that is missing IssuerRegistration
        let mdoc = test::create_example_pid_mdoc_unauthenticated();
        let pid_issuer = mock_issuance_session(mdoc);
        wallet.session = Some(Session::Issuance(IssuanceSession::new(true, pid_issuer)));

        // Accept the PID issuance with the PIN.
        let error = wallet
            .accept_issuance(PIN.to_owned())
            .await
            .expect_err("Accepting PID issuance should have resulted in an error");

        assert_matches!(error, IssuanceError::MissingIssuerRegistration);

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
            .accept_issuance(PIN.to_owned())
            .await
            .expect_err("Accepting PID issuance should have resulted in an error");

        assert_matches!(error, IssuanceError::NotRegistered);
    }

    #[tokio::test]
    async fn test_accept_pid_issuance_locked() {
        // Prepare a registered and locked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        wallet.lock();

        // Accepting PID issuance on a locked wallet should result in an error.
        let error = wallet
            .accept_issuance(PIN.to_owned())
            .await
            .expect_err("Accepting PID issuance should have resulted in an error");

        assert_matches!(error, IssuanceError::Locked);

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
            .accept_issuance(PIN.to_owned())
            .await
            .expect_err("Accepting PID issuance should have resulted in an error");

        assert_matches!(error, IssuanceError::SessionState);

        assert!(wallet.has_registration());
        assert!(!wallet.is_locked());
    }

    async fn test_accept_pid_issuance_error_remote_key(
        key_error: RemoteEcdsaKeyError,
    ) -> (WalletWithMocks, IssuanceError) {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Have the mock OpenID4VCI session return a particular `RemoteEcdsaKeyError` upon accepting.
        let pid_issuer = {
            let mut client = MockIssuanceSession::new();
            client
                .expect_accept()
                .return_once(|| Err(IssuanceSessionError::Jwt(JwtError::Signing(Box::new(key_error)))));

            client.expect_issuer().return_const(IssuerRegistration::new_mock());

            client
        };
        wallet.session = Some(Session::Issuance(IssuanceSession::new(true, pid_issuer)));

        // Accepting PID issuance should result in an error.
        let error = wallet
            .accept_issuance(PIN.to_owned())
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

        // Test that this error is converted to the appropriate variant of `IssuanceError`.
        assert_matches!(error, IssuanceError::Instruction(_));

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

        // Test that this error is converted to the appropriate variant of `IssuanceError`.
        assert_matches!(error, IssuanceError::Signature(_));

        assert!(wallet.has_registration());
        assert!(!wallet.is_locked());
    }

    #[tokio::test]
    async fn test_accept_pid_issuance_error_key_not_found() {
        let (wallet, error) =
            test_accept_pid_issuance_error_remote_key(RemoteEcdsaKeyError::KeyNotFound("not found".to_string())).await;

        // Test that this error is converted to the appropriate variant of `IssuanceError`.
        assert_matches!(error, IssuanceError::KeyNotFound(_));

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

            client.expect_issuer().return_const(IssuerRegistration::new_mock());

            client
        };
        wallet.session = Some(Session::Issuance(IssuanceSession::new(true, pid_issuer)));

        // Accepting PID issuance should result in an error.
        let error = wallet
            .accept_issuance(PIN.to_owned())
            .await
            .expect_err("Accepting PID issuance should have resulted in an error");

        assert_matches!(error, IssuanceError::IssuerServer { .. });

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
        wallet.session = Some(Session::Issuance(IssuanceSession::new(true, pid_issuer)));

        // Have the mdoc storage return an error on query.
        wallet.storage.write().await.has_query_error = true;

        // Accepting PID issuance should result in an error.
        let error = wallet
            .accept_issuance(PIN.to_owned())
            .await
            .expect_err("Accepting PID issuance should have resulted in an error");

        assert_matches!(error, IssuanceError::MdocStorage(_));

        assert!(wallet.has_registration());
        assert!(!wallet.is_locked());
    }
}
