use std::sync::Arc;

use configuration::wallet_config::WalletConfiguration;
use jwt::EcdsaDecodingKey;
use platform_support::attested_key::AttestedKey;
use platform_support::attested_key::AttestedKeyHolder;
use wallet_common::http::TlsPinningConfig;
use wallet_common::update_policy::VersionState;

use crate::account_provider::AccountProviderClient;
use crate::errors::ChangePinError;
use crate::instruction::InstructionClient;
use crate::pin::change::ChangePinStorage;
use crate::repository::Repository;
use crate::storage::RegistrationData;
use crate::storage::Storage;

use super::Wallet;

impl<CR, UR, S, AKH, APC, DS, IS, MDS, WIC> Wallet<CR, UR, S, AKH, APC, DS, IS, MDS, WIC>
where
    CR: Repository<Arc<WalletConfiguration>>,
    UR: Repository<VersionState>,
    S: Storage,
    AKH: AttestedKeyHolder,
    APC: AccountProviderClient,
    WIC: Default,
{
    /// Construct an [`InstructionClient`] for this [`Wallet`].
    /// This is the recommended way to obtain an [`InstructionClient`], because this function
    /// will try to finalize any unfinished PIN change process.
    pub(super) async fn new_instruction_client(
        &self,
        pin: String,
        attested_key: Arc<AttestedKey<AKH::AppleKey, AKH::GoogleKey>>,
        registration_data: RegistrationData,
        client_config: TlsPinningConfig,
        instruction_result_public_key: EcdsaDecodingKey,
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
            registration_data,
            client_config,
            instruction_result_public_key,
        );

        Ok(client)
    }
}
