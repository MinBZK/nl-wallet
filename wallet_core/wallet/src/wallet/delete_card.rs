use std::sync::Arc;

use tracing::info;
use tracing::instrument;

use error_category::ErrorCategory;
use error_category::sentry_capture_error;
use http_utils::client::TlsPinningConfig;
use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::oidc::OidcClient;
use platform_support::attested_key::AttestedKeyHolder;
use update_policy_model::update_policy::VersionState;
use utils::vec_at_least::VecNonEmpty;
use wallet_account::messages::instructions::DeleteKeys;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::account_provider::AccountProviderClient;
use crate::errors::ChangePinError;
use crate::errors::InstructionError;
use crate::errors::StorageError;
use crate::instruction::InstructionClientParameters;
use crate::repository::Repository;
use crate::repository::UpdateableRepository;
use crate::storage::Storage;
use crate::update_policy::UpdatePolicyError;
use crate::wallet::attestations::AttestationsError;

use super::Wallet;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum DeleteCardError {
    #[error("app version is blocked")]
    #[category(expected)]
    VersionBlocked,
    #[error("wallet is not registered")]
    #[category(expected)]
    NotRegistered,
    #[error("wallet is locked")]
    #[category(expected)]
    Locked,

    #[error("error fetching update policy: {0}")]
    UpdatePolicy(#[from] UpdatePolicyError),
    #[error("error finalizing pin change: {0}")]
    ChangePin(#[from] ChangePinError),
    #[error("error sending instruction to Wallet Provider: {0}")]
    Instruction(#[from] InstructionError),
    #[error("error accessing wallet storage: {0}")]
    Storage(#[from] StorageError),
    #[error("error emitting attestations: {0}")]
    Attestations(#[from] AttestationsError),

    #[error("attestation not found")]
    #[category(expected)]
    AttestationNotFound,
    #[error("could not parse attestation id: {0}")]
    #[category(critical)]
    AttestationIdParsing(#[from] uuid::Error),
}

impl<CR, UR, S, AKH, APC, OC, IS, DCC, CPC, SLC> Wallet<CR, UR, S, AKH, APC, OC, IS, DCC, CPC, SLC>
where
    CR: Repository<Arc<WalletConfiguration>>,
    UR: UpdateableRepository<VersionState, TlsPinningConfig, Error = UpdatePolicyError>,
    S: Storage,
    AKH: AttestedKeyHolder,
    APC: AccountProviderClient,
    OC: OidcClient,
    DCC: DisclosureClient,
{
    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn delete_card(&mut self, pin: String, attestation_id: String) -> Result<(), DeleteCardError> {
        info!("Deleting card {attestation_id}");

        let attestation_id = attestation_id.parse()?;

        let config = &self.config_repository.get();

        info!("Fetching update policy");
        self.update_policy_repository
            .fetch(&config.update_policy_server.http_config)
            .await?;

        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(DeleteCardError::VersionBlocked);
        }

        info!("Checking if registered");
        let (attested_key, registration_data) = self
            .registration
            .as_key_and_registration_data()
            .ok_or(DeleteCardError::NotRegistered)?;
        let attested_key = Arc::clone(attested_key);

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(DeleteCardError::Locked);
        }

        info!("Fetching key identifiers for attestation");
        let key_identifiers: VecNonEmpty<_> = self
            .storage
            .read()
            .await
            .fetch_key_identifiers_by_attestation_id(attestation_id)
            .await?
            .try_into()
            .map_err(|_| DeleteCardError::AttestationNotFound)?;

        let instruction_client = self
            .new_instruction_client(
                pin,
                attested_key,
                InstructionClientParameters::new(
                    registration_data.wallet_id.clone(),
                    registration_data.pin_salt.clone(),
                    registration_data.wallet_certificate.clone(),
                    config.account_server.http_config.clone(),
                    config.account_server.instruction_result_public_key.as_inner().into(),
                ),
            )
            .await?;

        info!("Sending DeleteKeys instruction to Wallet Provider");
        self.check_result_for_wallet_revocation(
            instruction_client
                .send(DeleteKeys {
                    identifiers: key_identifiers,
                })
                .await,
        )
        .await?;

        info!("Deleting attestation from local storage");
        self.storage.write().await.delete_attestation(attestation_id).await?;

        self.emit_attestations().await?;

        Ok(())
    }
}
