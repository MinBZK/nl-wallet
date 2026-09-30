use std::sync::Arc;

use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::wallet_issuance::IssuanceDiscovery;
use platform_support::attested_key::AttestedKey;
use platform_support::attested_key::AttestedKeyHolder;
use update_policy_model::update_policy::VersionState;
use wallet_configuration::wallet_config::WalletConfiguration;

use super::Wallet;
use super::refresh_certificate::RefreshCertificateError;
use crate::account_provider::AccountProviderClient;
use crate::errors::ChangePinError;
use crate::instruction::HwSignedInstructionClient;
use crate::instruction::InstructionClient;
use crate::instruction::InstructionClientParameters;
use crate::instruction::InstructionError;
use crate::instruction::RemoteWiaClient;
use crate::pin::change::ChangePinStorage;
use crate::pin::key::Pin;
use crate::repository::Repository;
use crate::storage::RegistrationData;
use crate::storage::Storage;

impl<CR, UR, S, AKH, APC, CID, DCC, CPC, SLC> Wallet<CR, UR, S, AKH, APC, CID, DCC, CPC, SLC>
where
    CR: Repository<Arc<WalletConfiguration>>,
    UR: Repository<VersionState>,
    S: Storage,
    AKH: AttestedKeyHolder,
    APC: AccountProviderClient,
    CID: IssuanceDiscovery,
    DCC: DisclosureClient,
{
    /// Construct an [`InstructionClient`] for this [`Wallet`].
    /// This is the recommended way to obtain an [`InstructionClient`], because this function
    /// will try to finalize any unfinished PIN change process and, if needed, refresh the wallet
    /// certificate before the caller sends its own instruction.
    pub(super) async fn new_instruction_client(
        &mut self,
        pin: Pin,
        attested_key: Arc<AttestedKey<AKH::AppleKey, AKH::GoogleKey>>,
        parameters: InstructionClientParameters,
    ) -> Result<InstructionClient<S, AKH::AppleKey, AKH::GoogleKey, APC>, ChangePinError> {
        tracing::info!("Try to finalize PIN change if it is in progress");

        if self.storage.get_change_pin_state().await?.is_some() {
            self.continue_change_pin(&pin).await?;
        }

        let client = InstructionClient::new(
            pin,
            Arc::clone(&self.storage),
            attested_key,
            Arc::clone(&self.account_provider_client),
            Arc::new(parameters),
        );

        let current_certificate = self
            .registration
            .as_key_and_registration_data()
            .map(|(_, registration_data)| registration_data.wallet_certificate.clone());

        if let Some(current_certificate) = current_certificate {
            let config = self.config_repository.get();

            // A refresh failure does not stop the caller from obtaining its instruction client, unless it reveals
            // that the account has been revoked, since that represents a real state change the caller needs to
            // react to.
            if let Err(RefreshCertificateError::Instruction(error @ InstructionError::AccountRevoked(_))) = self
                .refresh_wallet_certificate_if_needed(&client, &current_certificate, &config)
                .await
            {
                return Err(ChangePinError::Instruction(error));
            }
        }

        Ok(client)
    }

    pub(super) fn new_hw_signed_instruction_client(
        &self,
        attested_key: Arc<AttestedKey<AKH::AppleKey, AKH::GoogleKey>>,
        parameters: InstructionClientParameters,
    ) -> HwSignedInstructionClient<S, AKH::AppleKey, AKH::GoogleKey, APC> {
        HwSignedInstructionClient::new(
            Arc::clone(&self.storage),
            attested_key,
            Arc::clone(&self.account_provider_client),
            Arc::new(parameters),
        )
    }

    pub(super) fn new_remote_wia_client(
        &self,
        attested_key: Arc<AttestedKey<AKH::AppleKey, AKH::GoogleKey>>,
        registration_data: &RegistrationData,
        config: &WalletConfiguration,
    ) -> RemoteWiaClient<S, AKH::AppleKey, AKH::GoogleKey, APC> {
        RemoteWiaClient::new(self.new_hw_signed_instruction_client(
            attested_key,
            InstructionClientParameters::new(
                registration_data.wallet_id.clone(),
                registration_data.pin_salt.clone(),
                registration_data.wallet_certificate.clone(),
                config.account_server.http_config.clone(),
                config.account_server.instruction_result_public_keys.clone(),
            ),
        ))
    }
}
