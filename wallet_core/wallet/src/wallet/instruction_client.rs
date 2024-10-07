use platform_support::hw_keystore::PlatformEcdsaKey;
use wallet_common::{jwt::EcdsaDecodingKey, urls::BaseUrl};

use crate::{
    account_provider::AccountProviderClient, config::ConfigurationRepository, errors::ChangePinError,
    instruction::InstructionClient, pin::change::ChangePinStorage, storage::Storage,
};

use super::{Wallet, WalletRegistration};

impl<CR, S, PEK, APC, DS, IS, MDS> Wallet<CR, S, PEK, APC, DS, IS, MDS>
where
    CR: ConfigurationRepository,
    S: Storage,
    PEK: PlatformEcdsaKey,
    APC: AccountProviderClient,
{
    /// Construct an [`InstructionClient`] for this [`Wallet`].
    /// This is the recommended way to obtain an [`InstructionClient`], because this function
    /// will try to finalize any unfinished PIN change process.
    pub(super) async fn new_instruction_client<'a>(
        &'a self,
        pin: String,
        registration: &'a WalletRegistration<PEK>,
        account_provider_base_url: &'a BaseUrl,
        instruction_result_public_key: &'a EcdsaDecodingKey,
    ) -> Result<InstructionClient<'a, S, PEK, APC>, ChangePinError> {
        tracing::info!("Try to finalize PIN change if it is in progress");

        if self.storage.get_change_pin_state().await?.is_some() {
            self.continue_change_pin(pin.clone()).await?;
        }

        let client = InstructionClient::new(
            pin,
            &self.storage,
            &registration.hw_privkey,
            &self.account_provider_client,
            &registration.data,
            account_provider_base_url,
            instruction_result_public_key,
        );

        Ok(client)
    }
}
