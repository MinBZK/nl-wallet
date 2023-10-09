use std::error::Error;

use futures::future::TryFutureExt;
use platform_support::hw_keystore::PlatformEcdsaKey;
use tracing::{info, instrument};

use wallet_common::account::messages::instructions::CheckPin;

use crate::{
    account_provider::AccountProviderClient,
    config::ConfigurationRepository,
    instruction::{InstructionClient, InstructionError},
    storage::{Storage, StorageError},
};

use super::Wallet;

#[derive(Debug, thiserror::Error)]
pub enum WalletUnlockError {
    #[error("wallet is not registered")]
    NotRegistered,
    #[error("could not retrieve registration from database: {0}")]
    Database(#[from] StorageError),
    #[error("could not get hardware public key: {0}")]
    HardwarePublicKey(#[source] Box<dyn Error + Send + Sync>),
    #[error("error sending instruction to Wallet Provider: {0}")]
    Instruction(#[from] InstructionError),
}

impl<C, S, K, A, D, P> Wallet<C, S, K, A, D, P> {
    pub fn is_locked(&self) -> bool {
        self.lock.is_locked()
    }

    pub fn set_lock_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(bool) + Send + Sync + 'static,
    {
        callback(self.lock.is_locked());
        self.lock.set_lock_callback(callback);
    }

    pub fn clear_lock_callback(&mut self) {
        self.lock.clear_lock_callback()
    }

    pub fn lock(&mut self) {
        self.lock.lock()
    }

    #[instrument(skip_all)]
    pub async fn unlock(&mut self, pin: String) -> Result<(), WalletUnlockError>
    where
        C: ConfigurationRepository,
        S: Storage,
        K: PlatformEcdsaKey,
        A: AccountProviderClient,
    {
        info!("Validating pin");

        info!("Checking if already registered");
        let registration_data = self
            .registration
            .as_ref()
            .ok_or_else(|| WalletUnlockError::NotRegistered)?;

        let config = self.config_repository.config();

        let remote_instruction = InstructionClient::new(
            pin,
            &self.storage,
            &self.hw_privkey,
            &self.account_provider_client,
            registration_data,
            &config.account_server.base_url,
            &config.account_server.instruction_result_public_key,
        );

        info!("Sending unlock instruction to Wallet Provider");
        remote_instruction
            .send(CheckPin)
            .inspect_ok(|_| {
                info!("Unlock instruction successful, unlocking wallet");

                self.lock.unlock();
            })
            .await?;

        Ok(())
    }
}
