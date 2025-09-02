use std::sync::Arc;

use chrono::DateTime;
use chrono::Utc;
use derive_more::Constructor;
use itertools::Itertools;
use p256::ecdsa::signature;
use rustls_pki_types::TrustAnchor;
use tracing::info;
use tracing::instrument;
use url::Url;
use uuid::Uuid;

use attestation_data::auth::Organization;
use attestation_data::constants::PID_ATTESTATION_TYPE;
use attestation_data::constants::PID_RECOVERY_CODE;
use attestation_data::credential_payload::CredentialPayload;
use attestation_types::claim_path::ClaimPath;
use crypto::x509::CertificateError;
use error_category::ErrorCategory;
use error_category::sentry_capture_error;
use http_utils::reqwest::client_builder_accept_json;
use http_utils::reqwest::default_reqwest_client_builder;
use http_utils::tls::pinning::TlsPinningConfig;
use http_utils::urls;
use http_utils::urls::BaseUrl;
use jwt::error::JwtError;
use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::issuance_session::CredentialWithMetadata;
use openid4vc::issuance_session::HttpVcMessageClient;
use openid4vc::issuance_session::IssuanceSession;
use openid4vc::issuance_session::IssuanceSessionError;
use openid4vc::issuance_session::IssuedCredential;
use openid4vc::issuance_session::NormalizedCredentialPreview;
use openid4vc::token::CredentialPreviewError;
use openid4vc::token::TokenRequest;
use platform_support::attested_key::AppleAttestedKey;
use platform_support::attested_key::AttestedKeyHolder;
use platform_support::attested_key::GoogleAttestedKey;
use update_policy_model::update_policy::VersionState;
use utils::generator::Generator;
use utils::generator::TimeGenerator;
use utils::vec_at_least::VecNonEmpty;
use wallet_account::NL_WALLET_CLIENT_ID;
use wallet_account::messages::instructions::DiscloseRecoveryCode;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::account_provider::AccountProviderClient;
use crate::attestation::AttestationError;
use crate::attestation::AttestationIdentity;
use crate::attestation::AttestationPresentation;
use crate::config::UNIVERSAL_LINK_BASE_URL;
use crate::digid::DigidClient;
use crate::digid::DigidError;
use crate::digid::DigidSession;
use crate::errors::ChangePinError;
use crate::errors::UpdatePolicyError;
use crate::instruction::InstructionClient;
use crate::instruction::InstructionError;
use crate::instruction::RemoteEcdsaKeyError;
use crate::instruction::RemoteEcdsaWscd;
use crate::repository::Repository;
use crate::repository::UpdateableRepository;
use crate::storage::AttestationFormatQuery;
use crate::storage::Storage;
use crate::storage::StorageError;
use crate::storage::StoredAttestationCopy;
use crate::wallet::Session;
use crate::wallet::attestations::AttestationsError;

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
    DigidSessionStart(#[source] DigidError),

    #[error("could not finish DigiD session: {0}")]
    DigidSessionFinish(#[source] DigidError),

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

    #[error("could not insert attestations in database: {0}")]
    AttestationStorage(#[source] StorageError),

    #[error("could not query attestations in database: {0}")]
    AttestationQuery(#[source] StorageError),

    #[error("could not store event in history database: {0}")]
    EventStorage(#[source] StorageError),

    #[error("key '{0}' not found in Wallet Provider")]
    #[category(pd)]
    KeyNotFound(String),

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

    #[error("PID attestation in SD JWT format is missing")]
    #[category(critical)]
    MissingPidSdJwt,

    #[error("could not add recovery code disclosure: {0}")]
    #[category(pd)]
    RecoveryCodeDisclosure(sd_jwt::error::Error),
}

#[derive(Debug, Clone, Constructor)]
pub struct WalletIssuanceSession<IS> {
    is_pid: bool,
    preview_attestations: VecNonEmpty<AttestationPresentation>,
    protocol_state: IS,
}

impl<CR, UR, S, AKH, APC, DC, IS, DCC> Wallet<CR, UR, S, AKH, APC, DC, IS, DCC>
where
    CR: Repository<Arc<WalletConfiguration>>,
    UR: Repository<VersionState>,
    S: Storage,
    AKH: AttestedKeyHolder,
    DC: DigidClient,
    IS: IssuanceSession,
    DCC: DisclosureClient,
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
            .has_any_attestations_with_type(PID_ATTESTATION_TYPE)
            .await
            .map_err(IssuanceError::AttestationQuery)?;
        if has_pid {
            return Err(IssuanceError::PidAlreadyPresent);
        }

        let pid_issuance_config = &self.config_repository.get().pid_issuance;
        let session = self
            .digid_client
            .start_session(
                pid_issuance_config.digid.clone(),
                pid_issuance_config.digid_http_config.clone(),
                urls::issuance_base_uri(&UNIVERSAL_LINK_BASE_URL).as_ref().to_owned(),
            )
            .await
            .map_err(IssuanceError::DigidSessionStart)?;

        info!("DigiD auth URL generated");
        let auth_url = session.auth_url().clone();
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
    pub async fn continue_pid_issuance(
        &mut self,
        redirect_uri: Url,
    ) -> Result<Vec<AttestationPresentation>, IssuanceError> {
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

        let pid_issuance_config = &self.config_repository.get().pid_issuance;
        let token_request = session
            .into_token_request(&pid_issuance_config.digid_http_config, redirect_uri)
            .await
            .map_err(IssuanceError::DigidSessionFinish)?;

        let config = self.config_repository.get();

        self.issuance_fetch_previews(
            token_request,
            config.pid_issuance.pid_issuer_url.clone(),
            &config.issuer_trust_anchors(),
            true,
        )
        .await
    }

    #[instrument(skip_all)]
    pub(super) async fn issuance_fetch_previews(
        &mut self,
        token_request: TokenRequest,
        issuer_url: BaseUrl,
        issuer_trust_anchors: &Vec<TrustAnchor<'_>>,
        is_pid: bool,
    ) -> Result<Vec<AttestationPresentation>, IssuanceError> {
        let http_client = client_builder_accept_json(default_reqwest_client_builder())
            .build()
            .expect("Could not build reqwest HTTP client");

        let issuance_session = IS::start_issuance(
            HttpVcMessageClient::new(NL_WALLET_CLIENT_ID.to_string(), http_client),
            issuer_url,
            token_request,
            issuer_trust_anchors,
        )
        .await?;

        let preview_attestation_types = issuance_session
            .normalized_credential_preview()
            .iter()
            .map(|preview| preview.content.credential_payload.attestation_type.as_str())
            .collect();

        let stored = self
            .storage
            .read()
            .await
            .fetch_unique_attestations_by_type(&preview_attestation_types, AttestationFormatQuery::Any)
            .await
            .map_err(IssuanceError::AttestationQuery)?;

        // For every preview, try to find the first matching stored attestation to determine its database identity. If
        // there are more candidates, the algorithm matches the first one based on the ascending order of the Uuidv7 of
        // the list of stored attestations. This means the oldest attestation is matched first.
        let previews_and_identity: Vec<(&NormalizedCredentialPreview, Option<Uuid>)> =
            match_preview_and_stored_attestations(
                issuance_session.normalized_credential_preview(),
                stored,
                &TimeGenerator,
            );

        info!("successfully received token and previews from issuer");
        let organization = &issuance_session.issuer_registration().organization;
        let attestations = previews_and_identity
            .into_iter()
            .map(|(preview_data, identity)| {
                let attestation = AttestationPresentation::create_from_attributes(
                    identity.map_or(AttestationIdentity::Ephemeral, |id| AttestationIdentity::Fixed { id }),
                    preview_data.normalized_metadata.clone(),
                    organization.clone(),
                    &preview_data.content.credential_payload.attributes,
                )
                .map_err(|error| IssuanceError::Attestation {
                    organization: Box::new(organization.clone()),
                    error,
                })?;

                Ok(attestation)
            })
            .collect::<Result<Vec<_>, IssuanceError>>()?;

        // The IssuanceSession trait guarantees that credential_preview_data()
        // returns at least one value, so this unwrap() is safe.
        let event_attestations = attestations.clone().try_into().unwrap();
        self.session.replace(Session::Issuance(WalletIssuanceSession::new(
            is_pid,
            event_attestations,
            issuance_session,
        )));

        Ok(attestations)
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn accept_issuance(&mut self, pin: String) -> Result<(), IssuanceError>
    where
        UR: UpdateableRepository<VersionState, TlsPinningConfig, Error = UpdatePolicyError>,
        S: Storage,
        APC: AccountProviderClient,
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

        let remote_wscd = RemoteEcdsaWscd::new(remote_instruction.clone());
        info!("Signing nonce using Wallet Provider");

        let organization = issuance_session
            .protocol_state
            .issuer_registration()
            .organization
            .clone();

        let issuance_result = issuance_session
            .protocol_state
            .accept_issuance(&config.issuer_trust_anchors(), &remote_wscd, issuance_session.is_pid)
            .await
            .map_err(|error| {
                match error {
                    // We knowingly call unwrap() on the downcast to `RemoteEcdsaKeyError` here because we know
                    // that it is the error type of the `RemoteEcdsaWscd` we provide above.
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
        drop(remote_wscd);

        // If the Wallet Provider returns either a PIN timeout or a permanent block,
        // wipe the contents of the wallet and return it to its initial state.
        let issued_credentials_with_metadata = match issuance_result {
            Err(IssuanceError::Instruction(error @ (InstructionError::Timeout { .. } | InstructionError::Blocked))) => {
                drop(remote_instruction);
                self.reset_to_initial_state().await;
                return Err(IssuanceError::Instruction(error));
            }
            _ => issuance_result?,
        };

        info!("Isuance succeeded; removing issuance session state");
        let issuance_session = match self.session.take() {
            Some(Session::Issuance(issuance_session)) => issuance_session,
            _ => unreachable!(),
        };

        if issuance_session.is_pid {
            info!("This is a PID issuance session, therefore disclosing recovery code");
            self.disclose_recovery_code(&remote_instruction, &issued_credentials_with_metadata)
                .await?;
        }

        let all_previews = issued_credentials_with_metadata
            .into_iter()
            .zip_eq(issuance_session.preview_attestations)
            .collect_vec();

        let (existing, new): (Vec<_>, Vec<_>) = all_previews
            .into_iter()
            .partition(|(_, preview)| matches!(preview.identity, AttestationIdentity::Fixed { .. }));

        info!("Attestations accepted, storing credentials in database");
        if !existing.is_empty() {
            self.storage
                .write()
                .await
                .update_credentials(
                    Utc::now(),
                    existing
                        .into_iter()
                        .map(|(credential, preview)| (credential.copies, preview))
                        .collect_vec(),
                )
                .await
                .map_err(IssuanceError::AttestationStorage)?;
        }
        if !new.is_empty() {
            self.storage
                .write()
                .await
                .insert_credentials(Utc::now(), new)
                .await
                .map_err(IssuanceError::AttestationStorage)?;
        }

        self.emit_attestations().await.map_err(IssuanceError::Attestations)?;
        self.emit_recent_history().await.map_err(IssuanceError::EventStorage)?;

        Ok(())
    }

    /// Finds the PID SD JWT, creates a disclosure of just the recovery code, and sends it to the remote instruction
    /// endpoint of the Wallet Provider.
    async fn disclose_recovery_code<AK: AppleAttestedKey, GK: GoogleAttestedKey>(
        &self,
        instruction_client: &InstructionClient<S, AK, GK, APC>,
        issued_credentials_with_metadata: &[CredentialWithMetadata],
    ) -> Result<(), IssuanceError> {
        let pid = issued_credentials_with_metadata
            .iter()
            .find_map(|cred| {
                (cred.attestation_type == PID_ATTESTATION_TYPE).then(|| {
                    cred.copies.as_ref().iter().find_map(|copy| match copy {
                        IssuedCredential::MsoMdoc(_) => None,
                        IssuedCredential::SdJwt(verified_sd_jwt) => Some(verified_sd_jwt.clone()),
                    })
                })
            })
            .flatten()
            .ok_or(IssuanceError::MissingPidSdJwt)?
            .into_inner();
        let recovery_code_disclosure = pid
            .into_presentation_builder()
            .disclose(
                &vec![ClaimPath::SelectByKey(PID_RECOVERY_CODE.to_owned())]
                    .try_into()
                    .unwrap(),
            )
            .map_err(IssuanceError::RecoveryCodeDisclosure)?
            .finish();

        instruction_client
            .send(DiscloseRecoveryCode {
                recovery_code_disclosure: recovery_code_disclosure.into(),
            })
            .await?;

        Ok(())
    }
}

fn match_preview_and_stored_attestations<'a>(
    previews: &'a [NormalizedCredentialPreview],
    stored_attestations: Vec<StoredAttestationCopy>,
    time_generator: &impl Generator<DateTime<Utc>>,
) -> Vec<(&'a NormalizedCredentialPreview, Option<Uuid>)> {
    let stored_credential_payloads: Vec<(CredentialPayload, Uuid)> = stored_attestations
        .into_iter()
        .map(|copy| {
            let attestation_id = copy.attestation_id();

            (copy.into_credential_payload(), attestation_id)
        })
        .collect_vec();

    // Find the first matching stored preview based on the ordering of `stored_credential_payloads`.
    previews
        .iter()
        .map(|preview| {
            let identity = stored_credential_payloads
                .iter()
                .find(|(stored_preview, _)| {
                    preview
                        .content
                        .credential_payload
                        .matches_existing(&stored_preview.previewable_payload, time_generator)
                })
                .map(|(_, id)| *id);

            (preview, identity)
        })
        .collect_vec()
}

#[cfg(test)]
mod tests {
    use std::ops::Add;

    use assert_matches::assert_matches;
    use chrono::Duration;
    use futures::FutureExt;
    use itertools::multiunzip;
    use mockall::predicate::*;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;
    use rstest::rstest;
    use serial_test::serial;
    use url::Url;
    use uuid::Uuid;

    use attestation_data::attributes::AttributeValue;
    use attestation_data::auth::issuer_auth::IssuerRegistration;
    use attestation_data::constants::PID_ATTESTATION_TYPE;
    use attestation_data::credential_payload::IntoCredentialPayload;
    use attestation_data::x509::CertificateType;
    use crypto::server_keys::generate::Ca;
    use crypto::x509::BorrowingCertificateExtension;
    use openid4vc::issuance_session::CredentialWithMetadata;
    use openid4vc::issuance_session::IssuedCredential;
    use openid4vc::issuance_session::IssuedCredentialCopies;
    use openid4vc::mock::MockIssuanceSession;
    use openid4vc::oidc::OidcError;
    use openid4vc::token::TokenRequest;
    use openid4vc::token::TokenRequestGrantType;
    use sd_jwt::sd_jwt::VerifiedSdJwt;
    use sd_jwt_vc_metadata::VerifiedTypeMetadataDocuments;
    use utils::generator::mock::MockTimeGenerator;
    use wallet_account::messages::instructions::DiscloseRecoveryCodeResult;
    use wallet_account::messages::instructions::Instruction;

    use crate::WalletEvent;
    use crate::attestation::AttestationAttributeValue;
    use crate::digid::MockDigidSession;
    use crate::storage::StorageState;
    use crate::storage::StoredAttestation;
    use crate::wallet::test::WalletWithStorageMock;
    use crate::wallet::test::create_example_credential_payload;
    use crate::wallet::test::create_example_preview_data;
    use crate::wallet::test::create_wp_result;

    use super::super::test;
    use super::super::test::WalletDeviceVendor;
    use super::super::test::WalletWithMocks;
    use super::*;

    fn mock_issuance_session(
        credential: IssuedCredential,
        attestation_type: String,
        type_metadata: VerifiedTypeMetadataDocuments,
    ) -> (MockIssuanceSession, VecNonEmpty<AttestationPresentation>) {
        let mut client = MockIssuanceSession::new();
        let issuer_certificate = match &credential {
            IssuedCredential::MsoMdoc(mdoc) => mdoc.issuer_certificate().unwrap(),
            IssuedCredential::SdJwt(sd_jwt) => sd_jwt.as_ref().as_ref().issuer_certificate().unwrap().to_owned(),
        };

        let issuer_registration = match IssuerRegistration::from_certificate(&issuer_certificate) {
            Ok(Some(registration)) => registration,
            _ => IssuerRegistration::new_mock(),
        };

        let attestations = vec![match &credential {
            IssuedCredential::MsoMdoc(mdoc) => AttestationPresentation::create_from_mdoc(
                AttestationIdentity::Ephemeral,
                type_metadata.to_normalized().unwrap(),
                issuer_registration.organization.clone(),
                mdoc.issuer_signed.clone().into_entries_by_namespace(),
            )
            .unwrap(),
            IssuedCredential::SdJwt(sd_jwt) => {
                let payload = sd_jwt
                    .clone()
                    .into_inner()
                    .into_credential_payload(&type_metadata.to_normalized().unwrap())
                    .unwrap();
                AttestationPresentation::create_from_attributes(
                    AttestationIdentity::Ephemeral,
                    type_metadata.to_normalized().unwrap(),
                    issuer_registration.organization.clone(),
                    &payload.previewable_payload.attributes,
                )
                .unwrap()
            }
        }]
        .try_into()
        .unwrap();

        client.expect_issuer().return_const(issuer_registration);

        client.expect_accept().return_once(move || {
            Ok(vec![CredentialWithMetadata::new(
                IssuedCredentialCopies::new_or_panic(VecNonEmpty::try_from(vec![credential]).unwrap()),
                attestation_type,
                type_metadata,
            )])
        });

        (client, attestations)
    }

    #[tokio::test]
    async fn test_create_pid_issuance_auth_url() {
        const AUTH_URL: &str = "http://example.com/auth";

        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        assert!(wallet.session.is_none());

        // Set up a mock DigiD session.
        wallet.digid_client.expect_start_session().times(1).return_once(
            |_digid_config, _http_config, _redirect_uri| {
                let mut session = MockDigidSession::new();

                session.expect_auth_url().return_const(Url::parse(AUTH_URL).unwrap());

                Ok(session)
            },
        );

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
        wallet.session = Some(Session::Digid(MockDigidSession::new()));

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
        wallet.session = Some(Session::Issuance(WalletIssuanceSession::new(
            true,
            vec![AttestationPresentation::new_mock()].try_into().unwrap(),
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
    async fn test_create_pid_issuance_auth_url_error_digid_session_start() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Make DigiD session starting return an error.
        wallet
            .digid_client
            .expect_start_session()
            .times(1)
            .return_once(|_digid_config, _http_config, _redirect_uri| Err(OidcError::NoAuthCode.into()));

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
        wallet.session = Some(Session::Digid(MockDigidSession::new()));

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
        wallet.session = Some(Session::Issuance(WalletIssuanceSession::new(
            true,
            vec![AttestationPresentation::new_mock()].try_into().unwrap(),
            pid_issuer,
        )));

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

        // Set up the `MockIssuanceSession` to return one `CredentialPreviewState`.
        let start_context = MockIssuanceSession::start_context();
        start_context.expect().return_once(|| {
            let mut client = MockIssuanceSession::new();

            client
                .expect_normalized_credential_previews()
                .return_const(vec![create_example_preview_data(&MockTimeGenerator::default())]);

            client.expect_issuer().return_const(IssuerRegistration::new_mock());

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

    fn mock_digid_session() -> MockDigidSession<TlsPinningConfig> {
        // Set up a mock DigiD session that returns a token request.
        let mut session = MockDigidSession::new();

        session
            .expect_into_token_request()
            .return_once(|_http_config, _redirect_uri| {
                let token_request = TokenRequest {
                    grant_type: TokenRequestGrantType::PreAuthorizedCode {
                        pre_authorized_code: "123".to_string().into(),
                    },
                    code_verifier: None,
                    client_id: None,
                    redirect_uri: None,
                };

                Ok(token_request)
            });

        session
    }

    fn setup_wallet_with_digid_session() -> WalletWithMocks {
        // Prepare a registered wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);
        wallet.session = Some(Session::Digid(mock_digid_session()));
        wallet
    }

    fn setup_wallet_with_digid_session_and_database_mock() -> WalletWithStorageMock {
        // Prepare a registered wallet.
        let mut wallet =
            WalletWithStorageMock::new_registered_and_unlocked_with_storage_mock(WalletDeviceVendor::Apple);
        wallet.session = Some(Session::Digid(mock_digid_session()));
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
    #[serial(MockIssuanceSession)]
    async fn test_continue_pid_issuance_with_renewed_attestation() {
        let mut wallet = setup_wallet_with_digid_session_and_database_mock();

        let time_generator = MockTimeGenerator::default();

        // When the attestation already exists in the database, we expect the identity to be known
        let mut previews = vec![create_example_preview_data(&time_generator)];

        // When the attestation already exists in the database, but the preview has a newer nbf, it should be
        // considered as a new attestation and the identity is None.
        let mut preview = create_example_preview_data(&time_generator);
        preview.content.credential_payload.not_before = Some(Utc::now().add(Duration::days(365)).into());
        previews.push(preview);

        // When the attestation_type is different from the one stored in the database, it should be
        // considered as a new attestation and the identity is None.
        let mut preview = create_example_preview_data(&time_generator);
        preview.content.credential_payload.attestation_type = String::from("att_type_1");
        previews.push(preview);

        let holder_key = SigningKey::random(&mut OsRng);
        let ca = Ca::generate("myca", Default::default()).unwrap();
        let cert_type = CertificateType::from(IssuerRegistration::new_mock());
        let issuer_key_pair = ca.generate_key_pair("mycert", cert_type, Default::default()).unwrap();

        let (payload, _, normalized_metadata) = create_example_credential_payload(&time_generator);
        let sd_jwt = payload
            .into_sd_jwt(&normalized_metadata, holder_key.verifying_key(), &issuer_key_pair)
            .now_or_never()
            .unwrap()
            .unwrap();

        let attestation_id = Uuid::new_v4();
        let stored = StoredAttestationCopy::new(
            attestation_id,
            Uuid::new_v4(),
            StoredAttestation::SdJwt {
                sd_jwt: Box::new(VerifiedSdJwt::new_mock(sd_jwt)),
            },
            normalized_metadata,
        );

        let storage = wallet.mut_storage();
        storage
            .expect_fetch_unique_attestations_by_type()
            .return_once(move |_attestation_types, _format| Ok(vec![stored]));

        // Set up the `MockIssuanceSession` to return one `CredentialPreviewState`.
        let start_context = MockIssuanceSession::start_context();
        start_context.expect().return_once(|| {
            let mut client = MockIssuanceSession::new();

            client.expect_normalized_credential_previews().return_const(previews);

            client.expect_issuer().return_const(IssuerRegistration::new_mock());

            Ok(client)
        });

        // Continuing PID issuance should result in one preview `Attestation`.
        let attestations = wallet
            .continue_pid_issuance(Url::parse(REDIRECT_URI).unwrap())
            .await
            .expect("Could not continue PID issuance");

        assert_eq!(attestations.len(), 3);

        assert_matches!(
            &attestations[0].identity,
            AttestationIdentity::Fixed { id } if id == &attestation_id);
        assert_matches!(&attestations[1].identity, AttestationIdentity::Ephemeral);
        assert_matches!(&attestations[2].identity, AttestationIdentity::Ephemeral);
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
        wallet.session = Some(Session::Issuance(WalletIssuanceSession::new(
            true,
            vec![AttestationPresentation::new_mock()].try_into().unwrap(),
            pid_issuer,
        )));

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
        let attestations_callback = test::setup_mock_attestations_callback(&mut wallet).await.unwrap();

        // Register mock recent_history_callback
        let events = test::setup_mock_recent_history_callback(&mut wallet).await.unwrap();

        // Create a mock OpenID4VCI session that accepts the PID with a single
        // instance of `MdocCopies`, which contains a single valid `Mdoc`.
        let credential = test::create_example_pid_sd_jwt_credential();
        let (pid_issuer, attestations) = mock_issuance_session(
            credential.copies.into_inner().first().to_owned(),
            credential.attestation_type,
            credential.metadata_documents,
        );
        wallet.session = Some(Session::Issuance(WalletIssuanceSession::new(
            true,
            attestations,
            pid_issuer,
        )));

        setup_mock_recovery_code_instructions(&mut wallet);

        // Accept the PID issuance with the PIN.
        wallet
            .accept_issuance(PIN.to_owned())
            .await
            .expect("Could not accept PID issuance");

        {
            // Test which `Attestation` instances we have received through the callback.
            let attestations = attestations_callback.lock();

            // The first entry should be empty, because there are no mdocs in the database.
            assert_eq!(attestations.len(), 2);
            assert!(attestations[0].is_empty());

            // The second entry should contain a single attestation with the PID.
            assert_eq!(attestations[1].len(), 1);
            let attestation = &attestations[1][0];
            assert_matches!(attestation.identity, AttestationIdentity::Fixed { id: _ });
            assert_eq!(attestation.attestation_type, PID_ATTESTATION_TYPE);

            // Test that one successful issuance event is logged
            let events = events.lock();
            assert_eq!(events.len(), 2);
            assert!(events[0].is_empty());
            assert_eq!(events[1].len(), 1);
            assert_matches!(&events[1][0], WalletEvent::Issuance { .. });

            assert!(wallet.has_registration());
            assert!(!wallet.is_locked());
        }

        let err = wallet
            .create_pid_issuance_auth_url()
            .await
            .expect_err("creating new PID issuance auth URL when there already is a PID should fail");
        assert_matches!(err, IssuanceError::PidAlreadyPresent);
    }

    fn setup_mock_recovery_code_instructions(wallet: &mut WalletWithMocks) {
        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_instruction_challenge()
            .once()
            .returning(|_, _| Ok(crypto::utils::random_bytes(32)));

        let wp_result = create_wp_result(DiscloseRecoveryCodeResult {
            transfer_available: false,
        });

        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_instruction()
            .once()
            .return_once(move |_, _: Instruction<DiscloseRecoveryCode>| Ok(wp_result));
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
        wallet.session = Some(Session::Issuance(WalletIssuanceSession::new(
            true,
            vec![AttestationPresentation::new_mock()].try_into().unwrap(),
            pid_issuer,
        )));

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
        wallet.session = Some(Session::Issuance(WalletIssuanceSession::new(
            true,
            vec![AttestationPresentation::new_mock()].try_into().unwrap(),
            pid_issuer,
        )));

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
        let credential = test::create_example_pid_sd_jwt_credential();
        let (pid_issuer, attestations) = mock_issuance_session(
            credential.copies.into_inner().first().to_owned(),
            credential.attestation_type,
            credential.metadata_documents,
        );
        wallet.session = Some(Session::Issuance(WalletIssuanceSession::new(
            true,
            attestations,
            pid_issuer,
        )));

        // Have the mdoc storage return an error on query.
        wallet.storage.write().await.has_query_error = true;

        setup_mock_recovery_code_instructions(&mut wallet);

        // Accepting PID issuance should result in an error.
        let error = wallet
            .accept_issuance(PIN.to_owned())
            .await
            .expect_err("Accepting PID issuance should have resulted in an error");

        assert_matches!(error, IssuanceError::AttestationStorage(_));

        assert!(wallet.has_registration());
        assert!(!wallet.is_locked());
    }

    #[test]
    fn test_match_preview_and_stored_attestations() {
        let holder_key = SigningKey::random(&mut OsRng);

        let ca = Ca::generate("myca", Default::default()).unwrap();
        let cert_type = CertificateType::from(IssuerRegistration::new_mock());
        let issuer_key_pair = ca.generate_key_pair("mycert", cert_type, Default::default()).unwrap();

        let time_generator = MockTimeGenerator::default();

        let (payload, _, normalized_metadata) = create_example_credential_payload(&time_generator);
        let sd_jwt = payload
            .into_sd_jwt(&normalized_metadata, holder_key.verifying_key(), &issuer_key_pair)
            .now_or_never()
            .unwrap()
            .unwrap();

        let attestation_id = Uuid::new_v4();
        let stored = StoredAttestationCopy::new(
            attestation_id,
            Uuid::new_v4(),
            StoredAttestation::SdJwt {
                sd_jwt: Box::new(VerifiedSdJwt::new_mock(sd_jwt)),
            },
            normalized_metadata,
        );

        // When the attestation already exists in the database, we expect the identity to be known
        let previews = [create_example_preview_data(&time_generator)];
        let result = match_preview_and_stored_attestations(&previews, vec![stored.clone()], &time_generator);
        let (_, identities): (Vec<_>, Vec<_>) = multiunzip(result);
        assert_eq!(vec![Some(attestation_id)], identities);

        // When the attestation already exists in the database, but the preview has a newer nbf, it should be considered
        // as a new attestation and the identity is None.
        let mut preview = create_example_preview_data(&time_generator);
        preview.content.credential_payload.not_before = Some(Utc::now().add(Duration::days(365)).into());
        let previews = [preview];
        let result = match_preview_and_stored_attestations(&previews, vec![stored.clone()], &time_generator);
        let (_, identities): (Vec<_>, Vec<_>) = multiunzip(result);
        assert_eq!(vec![None], identities);

        // When the attestation doesn't exists in the database, the identity is None.
        let mut preview = create_example_preview_data(&time_generator);
        preview.content.credential_payload.attestation_type = String::from("att_type_1");
        let previews = [preview];
        let result = match_preview_and_stored_attestations(&previews, vec![stored], &time_generator);
        let (_, identities): (Vec<_>, Vec<_>) = multiunzip(result);
        assert_eq!(vec![None], identities);
    }
}
