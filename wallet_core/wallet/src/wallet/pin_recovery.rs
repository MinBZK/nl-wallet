use std::sync::Arc;

use tracing::info;
use url::Url;

use attestation_data::attributes::AttributeValue;
use attestation_data::constants::PID_ATTESTATION_TYPE;
use attestation_data::constants::PID_RECOVERY_CODE;
use attestation_types::claim_path::ClaimPath;
use error_category::ErrorCategory;
use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::issuance_session::IssuanceSession;
use openid4vc::issuance_session::IssuedCredential;
use openid4vc::oidc::OidcError;
use platform_support::attested_key::AttestedKeyHolder;
use update_policy_model::update_policy::VersionState;
use utils::vec_nonempty;
use wallet_account::messages::instructions::DiscloseRecoveryCodePinRecovery;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::AttestationAttributeValue;
use crate::account_provider::AccountProviderClient;
use crate::digid::DigidClient;
use crate::digid::DigidError;
use crate::digid::DigidSession;
use crate::errors::InstructionError;
use crate::errors::PinKeyError;
use crate::errors::PinValidationError;
use crate::errors::StorageError;
use crate::instruction::PinRecoveryRemoteEcdsaWscd;
use crate::pin::key::PinKey;
use crate::pin::key::new_pin_salt;
use crate::repository::Repository;
use crate::storage::AttestationFormatQuery;
use crate::storage::PinRecoveryData;
use crate::storage::PinRecoveryState;
use crate::storage::RegistrationData;
use crate::storage::Storage;
use crate::validate_pin;

use super::IssuanceError;
use super::Session;
use super::Wallet;
use super::WalletRegistration;

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum PinRecoveryError {
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

    #[error("error during PID issuance: {0}")]
    #[category(defer)]
    Issuance(#[from] IssuanceError),

    #[error("no recovery code found in PID")]
    #[category(unexpected)]
    MissingRegistrationCode,

    #[error("recovery code had unexpected format: {0:#?}")]
    #[category(pd)]
    InvalidRecoveryCodeFormat(AttestationAttributeValue),

    #[error("incorrect recovery code: expected {expected}, received {received}")]
    #[category(pd)]
    IncorrectRecoveryCode {
        expected: AttributeValue,
        received: AttributeValue,
    },

    #[error("the new PIN does not adhere to requirements: {0}")]
    #[category(expected)]
    PinValidation(#[from] PinValidationError),

    #[error("error computing PIN public key: {0}")]
    #[category(unexpected)]
    PinKey(#[from] PinKeyError),

    #[error("storage error: {0}")]
    #[category(unexpected)]
    Storage(#[from] StorageError),

    #[error("no PID received")]
    #[category(unexpected)]
    MissingPid,

    #[error("no SD-JWT PID received")]
    #[category(unexpected)]
    MissingSdJwtPid,

    #[error("failed to disclose recovery code to WP: {0}")]
    #[category(defer)]
    DiscloseRecoveryCode(#[source] InstructionError),

    #[error("PIN recovery in unexpected state: expected {expected:#?}, found {found:#?}")]
    #[category(unexpected)]
    UnexpectedState {
        expected: PinRecoveryState,
        found: Option<PinRecoveryState>,
    },

    #[error("cannot cancel PIN: already committed")]
    #[category(unexpected)]
    CommittedToPinRecovery,

    #[error("user denied DigiD authentication")]
    #[category(expected)]
    DeniedDigiD,
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
    async fn check_recovery_state(&self, expected: PinRecoveryState) -> Result<(), PinRecoveryError> {
        let found = self
            .storage
            .read()
            .await
            .fetch_data::<PinRecoveryData>()
            .await?
            .map(|data| data.state);

        if found != Some(expected) {
            return Err(PinRecoveryError::UnexpectedState { found, expected });
        }

        Ok(())
    }

    pub async fn create_pin_recovery_redirect_uri(&mut self) -> Result<Url, PinRecoveryError> {
        info!("Generating DigiD auth URL, starting OpenID connect discovery");

        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(PinRecoveryError::VersionBlocked);
        }

        info!("Checking if registered");
        if !self.registration.is_registered() {
            return Err(PinRecoveryError::NotRegistered);
        }

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(PinRecoveryError::Locked);
        }

        info!("Checking if there is an active session");
        if self.session.is_some() {
            return Err(PinRecoveryError::SessionState);
        }

        // No need to check if a `PinRecoveryData` is already stored: we can always start PIN recovery again.

        self.storage
            .write()
            .await
            .upsert_data(&PinRecoveryData {
                state: PinRecoveryState::Starting,
            })
            .await?;

        let url = self.pid_issuance_auth_url().await?;
        Ok(url)
    }

    pub async fn continue_pin_recovery(&mut self, redirect_uri: Url) -> Result<(), PinRecoveryError> {
        info!("Received redirect URI, processing URI and retrieving access token");

        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(PinRecoveryError::VersionBlocked);
        }

        info!("Checking if registered");
        if !self.registration.is_registered() {
            return Err(PinRecoveryError::NotRegistered);
        }

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(PinRecoveryError::Locked);
        }

        self.check_recovery_state(PinRecoveryState::Starting).await?;

        if !matches!(self.session, Some(Session::Digid(..))) {
            return Err(PinRecoveryError::SessionState);
        }

        info!("Checking if there is an active DigiD issuance session");
        let Some(Session::Digid(session)) = self.session.take() else {
            return Err(PinRecoveryError::SessionState);
        };

        let pid_issuance_config = &self.config_repository.get().pid_issuance;
        let token_request = session
            .into_token_request(&pid_issuance_config.digid_http_config, redirect_uri)
            .await
            .map_err(|error| {
                if matches!(error, DigidError::Oidc(OidcError::Denied)) {
                    PinRecoveryError::DeniedDigiD
                } else {
                    PinRecoveryError::Issuance(IssuanceError::DigidSessionFinish(error))
                }
            })?;

        let config = self.config_repository.get();

        // Check the recovery code in the received PID against the one in the stored PID, as otherwise
        // the WP will reject our PIN recovery instructions.

        let previews = self
            .issuance_fetch_previews(
                token_request,
                config.pid_issuance.pid_issuer_url.clone(),
                &config.issuer_trust_anchors(),
                true,
                false,
            )
            .await?;

        let received_recovery_code = &previews
            .first()
            .unwrap()
            .attributes
            .iter()
            .find(|attr| attr.key == vec![PID_RECOVERY_CODE])
            .ok_or(PinRecoveryError::MissingRegistrationCode)?
            .value;

        let AttestationAttributeValue::Basic(received_recovery_code) = received_recovery_code else {
            return Err(PinRecoveryError::InvalidRecoveryCodeFormat(
                received_recovery_code.clone(),
            ));
        };

        let stored_pid_credential_payload = self
            .storage
            .write()
            .await
            .fetch_unique_attestations_by_type(&[PID_ATTESTATION_TYPE].into(), AttestationFormatQuery::SdJwt)
            .await
            .map_err(IssuanceError::AttestationQuery)?
            .pop()
            .expect("no PID found in registered wallet")
            .into_credential_payload();

        let stored_recovery_code = stored_pid_credential_payload
            .previewable_payload
            .attributes
            .get(&vec_nonempty![ClaimPath::SelectByKey(PID_RECOVERY_CODE.to_string())])
            .expect("failed to retrieve recovery code from PID")
            .expect("no recovery code found in PID");

        if stored_recovery_code != received_recovery_code {
            return Err(PinRecoveryError::IncorrectRecoveryCode {
                expected: stored_recovery_code.clone(),
                received: received_recovery_code.clone(),
            });
        }

        Ok(())
    }

    pub async fn complete_pin_recovery(&mut self, new_pin: String) -> Result<(), PinRecoveryError> {
        info!("Checking if registered");
        if !self.registration.is_registered() {
            return Err(PinRecoveryError::NotRegistered);
        }

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(PinRecoveryError::Locked);
        }

        info!("Checking if there is an active issuance session");
        let Some(Session::Issuance(issuance_session)) = &self.session else {
            return Err(PinRecoveryError::SessionState);
        };

        self.check_recovery_state(PinRecoveryState::Starting).await?;

        // We don't check if the wallet is blocked, since PIN recovery is allowed in that case.

        validate_pin(&new_pin)?;

        // Both instructions sent to the WP in this method use the new PIN, and therefore also a new salt.
        // So, generate a new salt and use that in the instruction client below.

        let (attested_key, current_registration_data) = self
            .registration
            .as_key_and_registration_data()
            .expect("missing registration data");

        let new_pin_salt = new_pin_salt();
        let registration_data = RegistrationData {
            pin_salt: new_pin_salt.clone(),
            ..current_registration_data.clone()
        };

        // Accept issuance to obtain the PID. This sends the `StartPinRecovery` instruction to the WP.

        let config = self.config_repository.get();
        let instruction_client = self
            .new_instruction_client(
                new_pin.clone(),
                Arc::clone(attested_key),
                registration_data.clone(),
                config.account_server.http_config.clone(),
                config.account_server.instruction_result_public_key.as_inner().into(),
            )
            .await
            .map_err(IssuanceError::from)?;

        let pin_pubkey = PinKey {
            pin: &new_pin,
            salt: &new_pin_salt,
        }
        .verifying_key()?;

        let pin_recovery_wscd = PinRecoveryRemoteEcdsaWscd::new(instruction_client.clone(), pin_pubkey.into());

        // `accept_issuance()` below is the point of no return. If the app is killed between there and completion,
        // PIN recovery will have to start again from the start.

        self.storage
            .write()
            .await
            .upsert_data(&PinRecoveryData {
                state: PinRecoveryState::Completing,
            })
            .await?;

        let issuance_result = issuance_session
            .protocol_state
            .accept_issuance(&config.issuer_trust_anchors(), &pin_recovery_wscd, true)
            .await
            .map_err(|error| Self::handle_accept_issuance_error(error, issuance_session))?;

        // Store the new wallet certificate and the new salt.

        let new_wallet_certificate = pin_recovery_wscd
            .certificate()
            .take()
            .expect("missing new wallet certificate");

        let registration_data = RegistrationData {
            wallet_certificate: new_wallet_certificate,
            pin_salt: new_pin_salt,
            ..registration_data.clone()
        };
        self.storage.write().await.upsert_data(&registration_data).await?;

        let attested_key = Arc::clone(attested_key);
        self.registration = WalletRegistration::Registered {
            attested_key: Arc::clone(&attested_key),
            data: registration_data.clone(),
        };

        // Get an SD-JWT copy out of the PID we just received.

        let attestation = issuance_result
            .into_iter()
            .find(|attestation| attestation.attestation_type == PID_ATTESTATION_TYPE)
            .ok_or(PinRecoveryError::MissingPid)?;

        let pid = attestation
            .copies
            .into_inner()
            .into_iter()
            .find_map(|copy| match copy {
                IssuedCredential::MsoMdoc { .. } => None,
                IssuedCredential::SdJwt { sd_jwt, .. } => Some(sd_jwt),
            })
            .ok_or(PinRecoveryError::MissingSdJwtPid)?;

        let recovery_code_disclosure = pid
            .into_presentation_builder()
            .disclose(&vec_nonempty![ClaimPath::SelectByKey(PID_RECOVERY_CODE.to_string())])
            .unwrap()
            .finish()
            .into();

        // Finish PIN recovery by sending the second WP instruction.

        // Use a new instruction client that uses our new WP certificate
        self.new_instruction_client(
            new_pin.clone(),
            Arc::clone(&attested_key),
            registration_data,
            config.account_server.http_config.clone(),
            config.account_server.instruction_result_public_key.as_inner().into(),
        )
        .await
        .map_err(IssuanceError::from)?
        .send(DiscloseRecoveryCodePinRecovery {
            recovery_code_disclosure,
        })
        .await
        .map_err(PinRecoveryError::DiscloseRecoveryCode)?;

        self.storage.write().await.delete_data::<PinRecoveryData>().await?;

        Ok(())
    }

    pub async fn cancel_pin_recovery(&mut self) -> Result<(), PinRecoveryError> {
        info!("Checking if registered");
        if !self.registration.is_registered() {
            return Err(PinRecoveryError::NotRegistered);
        }

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(PinRecoveryError::Locked);
        }

        // We don't check if the wallet is blocked: PIN recovery is allowed in that case, so cancelling it is too.

        let Some(state) = self
            .storage
            .read()
            .await
            .fetch_data::<PinRecoveryData>()
            .await?
            .map(|data| data.state)
        else {
            return Ok(()); // Not currently recovering PIN; nothing to do
        };

        if matches!(state, PinRecoveryState::Completing) {
            return Err(PinRecoveryError::CommittedToPinRecovery);
        }

        self.storage.write().await.delete_data::<PinRecoveryData>().await?;

        Ok(())
    }
}
