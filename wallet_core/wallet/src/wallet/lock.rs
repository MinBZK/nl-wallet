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

    pub fn set_lock_callback<F>(&mut self, callback: F)
    where
        F: FnMut(bool) + Send + Sync + 'static,
    {
        // callback(self.lock.is_locked());
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

#[cfg(test)]
mod tests {
    use std::{
        ops::Deref,
        sync::{Arc, Mutex},
    };

    use mockall::predicate::*;

    use wallet_common::{
        account::{
            jwt::Jwt,
            messages::instructions::{CheckPin, Instruction, InstructionResultClaims},
            signed::SequenceNumberComparison,
        },
        keys::EcdsaKey,
        utils,
    };

    use crate::{config::ConfigurationRepository, pin::key::PinKey};

    use super::super::tests::{WalletWithMocks, ACCOUNT_SERVER_KEYS};

    // Tests both setting and clearing the lock callback.
    #[tokio::test]
    async fn test_wallet_lock_unlock_callback() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::registered().await;

        // Wrap a `Vec<bool>` in both a `Mutex` and `Arc`,
        // so we can write to it from the closure.
        let is_locked_vec = Arc::new(Mutex::new(Vec::<bool>::with_capacity(1)));
        let callback_is_locked_vec = Arc::clone(&is_locked_vec);

        // Set the lock callback on the `Wallet`,
        // which should immediately be called exactly once.
        wallet.set_lock_callback(move |is_locked| callback_is_locked_vec.lock().unwrap().push(is_locked));

        // Lock the `Wallet`, then lock it again.
        wallet.lock();
        wallet.lock();

        // Mock the responses of the account server for both the instruction challenge
        // and the actual instruction and check the contents of those messages.
        let challenge = utils::random_bytes(32);

        // Set up the instruction challenge.
        let challenge_response = challenge.clone();
        let wallet_cert = wallet.registration.as_ref().unwrap().wallet_certificate.clone();
        let hw_pubkey = wallet.hw_privkey.verifying_key().await.unwrap();

        wallet
            .account_provider_client
            .expect_instruction_challenge()
            .with(
                eq(wallet.config_repository.config().account_server.base_url.clone()),
                always(),
            )
            .return_once(move |_, challenge_request| {
                assert_eq!(challenge_request.certificate.0, wallet_cert.0);

                let claims = challenge_request
                    .message
                    .parse_and_verify(&hw_pubkey.into())
                    .expect("Could not verify check pin challenge request");

                assert_eq!(claims.sequence_number, 1);
                assert_eq!(claims.iss, "wallet");

                Ok(challenge_response)
            });

        // Set up the instruction.
        let wallet_cert = wallet.registration.as_ref().unwrap().wallet_certificate.clone();
        let hw_pubkey = wallet.hw_privkey.verifying_key().await.unwrap();

        let pin = "051097";
        let pin_key = PinKey::new(pin, &wallet.registration.as_ref().unwrap().pin_salt.0);
        let pin_pubkey = pin_key.verifying_key().unwrap();

        let result_claims = InstructionResultClaims {
            result: (),
            iss: "wallet_unit_test".to_string(),
            iat: jsonwebtoken::get_current_timestamp(),
        };
        let result = Jwt::sign(&result_claims, &ACCOUNT_SERVER_KEYS.instruction_result_signing_key)
            .await
            .unwrap();

        wallet
            .account_provider_client
            .expect_instruction()
            .with(
                eq(wallet.config_repository.config().account_server.base_url.clone()),
                always(),
            )
            .return_once(move |_, instruction: Instruction<CheckPin>| {
                assert_eq!(instruction.certificate.0, wallet_cert.0);

                instruction
                    .instruction
                    .parse_and_verify(
                        &challenge,
                        SequenceNumberComparison::LargerThan(1),
                        &hw_pubkey,
                        &pin_pubkey,
                    )
                    .expect("Could not verify check pin instruction");

                Ok(result)
            });

        // Unlock the `Wallet` with the PIN.
        wallet.unlock(pin.to_string()).await.expect("Could not unlock wallet");

        // Infer that the closure is still alive by counting the `Arc` references.
        assert_eq!(Arc::strong_count(&is_locked_vec), 2);

        // Test the contents of the `Vec<bool>`.
        {
            let is_locked_vec = is_locked_vec.lock().unwrap();

            assert_eq!(is_locked_vec.deref(), &vec![false, true, false]);
        }

        // Clear the lock callback on the `Wallet.`
        wallet.clear_lock_callback();

        // Infer that the closure is now dropped by counting the `Arc` references.
        assert_eq!(Arc::strong_count(&is_locked_vec), 1);

        // Lock the `Wallet` again.
        wallet.lock();

        // Test that the callback was not called.
        assert_eq!(is_locked_vec.lock().unwrap().len(), 3);
    }
}
