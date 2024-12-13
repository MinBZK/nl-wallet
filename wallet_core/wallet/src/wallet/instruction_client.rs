use std::sync::Arc;

use platform_support::hw_keystore::PlatformEcdsaKey;
use wallet_common::config::http::TlsPinningConfig;
use wallet_common::config::wallet_config::WalletConfiguration;
use wallet_common::jwt::EcdsaDecodingKey;
use wallet_common::update_policy::VersionState;

use crate::account_provider::AccountProviderClient;
use crate::errors::ChangePinError;
use crate::instruction::InstructionClient;
use crate::pin::change::ChangePinStorage;
use crate::repository::Repository;
use crate::storage::Storage;

use super::Wallet;
use super::WalletRegistration;

impl<CR, S, PEK, APC, DS, IC, MDS, WIC, UR> Wallet<CR, S, PEK, APC, DS, IC, MDS, WIC, UR>
where
    CR: Repository<Arc<WalletConfiguration>>,
    S: Storage,
    PEK: PlatformEcdsaKey,
    APC: AccountProviderClient,
    WIC: Default,
    UR: Repository<VersionState>,
{
    /// Construct an [`InstructionClient`] for this [`Wallet`].
    /// This is the recommended way to obtain an [`InstructionClient`], because this function
    /// will try to finalize any unfinished PIN change process.
    pub(super) async fn new_instruction_client<'a>(
        &'a self,
        pin: String,
        registration: &'a WalletRegistration<PEK>,
        client_config: &'a TlsPinningConfig,
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
            client_config,
            instruction_result_public_key,
        );

        Ok(client)
    }
}
