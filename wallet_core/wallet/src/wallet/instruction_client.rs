use platform_support::attested_key::AttestedKey;
use platform_support::attested_key::AttestedKeyHolder;
use wallet_common::config::http::TlsPinningConfig;
use wallet_common::jwt::EcdsaDecodingKey;

use crate::account_provider::AccountProviderClient;
use crate::config::ConfigurationRepository;
use crate::errors::ChangePinError;
use crate::instruction::InstructionClient;
use crate::pin::change::ChangePinStorage;
use crate::storage::RegistrationData;
use crate::storage::Storage;

use super::Wallet;

impl<CR, S, AKH, APC, DS, IS, MDS, WIC> Wallet<CR, S, AKH, APC, DS, IS, MDS, WIC>
where
    CR: ConfigurationRepository,
    S: Storage,
    AKH: AttestedKeyHolder,
    APC: AccountProviderClient,
    WIC: Default,
{
    /// Construct an [`InstructionClient`] for this [`Wallet`].
    /// This is the recommended way to obtain an [`InstructionClient`], because this function
    /// will try to finalize any unfinished PIN change process.
    pub(super) async fn new_instruction_client<'a>(
        &'a self,
        pin: String,
        attested_key: &'a AttestedKey<<AKH as AttestedKeyHolder>::AppleKey, <AKH as AttestedKeyHolder>::GoogleKey>,
        registration_data: &'a RegistrationData,
        client_config: &'a TlsPinningConfig,
        instruction_result_public_key: &'a EcdsaDecodingKey,
    ) -> Result<
        InstructionClient<'a, S, <AKH as AttestedKeyHolder>::AppleKey, <AKH as AttestedKeyHolder>::GoogleKey, APC>,
        ChangePinError,
    > {
        tracing::info!("Try to finalize PIN change if it is in progress");

        if self.storage.get_change_pin_state().await?.is_some() {
            self.continue_change_pin(pin.clone()).await?;
        }

        let client = InstructionClient::new(
            pin,
            &self.storage,
            attested_key,
            &self.account_provider_client,
            registration_data,
            client_config,
            instruction_result_public_key,
        );

        Ok(client)
    }
}
