use futures::future::TryFutureExt;
use platform_support::hw_keystore::PlatformEcdsaKey;
use tracing::{info, instrument};

use wallet_common::account::messages::instructions::CheckPin;

use crate::{
    account_provider::AccountProviderClient,
    config::ConfigurationRepository,
    instruction::{InstructionClient, InstructionError},
    storage::Storage,
};

use super::Wallet;

#[derive(Debug, thiserror::Error)]
pub enum WalletUnlockError {
    #[error("wallet is not registered")]
    NotRegistered,
    #[error("wallet is not locked")]
    NotLocked,
    #[error("error sending instruction to Wallet Provider: {0}")]
    Instruction(#[from] InstructionError),
}

impl<CR, S, PEK, APC, DGS, IS, MDS> Wallet<CR, S, PEK, APC, DGS, IS, MDS> {
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
        CR: ConfigurationRepository,
        S: Storage,
        PEK: PlatformEcdsaKey,
        APC: AccountProviderClient,
    {
        info!("Validating pin");

        info!("Checking if registered");
        let registration_data = self
            .registration
            .as_ref()
            .ok_or_else(|| WalletUnlockError::NotRegistered)?;

        info!("Checking if locked");
        if !self.lock.is_locked() {
            return Err(WalletUnlockError::NotLocked);
        }

        let config = self.config_repository.config();

        let instruction_result_public_key = config.account_server.instruction_result_public_key.clone().into();

        let remote_instruction = InstructionClient::new(
            pin,
            &self.storage,
            &self.hw_privkey,
            &self.account_provider_client,
            registration_data,
            &config.account_server.base_url,
            &instruction_result_public_key,
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

    use assert_matches::assert_matches;
    use http::StatusCode;
    use mockall::predicate::*;

    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;
    use wallet_common::{
        account::{
            messages::{
                errors::{ErrorData, ErrorType, IncorrectPinData, PinTimeoutData},
                instructions::{CheckPin, Instruction, InstructionResultClaims},
            },
            signed::SequenceNumberComparison,
        },
        jwt::Jwt,
        keys::EcdsaKey,
        utils,
    };

    use crate::{account_provider::AccountProviderResponseError, pin::key::PinKey};

    use super::{
        super::test::{WalletWithMocks, ACCOUNT_SERVER_KEYS},
        *,
    };

    const PIN: &str = "051097";

    // Tests both setting and clearing the lock callback.
    #[tokio::test]
    async fn test_wallet_lock_unlock_callback() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

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
                    .parse_and_verify_with_sub(&(&hw_pubkey).into())
                    .expect("Could not verify check pin challenge request");

                assert_eq!(claims.sequence_number, 1);
                assert_eq!(claims.iss, "wallet");

                Ok(challenge_response)
            });

        // Set up the instruction.
        let wallet_cert = wallet.registration.as_ref().unwrap().wallet_certificate.clone();
        let hw_pubkey = wallet.hw_privkey.verifying_key().await.unwrap();

        let pin_key = PinKey::new(PIN, &wallet.registration.as_ref().unwrap().pin_salt);
        let pin_pubkey = pin_key.verifying_key().unwrap();

        let result_claims = InstructionResultClaims {
            result: (),
            iss: "wallet_unit_test".to_string(),
            iat: jsonwebtoken::get_current_timestamp(),
        };
        let result = Jwt::sign_with_sub(&result_claims, &ACCOUNT_SERVER_KEYS.instruction_result_signing_key)
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
        wallet.unlock(PIN.to_string()).await.expect("Could not unlock wallet");

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

    #[tokio::test]
    async fn test_wallet_unlock_error_not_registered() {
        // Prepare an unregistered wallet
        let mut wallet = WalletWithMocks::new_unregistered().await;

        // Unlocking an unregistered `Wallet` should result in an error.
        let error = wallet
            .unlock(PIN.to_string())
            .await
            .expect_err("Wallet unlocking should have resulted in error");

        assert_matches!(error, WalletUnlockError::NotRegistered);
    }

    #[tokio::test]
    async fn test_wallet_unlock_error_not_locked() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Unlocking an already unlocked `Wallet` should result in an error.
        let error = wallet
            .unlock(PIN.to_string())
            .await
            .expect_err("Wallet unlocking should have resulted in error");

        assert_matches!(error, WalletUnlockError::NotLocked);
    }

    #[tokio::test]
    async fn test_wallet_unlock_error_instruction_server_challenge_404() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        wallet.lock();

        // A 404 response from the account server when requesting the instruction
        // challenge for unlocking should result in an `InstructionError::ServerError`.
        wallet
            .account_provider_client
            .expect_instruction_challenge()
            .return_once(|_, _| Err(AccountProviderResponseError::Status(StatusCode::NOT_FOUND).into()));

        let error = wallet
            .unlock(PIN.to_string())
            .await
            .expect_err("Wallet unlocking should have resulted in error");

        assert_matches!(error, WalletUnlockError::Instruction(InstructionError::ServerError(_)));
    }

    // Helper function for producing unlock errors based
    // on account server instruction responses.
    async fn test_wallet_unlock_error_instruction_response(
        response_error: AccountProviderResponseError,
    ) -> WalletUnlockError {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        wallet.lock();

        wallet
            .account_provider_client
            .expect_instruction_challenge()
            .return_once(|_, _| Ok(utils::random_bytes(32)));

        wallet
            .account_provider_client
            .expect_instruction()
            .return_once(move |_, _: Instruction<CheckPin>| Err(response_error.into()));

        wallet
            .unlock(PIN.to_string())
            .await
            .expect_err("Wallet unlocking should have resulted in error")
    }

    #[tokio::test]
    async fn test_wallet_unlock_error_instruction_incorrect_pin() {
        let error = test_wallet_unlock_error_instruction_response(AccountProviderResponseError::Data(
            StatusCode::FORBIDDEN,
            ErrorData {
                typ: ErrorType::IncorrectPin(IncorrectPinData {
                    attempts_left: 2,
                    is_final_attempt: false,
                }),
                title: "incorrect pin".to_string(),
            },
        ))
        .await;

        assert_matches!(
            error,
            WalletUnlockError::Instruction(InstructionError::IncorrectPin {
                leftover_attempts: 2,
                is_final_attempt: false
            })
        );
    }

    #[tokio::test]
    async fn test_wallet_unlock_error_instruction_timeout() {
        let error = test_wallet_unlock_error_instruction_response(AccountProviderResponseError::Data(
            StatusCode::FORBIDDEN,
            ErrorData {
                typ: ErrorType::PinTimeout(PinTimeoutData { time_left_in_ms: 5000 }),
                title: "pin timeout".to_string(),
            },
        ))
        .await;

        assert_matches!(
            error,
            WalletUnlockError::Instruction(InstructionError::Timeout { timeout_millis: 5000 })
        );
    }

    #[tokio::test]
    async fn test_wallet_unlock_error_instruction_blocked() {
        let error = test_wallet_unlock_error_instruction_response(AccountProviderResponseError::Data(
            StatusCode::UNAUTHORIZED,
            ErrorData {
                typ: ErrorType::AccountBlocked,
                title: "blocked".to_string(),
            },
        ))
        .await;

        assert_matches!(error, WalletUnlockError::Instruction(InstructionError::Blocked));
    }

    #[tokio::test]
    async fn test_wallet_unlock_error_instruction_validation() {
        let error = test_wallet_unlock_error_instruction_response(AccountProviderResponseError::Data(
            StatusCode::FORBIDDEN,
            ErrorData {
                typ: ErrorType::InstructionValidation,
                title: "instruction validation".to_string(),
            },
        ))
        .await;

        assert_matches!(
            error,
            WalletUnlockError::Instruction(InstructionError::InstructionValidation)
        );
    }

    #[tokio::test]
    async fn test_wallet_unlock_error_instruction_server_unexpected() {
        let error = test_wallet_unlock_error_instruction_response(AccountProviderResponseError::Data(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorData {
                typ: ErrorType::Unexpected,
                title: "unexpected error".to_string(),
            },
        ))
        .await;

        assert_matches!(error, WalletUnlockError::Instruction(InstructionError::ServerError(_)));
    }

    #[tokio::test]
    async fn test_wallet_unlock_error_instruction_signing() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        wallet.lock();

        // Have the hardware key signing fail.
        wallet
            .hw_privkey
            .next_private_key_error
            .lock()
            .unwrap()
            .replace(p256::ecdsa::Error::new());

        let error = wallet
            .unlock(PIN.to_string())
            .await
            .expect_err("Wallet unlocking should have resulted in error");

        assert_matches!(error, WalletUnlockError::Instruction(InstructionError::Signing(_)));
    }

    #[tokio::test]
    async fn test_wallet_unlock_error_instruction_result_validation() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        wallet.lock();

        wallet
            .account_provider_client
            .expect_instruction_challenge()
            .return_once(|_, _| Ok(utils::random_bytes(32)));

        // Have the account server sign the instruction result with a key
        // to which the instruction result public key does not belong.
        let result_claims = InstructionResultClaims {
            result: (),
            iss: "wallet_unit_test".to_string(),
            iat: jsonwebtoken::get_current_timestamp(),
        };
        let other_key = SigningKey::random(&mut OsRng);
        let result = Jwt::sign_with_sub(&result_claims, &other_key).await.unwrap();

        wallet
            .account_provider_client
            .expect_instruction()
            .return_once(move |_, _: Instruction<CheckPin>| Ok(result));

        // Unlocking the wallet should now result in a
        // `InstructionError::InstructionResultValidation` error.
        let error = wallet
            .unlock(PIN.to_string())
            .await
            .expect_err("Wallet unlocking should have resulted in error");

        assert_matches!(
            error,
            WalletUnlockError::Instruction(InstructionError::InstructionResultValidation(_))
        );
    }

    #[tokio::test]
    async fn test_wallet_unlock_error_instruction_store() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        wallet.lock();

        // Have the database return an error when fetching the sequence number.
        wallet.storage.get_mut().has_query_error = true;

        // Unlocking the wallet should now result in an
        // `InstructionError::StoreInstructionSequenceNumber` error.
        let error = wallet
            .unlock(PIN.to_string())
            .await
            .expect_err("Wallet unlocking should have resulted in error");

        assert_matches!(
            error,
            WalletUnlockError::Instruction(InstructionError::StoreInstructionSequenceNumber(_))
        );
    }
}
