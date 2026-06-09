use std::collections::HashMap;
use std::sync::Arc;

use attestation_data::auth::Organization;
use attestation_data::credential_payload::PreviewableCredentialPayload;
use attestation_data::validity::ValidityWindow;
use attestation_types::claim_path::ClaimPath;
use chrono::DateTime;
use chrono::Utc;
use crypto::x509::CertificateError;
use error_category::ErrorCategory;
use error_category::sentry_capture_error;
use http_utils::client::TlsPinningConfig;
use http_utils::urls;
use itertools::Itertools;
use jwt::error::JwtError;
use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::metadata::issuer_metadata::CredentialConfigurationId;
use openid4vc::token::CredentialPreview;
use openid4vc::token::CredentialPreviewError;
use openid4vc::wallet_issuance::AuthorizationSession;
use openid4vc::wallet_issuance::IssuanceDiscovery;
use openid4vc::wallet_issuance::IssuanceSession;
use openid4vc::wallet_issuance::WalletIssuanceError;
use openid4vc::wallet_issuance::authorization::OAuthError;
use openid4vc::wallet_issuance::credential::CredentialWithMetadata;
use openid4vc::wallet_issuance::credential::IssuedCredential;
use p256::ecdsa::signature;
use platform_support::attested_key::AppleAttestedKey;
use platform_support::attested_key::AttestedKeyHolder;
use platform_support::attested_key::GoogleAttestedKey;
use serde::Deserialize;
use serde::Serialize;
use tracing::info;
use tracing::instrument;
use update_policy_model::update_policy::VersionState;
use url::Url;
use utils::built_info::version;
use utils::generator::Generator;
use utils::generator::TimeGenerator;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;
use uuid::Uuid;
use wallet_account::NL_WALLET_CLIENT_ID;
use wallet_account::messages::instructions::DiscloseRecoveryCode;
use wallet_configuration::wallet_config::PidAttributesConfiguration;
use wallet_configuration::wallet_config::WalletConfiguration;

use super::PersistedIssuanceSessionData;
use super::Wallet;
use crate::account_provider::AccountProviderClient;
use crate::attestation::AttestationError;
use crate::attestation::AttestationIdentity;
use crate::attestation::AttestationPresentation;
use crate::attestation::AttestationValidity;
use crate::config::UNIVERSAL_LINK_BASE_URL;
use crate::errors::ChangePinError;
use crate::errors::HistoryError;
use crate::errors::UpdatePolicyError;
use crate::instruction::InstructionClient;
use crate::instruction::InstructionError;
use crate::instruction::RemoteEcdsaKeyError;
use crate::instruction::RemoteEcdsaWscd;
use crate::repository::Repository;
use crate::repository::UpdateableRepository;
use crate::storage::Storage;
use crate::storage::StorageError;
use crate::storage::StoredAttestationCopy;
use crate::storage::TransferData;
use crate::transfer::TransferSessionId;
use crate::wallet::Session;
use crate::wallet::attestations::AttestationsError;
use crate::wallet::notifications::NotificationsError;
use crate::wallet::recovery_code::RecoveryCodeError;
use crate::wallet::state::CheckPreconditionsError;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum IssuanceError {
    #[error("preconditions failed: {0}")]
    #[category(expected)]
    CheckPreconditions(#[from] CheckPreconditionsError),

    #[error("issuance session is not in the correct state")]
    #[category(expected)]
    SessionState,

    #[error("PID already present")]
    #[category(critical)]
    PidAlreadyPresent,

    #[error("cannot renew PID: wallet has no PID")]
    #[category(critical)]
    NoPidPresent,

    #[error("user denied DigiD authentication")]
    #[category(expected)]
    AuthorizationDenied,

    #[error("could not retrieve attestations from issuer: {0}")]
    IssuanceSession(#[from] WalletIssuanceError),

    #[error("could not retrieve attestations from issuer: {error}")]
    IssuerServer {
        organization: Box<Organization>,
        #[defer]
        #[source]
        error: WalletIssuanceError,
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

    #[error("could not update issuance OAuth session state in database: {0}")]
    SessionStorage(#[source] StorageError),

    #[error("key '{0}' not found in Wallet Provider")]
    #[category(pd)]
    KeyNotFound(String),

    #[error("error emitting attestations: {0}")]
    Attestations(#[from] AttestationsError),

    #[error("error emitting notifications: {0}")]
    Notifications(#[from] NotificationsError),

    #[error("error emtting history event: {0}")]
    Events(#[from] HistoryError),

    #[error("failed to read issuer registration from issuer certificate: {0}")]
    AttestationPreview(#[from] CredentialPreviewError),

    #[error("type metadata for config id `{0}` not found")]
    #[category(critical)]
    MissingTypeMetadata(CredentialConfigurationId),

    #[error("error finalizing pin change: {0}")]
    ChangePin(#[from] ChangePinError),

    #[error("JWT credential error: {0}")]
    JwtCredential(#[from] JwtError),

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
    RecoveryCodeDisclosure(sd_jwt::error::ClaimError),

    #[error("error storing transfer data in database: {0}")]
    TransferDataStorage(#[source] StorageError),

    #[error("recovery code error: {0}")]
    RecoveryCode(#[from] RecoveryCodeError),
}

#[derive(Debug)]
pub enum WalletIssuanceSession<AS, IS> {
    OAuth {
        purpose: PidIssuancePurpose,
        authorization_session: AS,
    },
    Issuance {
        pid_purpose: Option<PidIssuancePurpose>, // None if we're not doing PID issuance
        preview_attestations: VecNonEmpty<AttestationPresentation>,
        protocol_state: IS,
    },
}

#[derive(Debug, Clone)]
pub struct IssuanceResult {
    pub transfer_session_id: Option<TransferSessionId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PidIssuancePurpose {
    Enrollment,
    Renewal,
}

#[derive(Debug, Clone, Copy)]
pub enum PidAttestationFormat {
    SdJwt,
    Either,
}

impl<CR, UR, S, AKH, APC, CID, DCC, CPC, SLC> Wallet<CR, UR, S, AKH, APC, CID, DCC, CPC, SLC>
where
    S: Storage,
    AKH: AttestedKeyHolder,
    CID: IssuanceDiscovery,
    DCC: DisclosureClient,
{
    pub(super) async fn has_pid(
        &self,
        config: &PidAttributesConfiguration,
        format: PidAttestationFormat,
    ) -> Result<bool, StorageError> {
        let pid_attestation_types = match format {
            PidAttestationFormat::Either => config.pid_attestation_types().collect_vec(),
            PidAttestationFormat::SdJwt => config.sd_jwt.keys().map(String::as_str).collect_vec(),
        };

        self.storage
            .read()
            .await
            .has_any_attestations_with_types(&pid_attestation_types)
            .await
    }
}

impl<CR, UR, S, AKH, APC, CID, DCC, CPC, SLC> Wallet<CR, UR, S, AKH, APC, CID, DCC, CPC, SLC>
where
    CR: Repository<Arc<WalletConfiguration>>,
    UR: Repository<VersionState>,
    S: Storage,
    AKH: AttestedKeyHolder,
    CID: IssuanceDiscovery,
    DCC: DisclosureClient,
    APC: AccountProviderClient,
{
    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn create_pid_issuance_auth_url(&mut self, purpose: PidIssuancePurpose) -> Result<Url, IssuanceError> {
        info!("Generating OAuth URL, starting issuer discovery");

        self.check_session_preconditions()?;

        info!("Checking if there is an active session");
        if self.session.is_some() {
            return Err(IssuanceError::SessionState);
        }

        info!("Checking if a pid is already present");

        let has_pid = self
            .has_pid(
                &self.config_repository.get().pid_attributes,
                PidAttestationFormat::Either,
            )
            .await
            .map_err(IssuanceError::AttestationQuery)?;

        if purpose == PidIssuancePurpose::Enrollment && has_pid {
            return Err(IssuanceError::PidAlreadyPresent);
        }
        if purpose == PidIssuancePurpose::Renewal && !has_pid {
            return Err(IssuanceError::NoPidPresent);
        }

        let config = self.config_repository.get();

        info!("Fetching issuer metadata to discover authorization server");
        let authorization_session = self
            .issuance_discovery
            .start_authorization_code_flow(
                &config.pid_credential_offer,
                String::from(NL_WALLET_CLIENT_ID),
                urls::issuance_base_uri(&UNIVERSAL_LINK_BASE_URL).into_inner(),
            )
            .await?;

        info!("OAuth URL generated");
        let auth_url = authorization_session.auth_url().clone();
        self.storage
            .write()
            .await
            .upsert_data(&PersistedIssuanceSessionData {
                purpose,
                authorization_session: authorization_session.persist(),
            })
            .await
            .map_err(IssuanceError::SessionStorage)?;
        self.session.replace(Session::Issuance(WalletIssuanceSession::OAuth {
            purpose,
            authorization_session,
        }));

        Ok(auth_url)
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub(super) async fn cancel_issuance(&mut self) -> Result<(), IssuanceError> {
        info!("Issuance cancelled / rejected");
        let reject_result = {
            let Some(Session::Issuance(session)) = self.session.as_ref() else {
                return Err(IssuanceError::SessionState);
            };

            self.storage
                .write()
                .await
                .delete_data::<PersistedIssuanceSessionData<<CID::Authorization as AuthorizationSession>::Persisted>>()
                .await
                .map_err(IssuanceError::SessionStorage)?;

            if let WalletIssuanceSession::Issuance { protocol_state, .. } = session {
                let organization = protocol_state.issuer_registration().organization.clone();
                info!("Rejecting issuance");
                protocol_state
                    .reject_issuance()
                    .await
                    .map_err(|error| IssuanceError::IssuerServer { organization, error })
            } else {
                Ok(())
            }
        };
        self.session = None;
        reject_result
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn continue_pid_issuance(
        &mut self,
        redirect_uri: Url,
    ) -> Result<Vec<AttestationPresentation>, IssuanceError> {
        info!("Received redirect URI, processing URI and retrieving access token");

        self.check_session_preconditions()?;

        info!("Checking if there is an active OAuth issuance session");
        if !matches!(
            self.session,
            Some(Session::Issuance(WalletIssuanceSession::OAuth { .. }))
        ) {
            return Err(IssuanceError::SessionState);
        }

        self.storage
            .write()
            .await
            .delete_data::<PersistedIssuanceSessionData<<CID::Authorization as AuthorizationSession>::Persisted>>()
            .await
            .map_err(IssuanceError::SessionStorage)?;

        let Some(Session::Issuance(WalletIssuanceSession::OAuth {
            authorization_session,
            purpose,
        })) = self.session.take()
        else {
            return Err(IssuanceError::SessionState);
        };

        let config = self.config_repository.get();
        let trust_anchors = config.issuer_trust_anchors();

        let issuance_session = authorization_session
            .start_issuance(&redirect_uri, trust_anchors)
            .await
            .map_err(|e| match e {
                WalletIssuanceError::OAuth(OAuthError::Denied) => IssuanceError::AuthorizationDenied,
                e => IssuanceError::IssuanceSession(e),
            })?;

        self.issuance_process_previews(issuance_session, Some(purpose)).await
    }

    #[instrument(skip_all)]
    pub(super) async fn issuance_process_previews(
        &mut self,
        issuance_session: CID::Issuance,
        pid_purpose: Option<PidIssuancePurpose>,
    ) -> Result<Vec<AttestationPresentation>, IssuanceError> {
        let previews = issuance_session.credential_previews();
        let preview_attestation_types = previews
            .iter()
            .map(|preview| preview.credential_payload.attestation_type.clone())
            .collect();
        let type_metadata = issuance_session.type_metadata();

        let config = self.config_repository.get();
        if pid_purpose.is_some() {
            self.compare_recovery_code_against_stored(
                Self::pid_preview(previews, &config.pid_attributes)?,
                &config.pid_attributes,
            )
            .await?;
        }

        let stored = self
            .storage
            .read()
            .await
            .fetch_unique_attestations_by_types(&preview_attestation_types)
            .await
            .map_err(IssuanceError::AttestationQuery)?;

        // For every preview, try to find the first matching stored attestation to determine its database identity. If
        // there are more candidates, the algorithm matches the first one based on the ascending order of the Uuidv7 of
        // the list of stored attestations. This means the oldest attestation is matched first.
        let previews_and_identity: Vec<(&CredentialPreview, Option<Uuid>)> = match_preview_and_stored_attestations(
            previews,
            stored,
            &TimeGenerator,
            pid_purpose.is_some().then_some(&config.pid_attributes),
        );

        info!("successfully received token and previews from issuer");
        let organization = &issuance_session.issuer_registration().organization;
        let attestations = previews_and_identity
            .into_iter()
            .map(|(preview_data, identity)| {
                let normalized_metadata = &type_metadata
                    .get(&preview_data.config_id)
                    .map(Ok)
                    .unwrap_or_else(|| Err(IssuanceError::MissingTypeMetadata(preview_data.config_id.clone())))?
                    .normalized_metadata;

                let attestation = AttestationPresentation::create_from_attributes(
                    identity.map_or(AttestationIdentity::Ephemeral, |id| AttestationIdentity::Fixed { id }),
                    preview_data.format,
                    normalized_metadata.clone(),
                    organization.clone(),
                    AttestationValidity {
                        revocation_status: None,
                        validity_window: ValidityWindow {
                            valid_until: preview_data.credential_payload.expires.map(Into::into),
                            valid_from: preview_data.credential_payload.not_before.map(Into::into),
                        },
                    },
                    &preview_data.credential_payload.attributes,
                    &config.pid_attributes,
                )
                .map_err(|error| IssuanceError::Attestation {
                    organization: organization.clone(),
                    error,
                })?;

                Ok(attestation)
            })
            .collect::<Result<Vec<_>, IssuanceError>>()?;

        // The IssuanceSession trait guarantees that credential_preview_data()
        // returns at least one value, so this unwrap() is safe.
        let event_attestations = attestations.clone().try_into().unwrap();
        self.session.replace(Session::Issuance(WalletIssuanceSession::Issuance {
            pid_purpose,
            preview_attestations: event_attestations,
            protocol_state: issuance_session,
        }));

        Ok(attestations)
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn accept_issuance(&mut self, pin: String) -> Result<IssuanceResult, IssuanceError>
    where
        UR: UpdateableRepository<VersionState, TlsPinningConfig, Error = UpdatePolicyError>,
    {
        info!("Accepting issuance");

        let (attested_key, registration_data, config) = self.check_accept_session_preconditions().await?;

        // Prepare the `RemoteEcdsaWscd` for signing using the provided PIN.
        let remote_instruction_client = self
            .prepare_remote_instruction_client(pin, (attested_key, registration_data, Arc::clone(&config)))
            .await?;

        let remote_wscd = RemoteEcdsaWscd::new(remote_instruction_client.clone());

        info!("Checking if there is an active issuance session");
        let Some(Session::Issuance(WalletIssuanceSession::Issuance {
            protocol_state,
            pid_purpose,
            ..
        })) = &mut self.session
        else {
            return Err(IssuanceError::SessionState);
        };

        info!("Signing nonce using Wallet Provider");
        let issuance_result = protocol_state
            .accept_issuance(config.issuer_trust_anchors(), &remote_wscd, pid_purpose.is_some())
            .await
            .map_err(|error| Self::handle_accept_issuance_error(error, protocol_state));

        // In some cases, the contents of the wallet need to be wiped and the wallet returned to its initial state.
        let issued_credentials_with_metadata = match issuance_result {
            Err(error @ IssuanceError::Instruction(InstructionError::Timeout { .. } | InstructionError::Blocked)) => {
                if pid_purpose.is_some() {
                    self.reset_to_initial_state().await;
                }
                return Err(error);
            }
            Err(error @ IssuanceError::Instruction(InstructionError::AccountRevoked(data))) => {
                self.handle_wallet_revocation(data).await;
                return Err(error);
            }
            _ => issuance_result?,
        };

        let pid_purpose = *pid_purpose;

        info!("Issuance succeeded; removing issuance session state");
        let preview_attestations = match self.session.take() {
            Some(Session::Issuance(WalletIssuanceSession::Issuance {
                preview_attestations, ..
            })) => preview_attestations,
            _ => return Err(IssuanceError::SessionState),
        };

        let transfer_session_id = if pid_purpose.is_some() {
            info!("This is a PID issuance session, therefore disclosing recovery code");
            self.disclose_recovery_code(
                &config.pid_attributes,
                &remote_instruction_client,
                &issued_credentials_with_metadata,
            )
            .await?
            // If we're doing PID renewal as opposed to enrolling, we don't want to transfer.
            .filter(|_| pid_purpose == Some(PidIssuancePurpose::Enrollment))
        } else {
            None
        };

        if let Some(transfer_session_id) = transfer_session_id.as_ref() {
            self.storage
                .write()
                .await
                .insert_data(&TransferData {
                    transfer_session_id: *transfer_session_id,
                    key_data: None,
                })
                .await
                .map_err(IssuanceError::TransferDataStorage)?;
        }

        let all_previews = issued_credentials_with_metadata
            .into_iter()
            .zip_eq(preview_attestations)
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

        self.emit_attestations().await?;
        self.emit_notifications().await?;
        self.emit_recent_history().await?;

        Ok(IssuanceResult { transfer_session_id })
    }

    pub(super) fn handle_accept_issuance_error(
        error: WalletIssuanceError,
        issuance_session: &CID::Issuance,
    ) -> IssuanceError {
        match error {
            // We knowingly call unwrap() on the downcast to `RemoteEcdsaKeyError` here because we know
            // that it is the error type of the `RemoteEcdsaWscd` we provide above.
            WalletIssuanceError::PrivateKeyGeneration(error) | WalletIssuanceError::Jwt(JwtError::Signing(error)) => {
                match *error.downcast::<RemoteEcdsaKeyError>().unwrap() {
                    RemoteEcdsaKeyError::Instruction(error) => IssuanceError::Instruction(error),
                    RemoteEcdsaKeyError::Signature(error) => IssuanceError::Signature(error),
                    RemoteEcdsaKeyError::KeyNotFound(identifier) => IssuanceError::KeyNotFound(identifier),
                    RemoteEcdsaKeyError::MissingSignature => IssuanceError::MissingSignature,
                }
            }
            _ => IssuanceError::IssuerServer {
                organization: issuance_session.issuer_registration().organization.clone(),
                error,
            },
        }
    }

    /// Finds the PID SD JWT, creates a disclosure of just the recovery code, and sends it to the remote instruction
    /// endpoint of the Wallet Provider.
    async fn disclose_recovery_code<AK: AppleAttestedKey, GK: GoogleAttestedKey>(
        &mut self,
        pid_attributes: &PidAttributesConfiguration,
        instruction_client: &InstructionClient<S, AK, GK, APC>,
        issued_credentials_with_metadata: &[CredentialWithMetadata],
    ) -> Result<Option<TransferSessionId>, IssuanceError> {
        let (pid, pid_paths) = issued_credentials_with_metadata
            .iter()
            .find_map(|cred| {
                let pid_paths = pid_attributes.sd_jwt.get(&cred.attestation_type)?;

                let sd_jwt = cred.copies.as_ref().iter().find_map(|copy| match copy {
                    IssuedCredential::MsoMdoc { .. } => None,
                    IssuedCredential::SdJwt { sd_jwt, .. } => Some(sd_jwt),
                })?;

                Some((sd_jwt, pid_paths))
            })
            .ok_or(IssuanceError::MissingPidSdJwt)?;

        let claim_path = pid_paths
            .recovery_code
            .nonempty_iter()
            .map(|path| ClaimPath::SelectByKey(path.clone()))
            .collect();

        let recovery_code_disclosure = pid
            .clone()
            .into_presentation_builder()
            .disclose(&claim_path)
            .map_err(IssuanceError::RecoveryCodeDisclosure)?
            .finish();

        let result = instruction_client
            .send(DiscloseRecoveryCode {
                recovery_code_disclosure: recovery_code_disclosure.into(),
                app_version: version().clone(),
            })
            .await;
        let result = self.check_result_for_wallet_revocation(result).await?;

        Ok(result.transfer_session_id.map(Into::into))
    }
}

fn match_preview_and_stored_attestations<'a>(
    previews: &'a [CredentialPreview],
    stored_attestations: Vec<StoredAttestationCopy>,
    time_generator: &impl Generator<DateTime<Utc>>,
    pid_config: Option<&PidAttributesConfiguration>,
) -> Vec<(&'a CredentialPreview, Option<Uuid>)> {
    let mut stored_credential_payloads = stored_attestations
        .into_iter()
        .map(|copy| {
            let attestation_id = copy.attestation_id();
            let format = copy.format();

            (attestation_id, (format, copy.into_previewable_credential_payload()))
        })
        .collect::<HashMap<_, _>>();

    // Find the first matching stored preview based on the ordering of `stored_credential_payloads`.
    previews
        .iter()
        .map(|preview| {
            let identity = stored_credential_payloads
                .iter()
                .find_map(|(id, (format, stored_preview))| {
                    // The new credential is not a renewal of an existing credential if their formats differ.
                    if *format != preview.format {
                        return None;
                    }

                    pid_config
                        .map_or_else(
                            // If this is not PID issuance, then the two cards match if their contents are identical.
                            || compare_contents(preview, stored_preview, time_generator),
                            // If this is PID issuance, and the two cards are both PIDs, then they match.
                            // If not, fall back to contents comparison.
                            |pid_config| {
                                let pid_types = pid_config.pid_attestation_types().collect_vec();
                                let both_pid = pid_types
                                    .contains(&preview.credential_payload.attestation_type.as_str())
                                    && pid_types.contains(&stored_preview.attestation_type.as_str());
                                both_pid || compare_contents(preview, stored_preview, time_generator)
                            },
                        )
                        .then_some(*id)
                });

            // Remove the stored credential from being considered in future iterations, as a single credential cannot be
            // renewed by multiple incoming credentials.
            if let Some(identity) = identity {
                stored_credential_payloads.remove(&identity);
            }

            (preview, identity)
        })
        .collect()
}

fn compare_contents(
    preview: &CredentialPreview,
    stored_preview: &PreviewableCredentialPayload,
    time_generator: &impl Generator<DateTime<Utc>>,
) -> bool {
    preview
        .credential_payload
        .matches_existing(stored_preview, time_generator)
}

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use std::collections::HashMap;
    use std::ops::Add;

    use attestation_data::attributes::AttributeValue;
    use attestation_data::auth::issuer_auth::IssuerRegistration;
    use attestation_data::validity::ValidityWindow;
    use attestation_data::x509::CertificateType;
    use attestation_types::credential_format::Format;
    use attestation_types::pid_constants::PID_ATTESTATION_TYPE;
    use chrono::Duration;
    use crypto::server_keys::generate::Ca;
    use futures::FutureExt;
    use itertools::multiunzip;
    use mockall::predicate::*;
    use openid4vc::wallet_issuance::credential::IssuedCredential;
    use openid4vc::wallet_issuance::mock::MockAuthorizationSession;
    use openid4vc::wallet_issuance::mock::MockAuthorizationSessionData;
    use openid4vc::wallet_issuance::mock::MockIssuanceSession;
    use p256::ecdsa::SigningKey;
    use rstest::rstest;
    use sd_jwt_vc_metadata::NormalizedTypeMetadata;
    use sd_jwt_vc_metadata::VerifiedTypeMetadataDocuments;
    use url::Url;
    use utils::generator::mock::MockTimeGenerator;
    use utils::vec_nonempty;
    use uuid::Uuid;
    use wallet_account::messages::errors::AccountRevokedData;
    use wallet_account::messages::errors::RevocationReason;
    use wallet_account::messages::instructions::DiscloseRecoveryCodeResult;
    use wallet_account::messages::instructions::Instruction;
    use wallet_configuration::wallet_config::PidAttributePaths;

    use super::super::test;
    use super::super::test::TestWalletMockStorage;
    use super::super::test::WalletDeviceVendor;
    use super::*;
    use crate::WalletEvent;
    use crate::attestation::AttestationAttributeValue;
    use crate::storage::ChangePinData;
    use crate::storage::InstructionData;
    use crate::storage::RegistrationData;
    use crate::storage::StorageState;
    use crate::storage::StoredAttestation;
    use crate::wallet::state::CancelSessionError;
    use crate::wallet::test::AUTH_URL;
    use crate::wallet::test::create_example_credential_payload;
    use crate::wallet::test::create_example_pid_credential_payload;
    use crate::wallet::test::create_example_pid_mdoc;
    use crate::wallet::test::create_example_pid_preview_data;
    use crate::wallet::test::create_example_pid_sd_jwt;
    use crate::wallet::test::create_preview_from_payload;
    use crate::wallet::test::create_wp_result;
    use crate::wallet::test::mock_issuance_session;

    #[rstest]
    #[case(PidIssuancePurpose::Enrollment, false)]
    #[case(PidIssuancePurpose::Renewal, true)]
    #[tokio::test]
    async fn test_create_pid_issuance_auth_url(#[case] purpose: PidIssuancePurpose, #[case] pid_present: bool) {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        assert!(wallet.session.is_none());

        // Set up the credential issuer discovery mock
        wallet
            .issuance_discovery
            .expect_start_authorization_code_flow_sync()
            .return_once(|| {
                let mut authorization_session = MockAuthorizationSession::new();

                authorization_session
                    .expect_get_auth_url()
                    .return_const(Url::parse(AUTH_URL).unwrap());
                authorization_session
                    .expect_get_state()
                    .return_const("state".to_string());
                Ok(authorization_session)
            });

        wallet
            .mut_storage()
            .expect_has_any_attestations_with_types()
            .return_once(move |_| Ok(pid_present));
        wallet
            .mut_storage()
            .expect_upsert_data::<PersistedIssuanceSessionData<MockAuthorizationSessionData>>()
            .return_once(move |data| {
                assert_eq!(data.purpose, purpose);
                assert_eq!(data.authorization_session.auth_url.as_str(), AUTH_URL);
                assert_eq!(data.authorization_session.state, "state");
                Ok(())
            });

        // Have the `Wallet` generate an OAuth URL and test it.
        let auth_url = wallet
            .create_pid_issuance_auth_url(purpose)
            .await
            .expect("Could not generate PID issuance auth URL");

        assert!(auth_url.as_str().starts_with(AUTH_URL));
        assert!(wallet.session.is_some());
    }

    #[rstest]
    #[case(PidIssuancePurpose::Enrollment, true)]
    #[case(PidIssuancePurpose::Renewal, false)]
    #[tokio::test]
    async fn test_create_pid_issuance_auth_url_pid_mismatch(
        #[case] purpose: PidIssuancePurpose,
        #[case] pid_present: bool,
    ) {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        wallet
            .mut_storage()
            .expect_has_any_attestations_with_types()
            .return_once(move |_| Ok(pid_present));

        // Enrolling when we already have a PID, or renewing when we don't have a PID, should result in an error.
        wallet
            .create_pid_issuance_auth_url(purpose)
            .await
            .expect_err("Generating auth URL for the wrong purpose should have failed");
    }

    #[rstest]
    #[tokio::test]
    async fn test_create_pid_issuance_auth_url_error_locked(
        #[values(PidIssuancePurpose::Enrollment, PidIssuancePurpose::Renewal)] purpose: PidIssuancePurpose,
    ) {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        wallet.lock();

        // Creating an OAuth URL on
        // a locked wallet should result in an error.
        let error = wallet
            .create_pid_issuance_auth_url(purpose)
            .await
            .expect_err("PID issuance auth URL generation should have resulted in error");

        assert_matches!(
            error,
            IssuanceError::CheckPreconditions(CheckPreconditionsError::Locked)
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_create_pid_issuance_auth_url_error_unregistered(
        #[values(PidIssuancePurpose::Enrollment, PidIssuancePurpose::Renewal)] purpose: PidIssuancePurpose,
    ) {
        // Prepare an unregistered wallet.
        let mut wallet = TestWalletMockStorage::new_unregistered(WalletDeviceVendor::Apple).await;

        // Creating an OAuth URL on an
        // unregistered wallet should result in an error.
        let error = wallet
            .create_pid_issuance_auth_url(purpose)
            .await
            .expect_err("PID issuance auth URL generation should have resulted in error");

        assert_matches!(
            error,
            IssuanceError::CheckPreconditions(CheckPreconditionsError::NotRegistered)
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_create_pid_issuance_auth_url_error_session_state_oauth(
        #[values(PidIssuancePurpose::Enrollment, PidIssuancePurpose::Renewal)] purpose: PidIssuancePurpose,
    ) {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        // Set up a mock OAuth session.
        wallet.session = Some(Session::Issuance(WalletIssuanceSession::OAuth {
            purpose: PidIssuancePurpose::Enrollment,
            authorization_session: MockAuthorizationSession::new(),
        }));
        wallet
            .mut_storage()
            .expect_delete_data::<PersistedIssuanceSessionData<MockAuthorizationSessionData>>()
            .return_once(|| Ok(()));

        // Creating an OAuth URL on a `Wallet` that
        // has an active OAuth session should return an error.
        let error = wallet
            .create_pid_issuance_auth_url(purpose)
            .await
            .expect_err("PID issuance auth URL generation should have resulted in error");

        assert_matches!(error, IssuanceError::SessionState);
    }

    #[rstest]
    #[tokio::test]
    async fn test_create_pid_issuance_auth_url_error_session_state_pid_issuer(
        #[values(PidIssuancePurpose::Enrollment, PidIssuancePurpose::Renewal)] purpose: PidIssuancePurpose,
    ) {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        // Setup a mock OpenID4VCI session.
        wallet.session = Some(Session::Issuance(WalletIssuanceSession::Issuance {
            pid_purpose: Some(purpose),
            preview_attestations: vec_nonempty![AttestationPresentation::new_mock()],
            protocol_state: MockIssuanceSession::default(),
        }));

        // Creating an OAuth URL on a `Wallet` that has
        // an active OpenID4VCI session should return an error.
        let error = wallet
            .create_pid_issuance_auth_url(purpose)
            .await
            .expect_err("PID issuance auth URL generation should have resulted in error");

        assert_matches!(error, IssuanceError::SessionState);
    }

    #[tokio::test]
    async fn test_cancel_pid_issuance_oauth() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        // Set up a mock OAuth session.
        wallet.session = Some(Session::Issuance(WalletIssuanceSession::OAuth {
            purpose: PidIssuancePurpose::Enrollment,
            authorization_session: MockAuthorizationSession::new(),
        }));
        wallet
            .mut_storage()
            .expect_delete_data::<PersistedIssuanceSessionData<MockAuthorizationSessionData>>()
            .return_once(|| Ok(()));

        assert!(wallet.session.is_some());

        // Cancelling PID issuance should clear this session.
        wallet.cancel_session().await.expect("Could not cancel PID issuance");

        assert!(wallet.session.is_none());
    }

    #[tokio::test]
    async fn test_cancel_pid_issuance_delete_persisted_session_error_keeps_session() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        wallet.session = Some(Session::Issuance(WalletIssuanceSession::OAuth {
            purpose: PidIssuancePurpose::Enrollment,
            authorization_session: MockAuthorizationSession::new(),
        }));
        wallet
            .mut_storage()
            .expect_delete_data::<PersistedIssuanceSessionData<MockAuthorizationSessionData>>()
            .return_once(|| Err(StorageError::NotOpened));

        let error = wallet
            .cancel_session()
            .await
            .expect_err("Cancelling PID issuance should have resulted in an error");

        assert_matches!(
            error,
            CancelSessionError::Issuance(IssuanceError::SessionStorage(StorageError::NotOpened))
        );
        assert_matches!(
            wallet.session,
            Some(Session::Issuance(WalletIssuanceSession::OAuth {
                purpose: PidIssuancePurpose::Enrollment,
                ..
            }))
        );
    }

    #[tokio::test]
    async fn test_cancel_pid_issuance_pid() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        // Set up the `PidIssuerClient`
        let pid_issuer = {
            let mut client = MockIssuanceSession::new();
            client.expect_reject().return_once(|| Ok(()));
            client.expect_issuer().return_const(IssuerRegistration::new_mock());
            client
        };
        wallet.session = Some(Session::Issuance(WalletIssuanceSession::Issuance {
            pid_purpose: Some(PidIssuancePurpose::Enrollment),
            preview_attestations: vec_nonempty![AttestationPresentation::new_mock()],
            protocol_state: pid_issuer,
        }));
        wallet
            .mut_storage()
            .expect_delete_data::<PersistedIssuanceSessionData<MockAuthorizationSessionData>>()
            .return_once(|| Ok(()));

        // Cancelling PID issuance should not fail.
        wallet.cancel_session().await.expect("Could not cancel PID issuance");

        assert!(wallet.session.is_none());
    }

    #[tokio::test]
    async fn test_cancel_pid_issuance_error_locked() {
        // Prepare a registered and locked wallet.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        wallet.lock();

        // Cancelling PID issuance on a locked wallet should result in an error.
        let error = wallet
            .cancel_session()
            .await
            .expect_err("Cancelling PID issuance should have resulted in an error");

        assert_matches!(
            error,
            CancelSessionError::Preconditions(CheckPreconditionsError::Locked)
        );
    }

    #[tokio::test]
    async fn test_cancel_pid_issuance_error_unregistered() {
        // Prepare an unregistered wallet.
        let mut wallet = TestWalletMockStorage::new_unregistered(WalletDeviceVendor::Apple).await;

        // Cancelling PID issuance on an unregistered wallet should result in an error.
        let error = wallet
            .cancel_session()
            .await
            .expect_err("Cancelling PID issuance should have resulted in an error");

        assert_matches!(
            error,
            CancelSessionError::Preconditions(CheckPreconditionsError::NotRegistered)
        );
    }

    #[tokio::test]
    async fn test_cancel_pid_issuance_error_session_state() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        // Cancelling PID issuance on a wallet with no
        // active OAuth session should result in an error.
        let error = wallet
            .cancel_session()
            .await
            .expect_err("Cancelling PID issuance should have resulted in an error");

        assert_matches!(error, CancelSessionError::SessionState);
    }

    const REDIRECT_URI: &str = "redirect://here";

    #[tokio::test]
    async fn test_continue_pid_issuance() {
        // `setup_wallet_with_oauth_session_and_database_mock` already sets up the mock `start` expectation.
        let (mut wallet, redirect_uri) = setup_wallet_with_oauth_session_and_database_mock().await;

        let stored = {
            let (sd_jwt, metadata) = create_example_pid_sd_jwt();
            StoredAttestationCopy::new(
                Uuid::new_v4(),
                Uuid::new_v4(),
                ValidityWindow::new_valid_mock(),
                StoredAttestation::SdJwt {
                    key_identifier: "key".to_string(),
                    sd_jwt,
                },
                metadata,
                None,
            )
        };
        let stored_clone = stored.clone();

        wallet
            .mut_storage()
            .expect_fetch_unique_attestations_by_types()
            .return_once(move |_| Ok(vec![stored]));

        wallet
            .mut_storage()
            .expect_fetch_unique_attestations_by_types_and_format()
            .return_once(move |_, _| Ok(vec![stored_clone]));

        // Continuing PID issuance should result in one preview `Attestation`.
        let attestations = wallet
            .continue_pid_issuance(redirect_uri)
            .await
            .expect("Could not continue PID issuance");

        assert_eq!(attestations.len(), 1);

        let attestation = attestations.into_iter().next().unwrap();

        // A new PID always overwrites an older PID
        assert_matches!(attestation.identity, AttestationIdentity::Fixed { .. });
        assert_eq!(attestation.attributes.len(), 4);
        assert_eq!(attestation.attributes[0].key, vec_nonempty!["family_name".to_string()]);
        assert_matches!(
            &attestation.attributes[0].value,
            AttestationAttributeValue::Basic(AttributeValue::Text(string)) if string == "De Bruijn"
        );
    }

    #[tokio::test]
    async fn test_continue_pid_issuance_user_cancelled() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let mut authorization_session = MockAuthorizationSession::new();
        authorization_session
            .expect_start_issuance_sync()
            .return_once(|| Err(WalletIssuanceError::OAuth(OAuthError::Denied)));
        wallet.session = Some(Session::Issuance(WalletIssuanceSession::OAuth {
            purpose: PidIssuancePurpose::Enrollment,
            authorization_session,
        }));
        wallet
            .mut_storage()
            .expect_delete_data::<PersistedIssuanceSessionData<MockAuthorizationSessionData>>()
            .return_once(|| Ok(()));

        let denied_redirect = Url::parse(&(REDIRECT_URI.to_string() + "?error=access_denied&state=whatever")).unwrap();
        let error = wallet
            .continue_pid_issuance(denied_redirect)
            .await
            .expect_err("Continuing PID issuance should have resulted in error");

        assert_matches!(error, IssuanceError::AuthorizationDenied);
    }

    #[tokio::test]
    async fn test_continue_pid_issuance_delete_persisted_session_error_keeps_session() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        wallet.session = Some(Session::Issuance(WalletIssuanceSession::OAuth {
            purpose: PidIssuancePurpose::Enrollment,
            authorization_session: MockAuthorizationSession::new(),
        }));
        wallet
            .mut_storage()
            .expect_delete_data::<PersistedIssuanceSessionData<MockAuthorizationSessionData>>()
            .return_once(|| Err(StorageError::NotOpened));

        let error = wallet
            .continue_pid_issuance(Url::parse(REDIRECT_URI).unwrap())
            .await
            .expect_err("Continuing PID issuance should have resulted in error");

        assert_matches!(error, IssuanceError::SessionStorage(StorageError::NotOpened));
        assert_matches!(
            wallet.session,
            Some(Session::Issuance(WalletIssuanceSession::OAuth {
                purpose: PidIssuancePurpose::Enrollment,
                ..
            }))
        );
    }

    #[tokio::test]
    async fn test_continue_pid_issuance_error_locked() {
        // Prepare a registered and locked wallet.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        wallet.lock();

        // Continuing PID issuance on a locked wallet should result in an error.
        let error = wallet
            .continue_pid_issuance(Url::parse(REDIRECT_URI).unwrap())
            .await
            .expect_err("Continuing PID issuance should have resulted in error");

        assert_matches!(
            error,
            IssuanceError::CheckPreconditions(CheckPreconditionsError::Locked)
        );
    }

    #[tokio::test]
    async fn test_continue_pid_issuance_error_unregistered() {
        // Prepare an unregistered wallet.
        let mut wallet = TestWalletMockStorage::new_unregistered(WalletDeviceVendor::Apple).await;

        // Continuing PID issuance on an unregistered wallet should result in an error.
        let error = wallet
            .continue_pid_issuance(Url::parse(REDIRECT_URI).unwrap())
            .await
            .expect_err("Continuing PID issuance should have resulted in error");

        assert_matches!(
            error,
            IssuanceError::CheckPreconditions(CheckPreconditionsError::NotRegistered)
        );
    }

    #[tokio::test]
    async fn test_continue_pid_issuance_error_session_state() {
        // Prepare a registered wallet.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        // Continuing PID issuance on a wallet with no active OAuth session should result in an error.
        let error = wallet
            .continue_pid_issuance(Url::parse(REDIRECT_URI).unwrap())
            .await
            .expect_err("Continuing PID issuance should have resulted in error");

        assert_matches!(error, IssuanceError::SessionState);
    }

    async fn setup_wallet_with_oauth_session_and_database_mock() -> (TestWalletMockStorage, Url) {
        // Prepare a registered wallet.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let redirect_uri = Url::parse(REDIRECT_URI).unwrap();
        let mut authorization_session = MockAuthorizationSession::new();
        authorization_session.expect_start_issuance_sync().return_once(|| {
            let mut session = MockIssuanceSession::new();
            let (preview, type_metadata) =
                create_example_pid_preview_data(&MockTimeGenerator::default(), Format::SdJwt);
            session
                .expect_type_metadata()
                .return_const([(preview.config_id.clone(), type_metadata)].into());
            session.expect_credential_previews().return_const(vec![preview]);
            session.expect_issuer().return_const(IssuerRegistration::new_mock());
            Ok(session)
        });
        wallet.session = Some(Session::Issuance(WalletIssuanceSession::OAuth {
            purpose: PidIssuancePurpose::Enrollment,
            authorization_session,
        }));
        wallet
            .mut_storage()
            .expect_delete_data::<PersistedIssuanceSessionData<MockAuthorizationSessionData>>()
            .return_once(|| Ok(()));
        (wallet, redirect_uri)
    }

    #[tokio::test]
    async fn test_continue_pid_issuance_error_pid_issuer() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let redirect_uri = Url::parse(REDIRECT_URI).unwrap();

        // Set up the authorization session to return an error from start_issuance.
        let mut authorization_session = MockAuthorizationSession::new();
        authorization_session
            .expect_start_issuance_sync()
            .return_once(|| Err(WalletIssuanceError::IssuerMismatch));

        wallet.session = Some(Session::Issuance(WalletIssuanceSession::OAuth {
            purpose: PidIssuancePurpose::Enrollment,
            authorization_session,
        }));
        wallet
            .mut_storage()
            .expect_delete_data::<PersistedIssuanceSessionData<MockAuthorizationSessionData>>()
            .return_once(|| Ok(()));

        // Continuing PID issuance on a wallet should forward this error.
        let error = wallet
            .continue_pid_issuance(redirect_uri)
            .await
            .expect_err("Continuing PID issuance should have resulted in error");

        assert_matches!(error, IssuanceError::IssuanceSession { .. });
    }

    #[rstest]
    // An attestation that is identical to one that the wallet already has should overwrite the old one.
    #[case::non_pid_identical(None, vec![Format::SdJwt], "some_attestation_type", Some(0), |preview| {preview})]
    // An attestation that has a different format but is otherwise identical should not overwrite the existing one.
    #[case::non_pid_identical_mdoc(None, vec![Format::MsoMdoc], "some_attestation_type", None, |preview| {preview})]
    // Given two identical previews, onle the first one should overwerite an existing credential.
    #[case::non_pid_multiple_previews(
        None, vec![Format::SdJwt, Format::SdJwt], "some_attestation_type", Some(0), |preview| {preview}
    )]
    // Only the existing SD-JWT credential should be overwritten.
    #[case::non_pid_multiple_formats1(
        None, vec![Format::SdJwt, Format::MsoMdoc], "some_attestation_type", Some(0), |preview| {preview})]
    #[case::non_pid_multiple_formats2(
        None, vec![Format::MsoMdoc, Format::SdJwt], "some_attestation_type", Some(1), |preview| {preview}
    )]
    // When the attestation_type is different from the one stored in the database, it should be considered as a new
    // attestation and the identity is None.
    #[case::non_pid_different(
        None, vec![Format::SdJwt], "some_attestation_type", None, |mut preview: CredentialPreview| {
            preview.credential_payload.attestation_type = String::from("some_other_attestation_type");
            preview
        }
    )]
    // When the attestation already exists in the database, but the preview has a newer nbf, it should be considered as
    // a new attestation and the identity is None.
    #[case::non_pid_newer_nbf(
        None, vec![Format::SdJwt], "some_attestation_type", None,|mut preview: CredentialPreview| {
            preview.credential_payload.not_before = Some(Utc::now().add(Duration::days(365)).into());
            preview
        }
    )]
    // A new PID always overwrites an older PID, even in cases where other attestation types would be overwritten.
    #[case::pid_enrollment(
        Some(PidIssuancePurpose::Enrollment), vec![Format::SdJwt], PID_ATTESTATION_TYPE, Some(0), |preview| {preview}
    )]
    #[case::pid_renewal(
        Some(PidIssuancePurpose::Renewal), vec![Format::SdJwt], PID_ATTESTATION_TYPE, Some(0), |preview| {preview}
    )]
    #[case::pid_renewal_newer_nbf(
        Some(PidIssuancePurpose::Renewal),
        vec![Format::SdJwt],
        PID_ATTESTATION_TYPE,
        Some(0),
        |mut preview : CredentialPreview| {
            preview.credential_payload.not_before = Some(Utc::now().add(Duration::days(365)).into());
            preview
        }
    )]
    // However, the PID should not override a PID credential with the same attestation type but a different format.
    #[case::pid_renewal_multiple_formats1(
        Some(PidIssuancePurpose::Renewal),
        vec![Format::SdJwt, Format::MsoMdoc],
        PID_ATTESTATION_TYPE,
        Some(0),
        |preview| {preview}
    )]
    #[case::pid_renewal_multiple_formats2(
        Some(PidIssuancePurpose::Renewal),
        vec![Format::MsoMdoc, Format::SdJwt],
        PID_ATTESTATION_TYPE,
        Some(1),
        |preview| {preview}
    )]
    #[tokio::test]
    async fn test_continue_issuance_with_renewed_attestation(
        #[case] purpose: Option<PidIssuancePurpose>,
        #[case] preview_formats: Vec<Format>,
        #[case] attestation_type: &str,
        #[case] expect_fixed_index: Option<usize>,
        #[case] modifier: impl Fn(CredentialPreview) -> CredentialPreview,
    ) {
        // Prepare a registered wallet.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let time_generator = MockTimeGenerator::default();

        let (payload, type_metadata) = create_example_credential_payload(&time_generator, attestation_type);
        let preview_count = preview_formats.len();
        let previews = std::iter::repeat_n(payload, preview_count)
            .zip_eq(preview_formats.into_iter())
            .map(move |(payload, format)| {
                modifier(create_preview_from_payload(payload, format, format.to_string().into()))
            })
            .collect_vec();

        let ca = Ca::generate("myca", Default::default()).unwrap();
        let cert_type = CertificateType::from(IssuerRegistration::new_mock());
        let issuer_key_pair = ca.generate_key_pair("mycert", cert_type, Default::default()).unwrap();

        let (payload, stored_type_metadata) = create_example_credential_payload(&time_generator, attestation_type);
        let sd_jwt = payload
            .into_signed_sd_jwt(&type_metadata.normalized_metadata, &issuer_key_pair)
            .now_or_never()
            .unwrap()
            .unwrap();

        let attestation_id = Uuid::new_v4();
        let stored = StoredAttestationCopy::new(
            attestation_id,
            Uuid::new_v4(),
            ValidityWindow::new_valid_mock(),
            StoredAttestation::SdJwt {
                key_identifier: "sd_jwt_key_identifier".to_string(),
                sd_jwt: sd_jwt.into_verified(),
            },
            stored_type_metadata.normalized_metadata.clone(),
            None,
        );

        let stored_clone = stored.clone();
        let attestation_id = stored.attestation_id();

        let storage = wallet.mut_storage();
        storage
            .expect_fetch_unique_attestations_by_types()
            .return_once(move |_attestation_types| Ok(vec![stored]));
        storage
            .expect_fetch_unique_attestations_by_types_and_format()
            .return_once(move |_attestation_types, _format| Ok(vec![stored_clone]));

        // Set up the `MockIssuanceSession` directly.
        let mut issuance_session = MockIssuanceSession::new();
        issuance_session.expect_type_metadata().return_const(
            previews
                .iter()
                .map(|preview| preview.config_id.clone())
                .zip_eq(std::iter::repeat_n(type_metadata, preview_count))
                .collect(),
        );
        issuance_session.expect_credential_previews().return_const(previews);
        issuance_session
            .expect_issuer()
            .return_const(IssuerRegistration::new_mock());

        let attestations = wallet
            .issuance_process_previews(issuance_session, purpose)
            .await
            .expect("Could not continue issuance");

        assert_eq!(attestations.len(), preview_count);

        for (index, attestation) in attestations.iter().enumerate() {
            if expect_fixed_index.is_some_and(|expected_index| expected_index == index) {
                assert_matches!(&attestation.identity, AttestationIdentity::Fixed { id } if *id == attestation_id);
            } else {
                assert_matches!(&attestation.identity, AttestationIdentity::Ephemeral);
            }
        }
    }

    #[tokio::test]
    async fn test_cancel_pid_issuance_error_pid_issuer() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        // Set up a mock OpenID4VCI session that expects to be rejected, which returns an error.
        let pid_issuer = {
            let mut client = MockIssuanceSession::new();
            client
                .expect_reject()
                .return_once(|| Err(WalletIssuanceError::IssuerMismatch));

            client.expect_issuer().return_const(IssuerRegistration::new_mock());

            client
        };
        wallet.session = Some(Session::Issuance(WalletIssuanceSession::Issuance {
            pid_purpose: Some(PidIssuancePurpose::Enrollment),
            preview_attestations: vec_nonempty![AttestationPresentation::new_mock()],
            protocol_state: pid_issuer,
        }));
        wallet
            .mut_storage()
            .expect_delete_data::<PersistedIssuanceSessionData<MockAuthorizationSessionData>>()
            .return_once(|| Ok(()));

        // Canceling PID issuance on a wallet should forward this error.
        let error = wallet
            .cancel_session()
            .await
            .expect_err("Rejecting PID issuance should have resulted in an error");

        assert_matches!(error, CancelSessionError::Issuance(IssuanceError::IssuerServer { .. }));
        assert_matches!(wallet.session, None);
    }

    const PIN: &str = "051097";

    fn sd_jwt_pid() -> (IssuedCredential, VerifiedTypeMetadataDocuments, NormalizedTypeMetadata) {
        let (sd_jwt, normalized_metadata) = create_example_pid_sd_jwt();
        let credential = IssuedCredential::SdJwt {
            key_identifier: "key_id".to_string(),
            sd_jwt: sd_jwt.clone(),
        };
        let metadata_docs = VerifiedTypeMetadataDocuments::nl_pid_example();

        (credential, metadata_docs, normalized_metadata)
    }

    fn mdoc_pid() -> (IssuedCredential, VerifiedTypeMetadataDocuments, NormalizedTypeMetadata) {
        let (mdoc, normalized_metadata) = create_example_pid_mdoc(&SigningKey::random(&mut rand::thread_rng()));
        let credential = IssuedCredential::MsoMdoc { mdoc };
        let metadata_docs = VerifiedTypeMetadataDocuments::nl_pid_example();

        (credential, metadata_docs, normalized_metadata)
    }

    #[rstest]
    #[case([sd_jwt_pid()])]
    #[case([sd_jwt_pid(), mdoc_pid()])]
    #[case([mdoc_pid(), sd_jwt_pid()])]
    // Receiving the PID twice in SD-JWT is unrealistic, but let's test it anyway.
    #[case([sd_jwt_pid(), sd_jwt_pid()])]
    #[tokio::test]
    async fn test_accept_pid_issuance(
        #[case] pid_credentials: impl IntoIterator<
            Item = (IssuedCredential, VerifiedTypeMetadataDocuments, NormalizedTypeMetadata),
        >,
    ) {
        // Prepare a registered and unlocked wallet.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        wallet
            .mut_storage()
            .expect_fetch_unique_attestations()
            .return_once(|| Ok(vec![]));
        // Register mock document_callback
        let attestations_callback = test::setup_mock_attestations_callback(&mut wallet).await.unwrap();

        wallet
            .mut_storage()
            .expect_fetch_recent_wallet_events()
            .return_once(|| Ok(vec![]));
        // Register mock recent_history_callback
        let events = test::setup_mock_recent_history_callback(&mut wallet).await.unwrap();
        wallet.mut_storage().checkpoint();

        let (issuer_credentials, stored_copies) = pid_credentials
            .into_iter()
            .map(|(credential, metadata_docs, normalized_metadata)| {
                let stored_attestation = match credential.clone() {
                    IssuedCredential::MsoMdoc { mdoc } => StoredAttestation::MsoMdoc { mdoc },
                    IssuedCredential::SdJwt { key_identifier, sd_jwt } => {
                        StoredAttestation::SdJwt { key_identifier, sd_jwt }
                    }
                };
                let stored_copy = StoredAttestationCopy::new(
                    Uuid::new_v4(),
                    Uuid::new_v4(),
                    ValidityWindow::new_valid_mock(),
                    stored_attestation,
                    normalized_metadata,
                    None,
                );

                ((credential, metadata_docs), stored_copy)
            })
            .unzip::<_, _, Vec<_>, Vec<_>>();

        let credential_count = issuer_credentials.len();
        let (pid_issuer, attestations) = mock_issuance_session(issuer_credentials);
        wallet.session = Some(Session::Issuance(WalletIssuanceSession::Issuance {
            pid_purpose: Some(PidIssuancePurpose::Enrollment),
            preview_attestations: attestations,
            protocol_state: pid_issuer,
        }));

        wallet
            .mut_storage()
            .expect_fetch_unique_attestations()
            .times(1)
            .return_once(|| Ok(stored_copies));

        wallet
            .mut_storage()
            .expect_fetch_data::<ChangePinData>()
            .returning(|| Ok(None));

        wallet
            .mut_storage()
            .expect_insert_credentials()
            .withf(|_, _| true)
            .returning(|_, _| Ok(()));

        wallet
            .mut_storage()
            .expect_fetch_data::<InstructionData>()
            .returning(|| {
                Ok(Some(InstructionData {
                    instruction_sequence_number: 0,
                }))
            });

        wallet
            .mut_storage()
            .expect_upsert_data::<InstructionData>()
            .returning(|_| Ok(()));

        wallet
            .mut_storage()
            .expect_fetch_recent_wallet_events()
            .times(1)
            .returning(|| {
                Ok(vec![WalletEvent::Issuance {
                    id: Uuid::new_v4(),
                    attestation: Box::new(AttestationPresentation::new_mock()),
                    timestamp: Utc::now(),
                    renewed: false,
                }])
            });

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

            // The second entry should contain a the PID attestations.
            assert_eq!(attestations[1].len(), credential_count);
            for attestation in &attestations[1] {
                assert_matches!(attestation.identity, AttestationIdentity::Fixed { id: _ });
                assert_eq!(attestation.attestation_type, PID_ATTESTATION_TYPE);
            }

            // Test that one successful issuance event is logged
            let events = events.lock();
            assert_eq!(events.len(), 2);
            assert!(events[0].is_empty());
            assert_eq!(events[1].len(), 1);
            assert_matches!(&events[1][0], WalletEvent::Issuance { .. });

            assert!(wallet.has_registration());
            assert!(!wallet.is_locked());
        }

        wallet
            .mut_storage()
            .expect_has_any_attestations_with_types()
            .return_once(|_| Ok(true));

        let err = wallet
            .create_pid_issuance_auth_url(PidIssuancePurpose::Enrollment)
            .await
            .expect_err("creating new PID issuance auth URL when there already is a PID should fail");
        assert_matches!(err, IssuanceError::PidAlreadyPresent);
    }

    fn setup_mock_recovery_code_instructions(wallet: &mut TestWalletMockStorage) {
        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_instruction_challenge()
            .once()
            .returning(|_, _| Ok(crypto::utils::random_bytes(32)));

        let wp_result = create_wp_result(DiscloseRecoveryCodeResult {
            transfer_session_id: None,
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
        let mut wallet = TestWalletMockStorage::new_unregistered(WalletDeviceVendor::Apple).await;

        // Accepting PID issuance on an unregistered wallet should result in an error.
        let error = wallet
            .accept_issuance(PIN.to_owned())
            .await
            .expect_err("Accepting PID issuance should have resulted in an error");

        assert_matches!(
            error,
            IssuanceError::CheckPreconditions(CheckPreconditionsError::NotRegistered)
        );
    }

    #[tokio::test]
    async fn test_accept_pid_issuance_locked() {
        // Prepare a registered and locked wallet.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        wallet.lock();

        // Accepting PID issuance on a locked wallet should result in an error.
        let error = wallet
            .accept_issuance(PIN.to_owned())
            .await
            .expect_err("Accepting PID issuance should have resulted in an error");

        assert_matches!(
            error,
            IssuanceError::CheckPreconditions(CheckPreconditionsError::Locked)
        );

        assert!(wallet.has_registration());
        assert!(wallet.is_locked());
    }

    #[tokio::test]
    async fn test_accept_pid_issuance_session_state() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        wallet
            .mut_storage()
            .expect_fetch_data::<ChangePinData>()
            .return_once(|| Ok(None));

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
        expect_reset: bool,
    ) -> (TestWalletMockStorage, IssuanceError) {
        // Prepare a registered and unlocked wallet.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        if expect_reset {
            wallet.mut_storage().checkpoint();

            wallet.mut_storage().expect_clear().return_const(());
            wallet
                .mut_storage()
                .expect_state()
                .returning(|| Ok(StorageState::Uninitialized));
            wallet
                .mut_storage()
                .expect_fetch_data::<RegistrationData>()
                .return_once(|| Ok(None));
        }

        wallet
            .mut_storage()
            .expect_fetch_data::<ChangePinData>()
            .returning(|| Ok(None));

        // Have the mock OpenID4VCI session return a particular `RemoteEcdsaKeyError` upon accepting.
        let pid_issuer = {
            let mut client = MockIssuanceSession::new();
            client
                .expect_accept()
                .return_once(|| Err(WalletIssuanceError::Jwt(JwtError::Signing(Box::new(key_error)))));

            client.expect_issuer().return_const(IssuerRegistration::new_mock());

            client
        };
        wallet.session = Some(Session::Issuance(WalletIssuanceSession::Issuance {
            pid_purpose: Some(PidIssuancePurpose::Enrollment),
            preview_attestations: vec_nonempty![AttestationPresentation::new_mock()],
            protocol_state: pid_issuer,
        }));

        // Accepting PID issuance should result in an error.
        let error = wallet
            .accept_issuance(PIN.to_owned())
            .await
            .expect_err("Accepting PID issuance should have resulted in an error");

        (wallet, error)
    }

    #[rstest]
    #[case(InstructionError::IncorrectPin{ attempts_left_in_round: 1, is_final_round: false }, false)]
    #[case(InstructionError::Timeout{ timeout_millis: 10_000 }, true)]
    #[case(InstructionError::Blocked, true)]
    #[case(InstructionError::InstructionValidation, false)]
    #[tokio::test]
    async fn test_accept_pid_issuance_error_instruction(
        #[case] instruction_error: InstructionError,
        #[case] expect_reset: bool,
    ) {
        let (wallet, error) =
            test_accept_pid_issuance_error_remote_key(RemoteEcdsaKeyError::from(instruction_error), expect_reset).await;

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
            test_accept_pid_issuance_error_remote_key(RemoteEcdsaKeyError::from(signature::Error::default()), false)
                .await;

        // Test that this error is converted to the appropriate variant of `IssuanceError`.
        assert_matches!(error, IssuanceError::Signature(_));

        assert!(wallet.has_registration());
        assert!(!wallet.is_locked());
    }

    #[tokio::test]
    async fn test_accept_pid_issuance_error_key_not_found() {
        let (wallet, error) =
            test_accept_pid_issuance_error_remote_key(RemoteEcdsaKeyError::KeyNotFound("not found".to_string()), false)
                .await;

        // Test that this error is converted to the appropriate variant of `IssuanceError`.
        assert_matches!(error, IssuanceError::KeyNotFound(_));

        assert!(wallet.has_registration());
        assert!(!wallet.is_locked());
    }

    #[tokio::test]
    async fn test_accept_pid_issuance_error_pid_issuer() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        wallet
            .mut_storage()
            .expect_fetch_data::<ChangePinData>()
            .returning(|| Ok(None));

        // Have the mock OpenID4VCI session return an error upon accepting.
        let pid_issuer = {
            let mut client = MockIssuanceSession::new();
            client
                .expect_accept()
                .return_once(|| Err(WalletIssuanceError::IssuerMismatch));

            client.expect_issuer().return_const(IssuerRegistration::new_mock());

            client
        };
        wallet.session = Some(Session::Issuance(WalletIssuanceSession::Issuance {
            pid_purpose: Some(PidIssuancePurpose::Enrollment),
            preview_attestations: vec_nonempty![AttestationPresentation::new_mock()],
            protocol_state: pid_issuer,
        }));

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
    async fn test_accept_pid_issuance_error_missing_pid_sd_jwt() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        // Have the mock OpenID4VCI session issue the PID only in Mdoc format.
        let (mdoc_credential, metadata_documents, _) = mdoc_pid();
        let (pid_issuer, attestations) = mock_issuance_session([(mdoc_credential, metadata_documents)]);
        wallet.session = Some(Session::Issuance(WalletIssuanceSession::Issuance {
            pid_purpose: Some(PidIssuancePurpose::Enrollment),
            preview_attestations: attestations,
            protocol_state: pid_issuer,
        }));

        wallet
            .mut_storage()
            .expect_fetch_data::<ChangePinData>()
            .returning(|| Ok(None));

        // Accepting PID issuance should result in an error.
        let error = wallet
            .accept_issuance(PIN.to_owned())
            .await
            .expect_err("Accepting PID issuance should have resulted in an error");

        assert_matches!(error, IssuanceError::MissingPidSdJwt);

        assert!(wallet.has_registration());
        assert!(!wallet.is_locked());
    }

    #[tokio::test]
    async fn test_accept_pid_issuance_error_storage() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        // Have the mock OpenID4VCI session issue an SD-JWT PID.
        let (sd_jwt_credential, metadata_documents, _) = sd_jwt_pid();
        let (pid_issuer, attestations) = mock_issuance_session([(sd_jwt_credential, metadata_documents)]);
        wallet.session = Some(Session::Issuance(WalletIssuanceSession::Issuance {
            pid_purpose: Some(PidIssuancePurpose::Enrollment),
            preview_attestations: attestations,
            protocol_state: pid_issuer,
        }));

        // Have the storage return an error on query.
        wallet
            .mut_storage()
            .expect_fetch_data::<ChangePinData>()
            .returning(|| Ok(None));

        wallet
            .mut_storage()
            .expect_insert_credentials()
            .withf(|_, _| true)
            .returning(|_, _| Err(StorageError::AlreadyOpened));

        wallet
            .mut_storage()
            .expect_fetch_data::<InstructionData>()
            .returning(|| {
                Ok(Some(InstructionData {
                    instruction_sequence_number: 0,
                }))
            });

        wallet
            .mut_storage()
            .expect_upsert_data::<InstructionData>()
            .returning(|_| Ok(()));

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
        let ca = Ca::generate("myca", Default::default()).unwrap();
        let cert_type = CertificateType::from(IssuerRegistration::new_mock());
        let issuer_key_pair = ca.generate_key_pair("mycert", cert_type, Default::default()).unwrap();

        let time_generator = MockTimeGenerator::default();

        let (payload, type_metadata) = create_example_pid_credential_payload(&time_generator);
        let sd_jwt = payload
            .into_signed_sd_jwt(&type_metadata.normalized_metadata, &issuer_key_pair)
            .now_or_never()
            .unwrap()
            .unwrap();

        let attestation_id = Uuid::new_v4();
        let stored = StoredAttestationCopy::new(
            attestation_id,
            Uuid::new_v4(),
            ValidityWindow::new_valid_mock(),
            StoredAttestation::SdJwt {
                key_identifier: "sd_jwt_key_identifier".to_string(),
                sd_jwt: sd_jwt.into_verified(),
            },
            type_metadata.normalized_metadata,
            None,
        );

        // When the attestation already exists in the database, we expect the identity to be known.
        let previews = [create_example_pid_preview_data(&time_generator, Format::SdJwt).0];
        let result = match_preview_and_stored_attestations(&previews, vec![stored.clone()], &time_generator, None);
        let (_, identities): (Vec<_>, Vec<_>) = multiunzip(result);
        assert_eq!(vec![Some(attestation_id)], identities);

        // When the existing attestation has a different format, the identity is None.
        let previews = [create_example_pid_preview_data(&time_generator, Format::MsoMdoc).0];
        let result = match_preview_and_stored_attestations(&previews, vec![stored.clone()], &time_generator, None);
        let (_, identities): (Vec<_>, Vec<_>) = multiunzip(result);
        assert_eq!(vec![None], identities);

        // When the preview contains the same attestation twice, we expect only the first identity to be known.
        let previews = [
            create_example_pid_preview_data(&time_generator, Format::SdJwt).0,
            create_example_pid_preview_data(&time_generator, Format::SdJwt).0,
        ];
        let result = match_preview_and_stored_attestations(&previews, vec![stored.clone()], &time_generator, None);
        let (_, identities): (Vec<_>, Vec<_>) = multiunzip(result);
        assert_eq!(vec![Some(attestation_id), None], identities);

        // When the attestation already exists in the database, but the preview has a newer nbf, it should be considered
        // as a new attestation and the identity is None.
        let (mut preview, _) = create_example_pid_preview_data(&time_generator, Format::SdJwt);
        preview.credential_payload.not_before = Some(Utc::now().add(Duration::days(365)).into());
        let previews = [preview];
        let result = match_preview_and_stored_attestations(&previews, vec![stored.clone()], &time_generator, None);
        let (_, identities): (Vec<_>, Vec<_>) = multiunzip(result);
        assert_eq!(vec![None], identities);

        // When the attestation doesn't exists in the database, the identity is None.
        let (mut preview, _) = create_example_pid_preview_data(&time_generator, Format::SdJwt);
        preview.credential_payload.attestation_type = String::from("att_type_1");
        let previews = [preview];
        let result = match_preview_and_stored_attestations(&previews, vec![stored.clone()], &time_generator, None);
        let (_, identities): (Vec<_>, Vec<_>) = multiunzip(result);
        assert_eq!(vec![None], identities);

        // If the attestation is the PID, then its identity should match the identity of a stored PID
        // even when that wouldn't be the case for non-PID attestations.
        let paths = PidAttributePaths {
            login: vec_nonempty!["login".to_string()],
            recovery_code: vec_nonempty!["recovery_code".to_string()],
        };
        let pid_config: PidAttributesConfiguration = PidAttributesConfiguration {
            mso_mdoc: HashMap::new(),
            sd_jwt: HashMap::from([
                (PID_ATTESTATION_TYPE.to_string(), paths.clone()),
                ("att_type_1".to_string(), paths),
            ]),
        };
        let (mut preview, _) = create_example_pid_preview_data(&time_generator, Format::SdJwt);
        preview.credential_payload.attestation_type = String::from("att_type_1");
        let previews = [preview];
        let result = match_preview_and_stored_attestations(&previews, vec![stored], &time_generator, Some(&pid_config));
        let (_, identities): (Vec<_>, Vec<_>) = multiunzip(result);
        assert_eq!(vec![Some(attestation_id)], identities);
    }

    #[tokio::test]
    async fn test_accept_issuance_error_revoked_user_request() {
        // UserRequest revocation always resets the wallet, regardless of session type.
        let (wallet, error) = test_accept_pid_issuance_error_remote_key(
            RemoteEcdsaKeyError::Instruction(InstructionError::AccountRevoked(AccountRevokedData {
                revocation_reason: RevocationReason::UserRequest,
                can_register_new_account: true,
            })),
            true,
        )
        .await;

        assert_matches!(
            error,
            IssuanceError::Instruction(InstructionError::AccountRevoked(AccountRevokedData {
                revocation_reason: RevocationReason::UserRequest,
                can_register_new_account: true,
            }))
        );
        assert!(!wallet.has_registration());
        assert!(wallet.is_locked());
        assert_matches!(
            wallet.storage.read().await.state().await.unwrap(),
            StorageState::Uninitialized
        );
    }

    #[tokio::test]
    async fn test_accept_issuance_error_revoked_admin_request() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        wallet
            .mut_storage()
            .expect_fetch_data::<ChangePinData>()
            .returning(|| Ok(None));

        // AdminRequest revocation stores the revocation reason without resetting the wallet.
        wallet
            .mut_storage()
            .expect_insert_data::<AccountRevokedData>()
            .times(1)
            .returning(|_| Ok(()));

        let pid_issuer = {
            let mut client = MockIssuanceSession::new();
            client.expect_accept().return_once(|| {
                Err(WalletIssuanceError::Jwt(JwtError::Signing(Box::new(
                    RemoteEcdsaKeyError::Instruction(InstructionError::AccountRevoked(AccountRevokedData {
                        revocation_reason: RevocationReason::AdminRequest,
                        can_register_new_account: true,
                    })),
                ))))
            });
            client.expect_issuer().return_const(IssuerRegistration::new_mock());
            client
        };
        wallet.session = Some(Session::Issuance(WalletIssuanceSession::Issuance {
            pid_purpose: Some(PidIssuancePurpose::Enrollment),
            preview_attestations: vec_nonempty![AttestationPresentation::new_mock()],
            protocol_state: pid_issuer,
        }));

        let error = wallet
            .accept_issuance(PIN.to_owned())
            .await
            .expect_err("Accepting PID issuance should have resulted in an error");

        assert_matches!(
            error,
            IssuanceError::Instruction(InstructionError::AccountRevoked(AccountRevokedData {
                revocation_reason: RevocationReason::AdminRequest,
                can_register_new_account: true
            }))
        );
        // After an AdminRequest revocation, the wallet remains registered.
        assert!(wallet.has_registration());
        assert!(!wallet.is_locked());
    }
}
