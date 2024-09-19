mod config;
mod instruction;
mod storage;

use tracing::info;

use platform_support::hw_keystore::PlatformEcdsaKey;

use crate::{
    account_provider::AccountProviderClient,
    config::ConfigurationRepository,
    instruction::InstructionClientFactory,
    pin::change::{ChangePinError, ChangePinSession},
    storage::Storage,
    Wallet,
};

impl<CR, S, PEK, APC, DS, IS, MDS> Wallet<CR, S, PEK, APC, DS, IS, MDS>
where
    CR: ConfigurationRepository,
    S: Storage,
    PEK: PlatformEcdsaKey,
    APC: AccountProviderClient,
{
    pub async fn begin_change_pin(&self, old_pin: String, new_pin: String) -> Result<(), ChangePinError> {
        info!("Checking if registered");
        let registration = self
            .registration
            .as_ref()
            .ok_or_else(|| ChangePinError::NotRegistered)?;

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(ChangePinError::Locked);
        }

        let config = self.config_repository.config();
        let instruction_result_public_key = config.account_server.instruction_result_public_key.clone().into();

        let instruction_client = InstructionClientFactory::new(
            &self.storage,
            &registration.hw_privkey,
            &self.account_provider_client,
            &registration.data,
            &config.account_server.base_url,
            &instruction_result_public_key,
        );

        let change_pin_config = ();

        let session = ChangePinSession::new(&instruction_client, &self.storage, &change_pin_config);

        session.begin_change_pin(old_pin, new_pin).await?;

        Ok(())
    }

    pub async fn continue_change_pin(&self, pin: String) -> Result<(), ChangePinError> {
        info!("Checking if registered");

        let registration = self
            .registration
            .as_ref()
            .ok_or_else(|| ChangePinError::NotRegistered)?;

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(ChangePinError::Locked);
        }

        let config = self.config_repository.config();
        let instruction_result_public_key = config.account_server.instruction_result_public_key.clone().into();

        let instruction_client = InstructionClientFactory::new(
            &self.storage,
            &registration.hw_privkey,
            &self.account_provider_client,
            &registration.data,
            &config.account_server.base_url,
            &instruction_result_public_key,
        );

        let change_pin_config = ();

        let session = ChangePinSession::new(&instruction_client, &self.storage, &change_pin_config);

        session.continue_change_pin(pin).await?;

        Ok(())
    }
}
