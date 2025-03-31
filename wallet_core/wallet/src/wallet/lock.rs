use std::sync::Arc;

use tracing::info;
use tracing::instrument;

use wallet_configuration::wallet_config::WalletConfiguration;
use error_category::sentry_capture_error;
use error_category::ErrorCategory;
use platform_support::attested_key::AttestedKeyHolder;
use wallet_account::messages::instructions::CheckPin;
use wallet_common::http::TlsPinningConfig;
use wallet_common::update_policy::VersionState;

pub use crate::lock::LockCallback;
pub use crate::storage::UnlockMethod;

use crate::account_provider::AccountProviderClient;
use crate::errors::ChangePinError;
use crate::errors::StorageError;
use crate::instruction::InstructionError;
use crate::repository::Repository;
use crate::repository::UpdateableRepository;
use crate::storage::Storage;
use crate::storage::UnlockData;
use crate::update_policy::UpdatePolicyError;

use super::Wallet;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum WalletUnlockError {
    #[category(expected)]
    #[error("app version is blocked")]
    VersionBlocked,
    #[error("wallet is not registered")]
    #[category(expected)]
    NotRegistered,
    #[error("wallet is not locked")]
    #[category(expected)]
    NotLocked,
    #[error("can not update setting while locked")]
    #[category(expected)]
    Locked,
    #[error("unlocking with biometrics is not enabled")]
    #[category(expected)]
    BiometricsUnlockingNotEnabled,
    #[error("error sending instruction to Wallet Provider: {0}")]
    Instruction(#[from] InstructionError),
    #[error("could not write or read unlock method to or from database: {0}")]
    UnlockMethodStorage(#[source] StorageError),
    #[error("error finalizing pin change: {0}")]
    ChangePin(#[from] ChangePinError),
    #[error("error fetching update policy: {0}")]
    UpdatePolicy(#[from] UpdatePolicyError),
}

impl<CR, UR, S, AKH, APC, DS, IS, MDS, WIC> Wallet<CR, UR, S, AKH, APC, DS, IS, MDS, WIC>
where
    AKH: AttestedKeyHolder,
{
    pub fn is_locked(&self) -> bool {
        self.lock.is_locked()
    }

    pub fn set_lock_callback(&mut self, callback: LockCallback) -> Option<LockCallback> {
        self.lock.set_lock_callback(callback)
    }

    pub fn clear_lock_callback(&mut self) -> Option<LockCallback> {
        self.lock.clear_lock_callback()
    }

    async fn fetch_unlock_method(&self) -> Result<UnlockMethod, WalletUnlockError>
    where
        S: Storage,
    {
        let method = self
            .storage
            .read()
            .await
            .fetch_data::<UnlockData>()
            .await
            .map_err(WalletUnlockError::UnlockMethodStorage)?
            .map(|data| data.method)
            .unwrap_or_default();

        Ok(method)
    }

    #[instrument(skip_all)]
    pub async fn unlock_method(&self) -> Result<UnlockMethod, WalletUnlockError>
    where
        S: Storage,
    {
        self.fetch_unlock_method().await
    }

    #[instrument(skip_all)]
    pub async fn set_unlock_method(&mut self, method: UnlockMethod) -> Result<(), WalletUnlockError>
    where
        UR: Repository<VersionState>,
        S: Storage,
    {
        info!("Setting unlock method to: {}", method);

        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(WalletUnlockError::VersionBlocked);
        }

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(WalletUnlockError::Locked);
        }

        let data = UnlockData { method };
        self.storage
            .write()
            .await
            .upsert_data(&data)
            .await
            .map_err(WalletUnlockError::UnlockMethodStorage)?;

        Ok(())
    }

    #[instrument(skip_all)]
    pub fn lock(&mut self) {
        self.lock.lock();
    }

    async fn send_check_pin_instruction(&self, pin: String) -> Result<(), WalletUnlockError>
    where
        CR: Repository<Arc<WalletConfiguration>>,
        UR: UpdateableRepository<VersionState, TlsPinningConfig, Error = UpdatePolicyError>,
        S: Storage,
        APC: AccountProviderClient,
        WIC: Default,
    {
        let config = &self.config_repository.get();

        info!("Fetching update policy");
        self.update_policy_repository
            .fetch(&config.update_policy_server.http_config)
            .await?;

        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(WalletUnlockError::VersionBlocked);
        }

        info!("Checking if registered");
        let (attested_key, registration_data) = self
            .registration
            .as_key_and_registration_data()
            .ok_or_else(|| WalletUnlockError::NotRegistered)?;

        let instruction_result_public_key = config.account_server.instruction_result_public_key.as_inner().into();

        let remote_instruction = self
            .new_instruction_client(
                pin,
                Arc::clone(attested_key),
                registration_data.clone(),
                config.account_server.http_config.clone(),
                instruction_result_public_key,
            )
            .await?;

        info!("Sending check pin instruction to Wallet Provider");

        remote_instruction.send(CheckPin).await?;

        Ok(())
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn unlock(&mut self, pin: String) -> Result<(), WalletUnlockError>
    where
        CR: Repository<Arc<WalletConfiguration>>,
        UR: UpdateableRepository<VersionState, TlsPinningConfig, Error = UpdatePolicyError>,
        S: Storage,
        APC: AccountProviderClient,
        WIC: Default,
    {
        info!("Unlocking wallet with pin");

        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(WalletUnlockError::VersionBlocked);
        }

        info!("Checking if locked");
        if !self.lock.is_locked() {
            return Err(WalletUnlockError::NotLocked);
        }

        self.send_check_pin_instruction(pin).await?;

        info!("Unlock instruction successful, unlocking wallet");

        self.lock.unlock();

        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn check_pin(&self, pin: String) -> Result<(), WalletUnlockError>
    where
        CR: Repository<Arc<WalletConfiguration>>,
        UR: UpdateableRepository<VersionState, TlsPinningConfig, Error = UpdatePolicyError>,
        S: Storage,
        APC: AccountProviderClient,
        WIC: Default,
    {
        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(WalletUnlockError::VersionBlocked);
        }

        info!("Checking pin");
        self.send_check_pin_instruction(pin).await
    }

    #[instrument(skip_all)]
    pub async fn unlock_without_pin(&mut self) -> Result<(), WalletUnlockError>
    where
        CR: Repository<Arc<WalletConfiguration>>,
        UR: UpdateableRepository<VersionState, TlsPinningConfig, Error = UpdatePolicyError>,
        S: Storage,
    {
        info!("Unlocking wallet without pin");
        let config = &self.config_repository.get();

        info!("Fetching update policy");
        self.update_policy_repository
            .fetch(&config.update_policy_server.http_config)
            .await?;

        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(WalletUnlockError::VersionBlocked);
        }

        info!("Checking if locked");
        if !self.lock.is_locked() {
            return Err(WalletUnlockError::NotLocked);
        }

        info!("Checking if unlocking with biometrics is enabled");
        if !self.fetch_unlock_method().await?.has_biometrics() {
            return Err(WalletUnlockError::BiometricsUnlockingNotEnabled);
        }

        self.lock.unlock();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use assert_matches::assert_matches;
    use http::StatusCode;
    use mockall::predicate::*;
    use p256::ecdsa::SigningKey;
    use parking_lot::Mutex;
    use rand_core::OsRng;
    use rstest::rstest;

    use apple_app_attest::AssertionCounter;
    use crypto::utils;
    use jwt::Jwt;
    use platform_support::attested_key::AttestedKey;
    use wallet_account::messages::errors::AccountError;
    use wallet_account::messages::errors::IncorrectPinData;
    use wallet_account::messages::errors::PinTimeoutData;
    use wallet_account::messages::instructions::CheckPin;
    use wallet_account::messages::instructions::Instruction;
    use wallet_account::messages::instructions::InstructionResultClaims;
    use wallet_account::signed::SequenceNumberComparison;

    use crate::account_provider::AccountProviderResponseError;
    use crate::pin::key::PinKey;
    use crate::storage::InstructionData;
    use crate::storage::KeyedData;

    use super::super::test::WalletDeviceVendor;
    use super::super::test::WalletWithMocks;
    use super::super::test::ACCOUNT_SERVER_KEYS;
    use super::super::WalletRegistration;
    use super::*;

    const PIN: &str = "051097";

    #[tokio::test]
    #[rstest]
    async fn test_wallet_lock_unlock(
        #[values(WalletDeviceVendor::Apple, WalletDeviceVendor::Google)] vendor: WalletDeviceVendor,
    ) {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(vendor);

        // Wrap a `Vec<bool>` in both a `Mutex` and `Arc`,
        // so we can write to it from the closure.
        let is_locked_vec = Arc::new(Mutex::new(Vec::<bool>::with_capacity(1)));
        let callback_is_locked_vec = Arc::clone(&is_locked_vec);

        // Set the lock callback on the `Wallet`,
        // which should immediately be called exactly once.
        wallet.set_lock_callback(Box::new(move |is_locked| callback_is_locked_vec.lock().push(is_locked)));

        // Lock the `Wallet`, then lock it again.
        wallet.lock();
        wallet.lock();

        // Mock the responses of the account server for both the instruction challenge
        // and the actual instruction and check the contents of those messages.
        let challenge = utils::random_bytes(32);

        // Set up the instruction challenge.
        let challenge_response = challenge.clone();
        let (attested_key, registration_data) = wallet.registration.as_key_and_registration_data().unwrap();
        let wallet_cert = registration_data.wallet_certificate.clone();
        let wallet_id = registration_data.wallet_id.clone();

        let (attested_public_key, app_identifier_and_next_counter) = match attested_key.as_ref() {
            AttestedKey::Apple(key) => (
                *key.verifying_key(),
                Some((key.app_identifier.clone(), key.next_counter())),
            ),
            AttestedKey::Google(key) => (*key.verifying_key(), None),
        };

        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_instruction_challenge()
            .with(
                eq(wallet.config_repository.get().account_server.http_config.clone()),
                always(),
            )
            .return_once(move |_, challenge_request| {
                assert_eq!(challenge_request.certificate.0, wallet_cert.0);

                match app_identifier_and_next_counter {
                    Some((app_identifier, next_counter)) => {
                        challenge_request
                            .request
                            .parse_and_verify_apple(
                                &wallet_id,
                                SequenceNumberComparison::EqualTo(1),
                                &attested_public_key,
                                &app_identifier,
                                AssertionCounter::from(*next_counter - 1),
                            )
                            .expect("challenge request should be valid for Apple attested key");
                    }
                    None => {
                        challenge_request
                            .request
                            .parse_and_verify_google(
                                &wallet_id,
                                SequenceNumberComparison::EqualTo(1),
                                &attested_public_key,
                            )
                            .expect("challenge request should be valid for Google attested key");
                    }
                }

                Ok(challenge_response)
            });

        // Set up the instruction.
        let wallet_cert = registration_data.wallet_certificate.clone();

        let app_identifier_and_next_counter = match attested_key.as_ref() {
            AttestedKey::Apple(key) => Some((key.app_identifier.clone(), key.next_counter())),
            AttestedKey::Google(_) => None,
        };

        let pin_key = PinKey {
            pin: PIN,
            salt: &registration_data.pin_salt,
        };
        let pin_pubkey = pin_key.verifying_key().unwrap();

        let result_claims = InstructionResultClaims {
            result: (),
            iss: "wallet_unit_test".to_string(),
            iat: jsonwebtoken::get_current_timestamp(),
        };
        let result = Jwt::sign_with_sub(&result_claims, &ACCOUNT_SERVER_KEYS.instruction_result_signing_key)
            .await
            .unwrap();

        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_instruction()
            .with(
                eq(wallet.config_repository.get().account_server.http_config.clone()),
                always(),
            )
            .return_once(move |_, instruction: Instruction<CheckPin>| {
                assert_eq!(instruction.certificate.0, wallet_cert.0);

                match app_identifier_and_next_counter {
                    Some((app_identifier, next_counter)) => {
                        instruction
                            .instruction
                            .parse_and_verify_apple(
                                &challenge,
                                SequenceNumberComparison::LargerThan(1),
                                &attested_public_key,
                                &app_identifier,
                                AssertionCounter::from(*next_counter - 1),
                                &pin_pubkey,
                            )
                            .expect("check pin instruction should be valid for Apple attested key");
                    }
                    None => {
                        instruction
                            .instruction
                            .parse_and_verify_google(
                                &challenge,
                                SequenceNumberComparison::LargerThan(1),
                                &attested_public_key,
                                &pin_pubkey,
                            )
                            .expect("check pin instruction should be valid for Google attested key");
                    }
                }

                Ok(result)
            });

        // Unlock the `Wallet` with the PIN.
        wallet.unlock(PIN.to_owned()).await.expect("Could not unlock wallet");

        // Infer that the closure is still alive by counting the `Arc` references.
        assert_eq!(Arc::strong_count(&is_locked_vec), 2);

        // Test the contents of the `Vec<bool>`.
        {
            let is_locked_vec = is_locked_vec.lock();

            assert_eq!(*is_locked_vec, vec![false, true, false]);
        }

        // Clear the lock callback on the `Wallet.`
        wallet.clear_lock_callback();

        // Infer that the closure is now dropped by counting the `Arc` references.
        assert_eq!(Arc::strong_count(&is_locked_vec), 1);

        // Lock the `Wallet` again.
        wallet.lock();

        // Test that the callback was not called.
        assert_eq!(is_locked_vec.lock().len(), 3);
    }

    #[tokio::test]
    async fn test_wallet_unlock_error_not_registered() {
        // Prepare an unregistered wallet
        let mut wallet = WalletWithMocks::new_unregistered(WalletDeviceVendor::Apple);

        // Unlocking an unregistered `Wallet` should result in an error.
        let error = wallet
            .unlock(PIN.to_owned())
            .await
            .expect_err("Wallet unlocking should have resulted in error");

        assert_matches!(error, WalletUnlockError::NotRegistered);
    }

    #[tokio::test]
    async fn test_wallet_unlock_error_not_locked() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Unlocking an already unlocked `Wallet` should result in an error.
        let error = wallet
            .unlock(PIN.to_owned())
            .await
            .expect_err("Wallet unlocking should have resulted in error");

        assert_matches!(error, WalletUnlockError::NotLocked);
    }

    #[tokio::test]
    async fn test_wallet_unlock_error_instruction_server_challenge_404() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        wallet.lock();

        // A 404 response from the account server when requesting the instruction
        // challenge for unlocking should result in an `InstructionError::ServerError`.
        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_instruction_challenge()
            .return_once(|_, _| Err(AccountProviderResponseError::Status(StatusCode::NOT_FOUND).into()));

        let error = wallet
            .unlock(PIN.to_owned())
            .await
            .expect_err("Wallet unlocking should have resulted in error");

        assert_matches!(error, WalletUnlockError::Instruction(InstructionError::ServerError(_)));
    }

    // Helper function for producing unlock errors based
    // on account server instruction responses.
    async fn test_wallet_unlock_error_instruction_response(
        response_error: AccountProviderResponseError,
    ) -> WalletUnlockError {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        wallet.lock();

        let account_provider_client = Arc::get_mut(&mut wallet.account_provider_client).unwrap();

        account_provider_client
            .expect_instruction_challenge()
            .return_once(|_, _| Ok(utils::random_bytes(32)));

        account_provider_client
            .expect_instruction()
            .return_once(move |_, _: Instruction<CheckPin>| Err(response_error.into()));

        wallet
            .unlock(PIN.to_owned())
            .await
            .expect_err("Wallet unlocking should have resulted in error")
    }

    #[tokio::test]
    async fn test_wallet_unlock_error_instruction_incorrect_pin() {
        let error = test_wallet_unlock_error_instruction_response(AccountProviderResponseError::Account(
            AccountError::IncorrectPin(IncorrectPinData {
                attempts_left_in_round: 2,
                is_final_round: false,
            }),
            None,
        ))
        .await;

        assert_matches!(
            error,
            WalletUnlockError::Instruction(InstructionError::IncorrectPin {
                attempts_left_in_round: 2,
                is_final_round: false
            })
        );
    }

    #[tokio::test]
    async fn test_wallet_unlock_error_instruction_timeout() {
        let error = test_wallet_unlock_error_instruction_response(AccountProviderResponseError::Account(
            AccountError::PinTimeout(PinTimeoutData { time_left_in_ms: 5000 }),
            None,
        ))
        .await;

        assert_matches!(
            error,
            WalletUnlockError::Instruction(InstructionError::Timeout { timeout_millis: 5000 })
        );
    }

    #[tokio::test]
    async fn test_wallet_unlock_error_instruction_blocked() {
        let error = test_wallet_unlock_error_instruction_response(AccountProviderResponseError::Account(
            AccountError::AccountBlocked,
            None,
        ))
        .await;

        assert_matches!(error, WalletUnlockError::Instruction(InstructionError::Blocked));
    }

    #[tokio::test]
    async fn test_wallet_unlock_error_instruction_validation() {
        let error = test_wallet_unlock_error_instruction_response(AccountProviderResponseError::Account(
            AccountError::InstructionValidation,
            None,
        ))
        .await;

        assert_matches!(
            error,
            WalletUnlockError::Instruction(InstructionError::InstructionValidation)
        );
    }

    #[tokio::test]
    async fn test_wallet_unlock_error_instruction_server_unexpected() {
        let error = test_wallet_unlock_error_instruction_response(AccountProviderResponseError::Account(
            AccountError::Unexpected,
            None,
        ))
        .await;

        assert_matches!(error, WalletUnlockError::Instruction(InstructionError::ServerError(_)));
    }

    #[tokio::test]
    async fn test_wallet_unlock_error_instruction_signing() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        wallet.lock();

        // Have the hardware key signing fail.
        match &mut wallet.registration {
            WalletRegistration::Registered { attested_key, .. } => match Arc::get_mut(attested_key).unwrap() {
                AttestedKey::Apple(attested_key) => attested_key.has_error = true,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }

        let error = wallet
            .unlock(PIN.to_owned())
            .await
            .expect_err("Wallet unlocking should have resulted in error");

        assert_matches!(error, WalletUnlockError::Instruction(InstructionError::Signing(_)));
    }

    #[tokio::test]
    async fn test_wallet_unlock_error_instruction_result_validation() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        wallet.lock();

        let account_provider_client = Arc::get_mut(&mut wallet.account_provider_client).unwrap();
        account_provider_client
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

        account_provider_client
            .expect_instruction()
            .return_once(move |_, _: Instruction<CheckPin>| Ok(result));

        // Unlocking the wallet should now result in a
        // `InstructionError::InstructionResultValidation` error.
        let error = wallet
            .unlock(PIN.to_owned())
            .await
            .expect_err("Wallet unlocking should have resulted in error");

        assert_matches!(
            error,
            WalletUnlockError::Instruction(InstructionError::InstructionResultValidation(_))
        );
    }

    #[tokio::test]
    async fn test_wallet_unlock_error_instruction_store() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        wallet.lock();

        // Have the database return an error when fetching the sequence number.
        wallet.storage.write().await.set_keyed_data_error(InstructionData::KEY);

        // Unlocking the wallet should now result in an
        // `InstructionError::StoreInstructionSequenceNumber` error.
        let error = wallet
            .unlock(PIN.to_owned())
            .await
            .expect_err("Wallet unlocking should have resulted in error");

        assert_matches!(
            error,
            WalletUnlockError::Instruction(InstructionError::StoreInstructionSequenceNumber(_))
        );
    }
}
