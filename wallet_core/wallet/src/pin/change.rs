use std::{error::Error, future::Future};

use p256::ecdsa::VerifyingKey;
use serde::{Deserialize, Serialize};

use error_category::ErrorCategory;
use wallet_common::{
    account::messages::auth::WalletCertificate,
    jwt::{EcdsaDecodingKey, JwtError},
};

use crate::{
    errors::{InstructionError, PinValidationError, StorageError},
    pin::key::{self as pin_key},
    validate_pin,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum State {
    Begin,
    Commit,
    Rollback,
}

pub trait ChangePinClientError: std::error::Error {
    /// Classify error as network error, meaning that something went wrong with networking,
    /// so that no statement can be done about the status of the server. Implementations
    /// of this trait should implement this conservatively, meaning that in cases of uncertainty
    /// this should return `true`.
    fn is_network_error(&self) -> bool;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock(type Error = mock::ChangePinClientTestError;))]
pub trait ChangePinClient {
    type Error: ChangePinClientError;
    async fn start_new_pin(
        &self,
        old_pin: &str,
        new_pin: &str,
        new_pin_salt: &[u8],
    ) -> Result<WalletCertificate, Self::Error>;
    async fn commit_new_pin(&self, new_pin: &str) -> Result<(), Self::Error>;
    async fn rollback_new_pin(&self, old_pin: &str) -> Result<(), Self::Error>;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait ChangePinStorage {
    async fn get_change_pin_state(&self) -> Result<Option<State>, StorageError>;
    async fn store_change_pin_state(&self, state: State) -> Result<(), StorageError>;
    async fn clear_change_pin_state(&self) -> Result<(), StorageError>;

    async fn change_pin(
        &self,
        wallet_id: String,
        new_pin_salt: Vec<u8>,
        new_pin_certificate: WalletCertificate,
    ) -> Result<(), StorageError>;
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum ChangePinError {
    #[error("wallet is not registered")]
    #[category(expected)]
    NotRegistered,
    #[error("wallet is locked")]
    #[category(expected)]
    Locked,
    #[error("pin_change transaction already in progress")]
    #[category(expected)]
    ChangePinAlreadyInProgress,
    #[error("no pin_change transaction in progress")]
    #[category(expected)]
    NoChangePinInProgress,
    #[error("the new PIN does not adhere to requirements: {0}")]
    PinValidation(#[from] PinValidationError),
    #[error("instruction failed: {0}")]
    Instruction(#[from] InstructionError),
    #[error("storage error: {0}")]
    Storage(#[from] StorageError),
    #[error("could not get hardware public key: {0}")]
    #[category(pd)]
    HardwarePublicKey(#[source] Box<dyn Error + Send + Sync>),
    #[error("could not validate new registration certificate received from Wallet Provider: {0}")]
    CertificateValidation(#[source] JwtError),
    #[error(
        "public key in new registration certificate received from Wallet Provider does not match hardware public key"
    )]
    #[category(critical)]
    PublicKeyMismatch,
    #[error(
        "wallet ID in new registration certificate received from Wallet Provider does not match current wallet ID"
    )]
    #[category(critical)]
    WalletIdMismatch,
}

pub type ChangePinResult<T> = Result<T, ChangePinError>;

pub struct ChangePinSession<'a, C, S> {
    client: &'a C,
    storage: &'a S,
    retries: u8,
}

impl<'a, C, S> ChangePinSession<'a, C, S> {
    pub fn new(client: &'a C, storage: &'a S, retries: u8) -> Self {
        Self {
            client,
            storage,
            retries,
        }
    }

    // Perform the same sanity checks as during registration, with the addition of checking the received wallet_id.
    pub fn validate_certificate(
        certificate: &WalletCertificate,
        certificate_public_key: &EcdsaDecodingKey,
        hw_pubkey: &VerifyingKey,
        wallet_id: &str,
    ) -> ChangePinResult<()> {
        let cert_claims = certificate
            .parse_and_verify_with_sub(certificate_public_key)
            .map_err(ChangePinError::CertificateValidation)?;

        if &cert_claims.hw_pubkey.0 != hw_pubkey {
            return Err(ChangePinError::PublicKeyMismatch);
        }

        if cert_claims.wallet_id != wallet_id {
            return Err(ChangePinError::WalletIdMismatch);
        }

        Ok(())
    }
}

impl<'a, C, S, E> ChangePinSession<'a, C, S>
where
    C: ChangePinClient<Error = E>,
    S: ChangePinStorage,
    E: ChangePinClientError + Into<ChangePinError>,
{
    /// Perform [`operation`] and retry a number of times when a network error occurs.
    async fn with_retries<F, Fut>(&self, operation_name: &str, operation: F) -> ChangePinResult<()>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<(), E>>,
    {
        tracing::info!("{operation_name} change PIN transaction");

        let mut retries = 0;
        loop {
            retries += 1;
            let result = operation();
            match result.await {
                Ok(()) => break,
                Err(error) if error.is_network_error() => {
                    if retries >= self.retries {
                        tracing::warn!("Network error during {operation_name}, aborting: {error:?}");
                        return Err(error.into());
                    } else {
                        tracing::warn!("Network error during {operation_name}, trying again: {error:?}");
                    }
                }
                Err(error) => {
                    tracing::error!("Error during {operation_name}: {error:?}");
                    return Err(error.into());
                }
            }
        }

        tracing::info!("{operation_name} successful");
        Ok(())
    }

    async fn commit(&self, new_pin: String) -> ChangePinResult<()> {
        self.with_retries("commit", || async { self.client.commit_new_pin(&new_pin).await })
            .await?;
        self.storage.clear_change_pin_state().await?;
        Ok(())
    }

    async fn rollback(&self, old_pin: String) -> ChangePinResult<()> {
        self.with_retries("rollback", || async { self.client.rollback_new_pin(&old_pin).await })
            .await?;
        self.storage.clear_change_pin_state().await?;
        Ok(())
    }

    pub async fn begin_change_pin(
        &self,
        certificate_public_key: &EcdsaDecodingKey,
        hw_pubkey: &VerifyingKey,
        wallet_id: String,
        old_pin: String,
        new_pin: String,
    ) -> ChangePinResult<(Vec<u8>, WalletCertificate)> {
        tracing::info!("Start change PIN transaction");

        tracing::info!("Ensure no PIN change is in progress");
        if self.storage.get_change_pin_state().await?.is_some() {
            return Err(ChangePinError::ChangePinAlreadyInProgress);
        }

        tracing::info!("Validating new PIN");
        // Make sure the new PIN adheres to the requirements.
        validate_pin(&new_pin)?;

        let new_pin_salt = pin_key::new_pin_salt();

        self.storage.store_change_pin_state(State::Begin).await?;

        let start_result = self
            .client
            .start_new_pin(&old_pin, &new_pin, &new_pin_salt)
            .await
            .map_err(|error| {
                let is_network_error = error.is_network_error();

                // Initiate a rollback if the error is detected to be a network error.
                (error.into(), is_network_error)
            })
            .and_then(|new_pin_certificate| {
                // If the received certificate does not validate, initiate a rollback of the PIN change.
                Self::validate_certificate(&new_pin_certificate, certificate_public_key, hw_pubkey, &wallet_id)
                    .map_err(|error| (error, true))?;

                Ok(new_pin_certificate)
            });

        match start_result {
            Ok(new_pin_certificate) => {
                self.storage.store_change_pin_state(State::Commit).await?;
                self.storage
                    .change_pin(wallet_id, new_pin_salt.clone(), new_pin_certificate.clone())
                    .await?;
                Ok((new_pin_salt, new_pin_certificate))
            }
            Err((error, true)) => {
                self.storage.store_change_pin_state(State::Rollback).await?;
                Err(error)
            }
            Err((error, false)) => {
                self.storage.clear_change_pin_state().await?;
                Err(error)
            }
        }
    }

    pub async fn continue_change_pin(&self, pin: String) -> ChangePinResult<()> {
        tracing::info!("Continue change PIN transaction");

        match self.storage.get_change_pin_state().await? {
            None => Err(ChangePinError::NoChangePinInProgress),
            Some(State::Commit) => self.commit(pin).await,
            Some(State::Begin) | Some(State::Rollback) => self.rollback(pin).await,
        }
    }
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use super::*;

    #[derive(Debug, thiserror::Error)]
    #[error("")]
    pub struct ChangePinClientTestError {
        pub is_network: bool,
    }

    impl ChangePinClientError for ChangePinClientTestError {
        fn is_network_error(&self) -> bool {
            self.is_network
        }
    }

    impl From<ChangePinClientTestError> for ChangePinError {
        fn from(value: ChangePinClientTestError) -> Self {
            if value.is_network_error() {
                Self::Instruction(InstructionError::Timeout { timeout_millis: 15 })
            } else {
                Self::Instruction(InstructionError::InstructionValidation)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;
    use mockall::predicate::eq;

    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;

    use wallet_common::{
        account::messages::auth::{WalletCertificate, WalletCertificateClaims},
        jwt::Jwt,
        utils,
    };

    use super::*;

    use crate::{
        errors::InstructionError,
        pin::change::{
            mock::ChangePinClientTestError, ChangePinError, MockChangePinClient, MockChangePinStorage, State,
        },
    };

    async fn create_wallet_certificate() -> (WalletCertificate, EcdsaDecodingKey, VerifyingKey, String) {
        let certificate_signing_key = SigningKey::random(&mut OsRng);
        let hw_privkey = SigningKey::random(&mut OsRng);
        let certificate_public_key = certificate_signing_key.verifying_key().into();
        let hw_pubkey = *hw_privkey.verifying_key();
        let wallet_id = utils::random_string(32);

        let certificate_claims = WalletCertificateClaims {
            wallet_id: wallet_id.clone(),
            hw_pubkey: hw_pubkey.into(),
            // The hash does not need to be value for testing.
            pin_pubkey_hash: utils::random_bytes(32),
            version: 0,
            iss: "pin_change_unit_test".to_string(),
            iat: jsonwebtoken::get_current_timestamp(),
        };

        let certificate = Jwt::sign_with_sub(&certificate_claims, &certificate_signing_key)
            .await
            .unwrap();

        (certificate, certificate_public_key, hw_pubkey, wallet_id)
    }

    #[tokio::test]
    async fn begin_change_pin_success() {
        let (certificate, certificate_public_key, hw_pubkey, wallet_id) = create_wallet_certificate().await;
        let returned_certificate = certificate.clone();

        let mut change_pin_client = MockChangePinClient::new();
        change_pin_client
            .expect_start_new_pin()
            .return_once(|_, _, _| Ok(returned_certificate));

        let mut change_pin_storage = MockChangePinStorage::new();
        change_pin_storage
            .expect_get_change_pin_state()
            .return_once(|| Ok(None));
        change_pin_storage
            .expect_store_change_pin_state()
            .with(eq(State::Begin))
            .return_once(|_| Ok(()));
        change_pin_storage
            .expect_store_change_pin_state()
            .with(eq(State::Commit))
            .return_once(|_| Ok(()));
        change_pin_storage.expect_change_pin().return_once(|_, _, _| Ok(()));

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, 2);

        let (new_pin_salt, new_wallet_certificate) = change_pin_session
            .begin_change_pin(
                &certificate_public_key,
                &hw_pubkey,
                wallet_id,
                "000111".to_string(),
                "123789".to_string(),
            )
            .await
            .expect("begin changing PIN should succeed");

        assert!(!new_pin_salt.is_empty());
        assert_eq!(new_wallet_certificate.0, certificate.0);
    }

    #[tokio::test]
    async fn begin_change_pin_network_error() {
        let (_, certificate_public_key, hw_pubkey, wallet_id) = create_wallet_certificate().await;

        let mut change_pin_client = MockChangePinClient::new();
        // return a network error
        change_pin_client
            .expect_start_new_pin()
            .returning(|_, _, _| Err(ChangePinClientTestError { is_network: true }));

        let mut change_pin_storage = MockChangePinStorage::new();
        change_pin_storage
            .expect_get_change_pin_state()
            .times(1)
            .returning(|| Ok(None));
        change_pin_storage
            .expect_store_change_pin_state()
            .with(eq(State::Begin))
            .returning(|_| Ok(()));
        change_pin_storage
            .expect_store_change_pin_state()
            .with(eq(State::Rollback))
            .returning(|_| Ok(()));

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, 2);

        let actual = change_pin_session
            .begin_change_pin(
                &certificate_public_key,
                &hw_pubkey,
                wallet_id,
                "000111".to_string(),
                "123789".to_string(),
            )
            .await;

        assert_matches!(
            actual,
            Err(ChangePinError::Instruction(InstructionError::Timeout {
                timeout_millis: 15
            }))
        );
    }

    #[tokio::test]
    async fn begin_change_pin_instruction_error() {
        let (_, certificate_public_key, hw_pubkey, wallet_id) = create_wallet_certificate().await;

        let mut change_pin_client = MockChangePinClient::new();
        change_pin_client
            .expect_start_new_pin()
            .returning(|_, _, _| Err(ChangePinClientTestError { is_network: false }));

        let mut change_pin_storage = MockChangePinStorage::new();
        change_pin_storage
            .expect_get_change_pin_state()
            .times(1)
            .returning(|| Ok(None));
        change_pin_storage
            .expect_store_change_pin_state()
            .with(eq(State::Begin))
            .returning(|_| Ok(()));
        change_pin_storage
            .expect_clear_change_pin_state()
            .times(1)
            .returning(|| Ok(()));

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, 2);

        let actual = change_pin_session
            .begin_change_pin(
                &certificate_public_key,
                &hw_pubkey,
                wallet_id,
                "000111".to_string(),
                "123789".to_string(),
            )
            .await;

        assert_matches!(
            actual,
            Err(ChangePinError::Instruction(InstructionError::InstructionValidation))
        );
    }

    #[tokio::test]
    async fn begin_change_pin_error_already_in_progress() {
        let (_, certificate_public_key, hw_pubkey, wallet_id) = create_wallet_certificate().await;

        let change_pin_client = MockChangePinClient::new();

        let mut change_pin_storage = MockChangePinStorage::new();
        change_pin_storage
            .expect_get_change_pin_state()
            .times(1)
            .returning(|| Ok(Some(State::Begin)));

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, 2);

        let actual = change_pin_session
            .begin_change_pin(
                &certificate_public_key,
                &hw_pubkey,
                wallet_id,
                "000111".to_string(),
                "123789".to_string(),
            )
            .await;

        assert_matches!(actual, Err(ChangePinError::ChangePinAlreadyInProgress));
    }

    async fn setup_change_pin_certificate_sanity_check_test() -> (
        MockChangePinClient,
        MockChangePinStorage,
        EcdsaDecodingKey,
        VerifyingKey,
        String,
    ) {
        let (certificate, certificate_public_key, hw_pubkey, wallet_id) = create_wallet_certificate().await;
        let returned_certificate = certificate.clone();

        let mut change_pin_client = MockChangePinClient::new();
        change_pin_client
            .expect_start_new_pin()
            .return_once(|_, _, _| Ok(returned_certificate));

        let mut change_pin_storage = MockChangePinStorage::new();
        change_pin_storage
            .expect_get_change_pin_state()
            .times(1)
            .returning(|| Ok(None));
        change_pin_storage
            .expect_store_change_pin_state()
            .with(eq(State::Begin))
            .returning(|_| Ok(()));
        change_pin_storage
            .expect_store_change_pin_state()
            .with(eq(State::Rollback))
            .returning(|_| Ok(()));

        (
            change_pin_client,
            change_pin_storage,
            certificate_public_key,
            hw_pubkey,
            wallet_id,
        )
    }

    #[tokio::test]
    async fn begin_change_pin_certificate_validation_error() {
        let (change_pin_client, change_pin_storage, _, hw_pubkey, wallet_id) =
            setup_change_pin_certificate_sanity_check_test().await;
        let other_certificate_public_key = SigningKey::random(&mut OsRng).verifying_key().into();

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, 2);

        // Validation with a different certificate public key should fail.
        let error = change_pin_session
            .begin_change_pin(
                &other_certificate_public_key,
                &hw_pubkey,
                wallet_id,
                "000111".to_string(),
                "123789".to_string(),
            )
            .await
            .expect_err("begin changing PIN should fail");

        assert_matches!(error, ChangePinError::CertificateValidation(_));
    }

    #[tokio::test]
    async fn begin_change_pin_public_key_mismatch_error() {
        let (change_pin_client, change_pin_storage, other_certificate_public_key, _, wallet_id) =
            setup_change_pin_certificate_sanity_check_test().await;
        let other_hw_pubkey = *SigningKey::random(&mut OsRng).verifying_key();

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, 2);

        // Validation with a different hardware public key should fail.
        let error = change_pin_session
            .begin_change_pin(
                &other_certificate_public_key,
                &other_hw_pubkey,
                wallet_id,
                "000111".to_string(),
                "123789".to_string(),
            )
            .await
            .expect_err("begin changing PIN should fail");

        assert_matches!(error, ChangePinError::PublicKeyMismatch);
    }

    #[tokio::test]
    async fn begin_change_pin_wallet_id_mismatch_error() {
        let (change_pin_client, change_pin_storage, other_certificate_public_key, hw_pubkey, _) =
            setup_change_pin_certificate_sanity_check_test().await;

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, 2);

        // Validation with a different wallet ID should fail.
        let error = change_pin_session
            .begin_change_pin(
                &other_certificate_public_key,
                &hw_pubkey,
                "other_wallet_id".to_string(),
                "000111".to_string(),
                "123789".to_string(),
            )
            .await
            .expect_err("begin changing PIN should fail");

        assert_matches!(error, ChangePinError::WalletIdMismatch);
    }

    #[tokio::test]
    async fn continue_change_pin_commit_success() {
        let mut change_pin_client = MockChangePinClient::new();
        change_pin_client.expect_commit_new_pin().times(1).returning(|_| Ok(()));

        let mut change_pin_storage = MockChangePinStorage::new();
        change_pin_storage
            .expect_get_change_pin_state()
            .times(1)
            .returning(|| Ok(Some(State::Commit)));
        change_pin_storage
            .expect_clear_change_pin_state()
            .times(1)
            .returning(|| Ok(()));

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, 2);

        let actual = change_pin_session.continue_change_pin("123789".to_string()).await;

        assert_matches!(actual, Ok(()));
    }

    #[tokio::test]
    async fn continue_change_pin_commit_network_error() {
        let mut change_pin_client = MockChangePinClient::new();
        change_pin_client
            .expect_commit_new_pin()
            .times(2)
            .returning(|_| Err(ChangePinClientTestError { is_network: true }));

        let mut change_pin_storage = MockChangePinStorage::new();
        change_pin_storage
            .expect_get_change_pin_state()
            .times(1)
            .returning(|| Ok(Some(State::Commit)));

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, 2);

        let actual = change_pin_session.continue_change_pin("123789".to_string()).await;

        assert_matches!(
            actual,
            Err(ChangePinError::Instruction(InstructionError::Timeout {
                timeout_millis: 15
            }))
        );
    }

    #[tokio::test]
    async fn continue_change_pin_commit_one_network_error() {
        let mut change_pin_client = MockChangePinClient::new();
        // First return a network error
        change_pin_client
            .expect_commit_new_pin()
            .times(1)
            .returning(|_| Err(ChangePinClientTestError { is_network: true }));
        // Then return successfully
        change_pin_client.expect_commit_new_pin().times(1).returning(|_| Ok(()));

        let mut change_pin_storage = MockChangePinStorage::new();
        change_pin_storage
            .expect_get_change_pin_state()
            .times(1)
            .returning(|| Ok(Some(State::Commit)));
        change_pin_storage
            .expect_clear_change_pin_state()
            .times(1)
            .returning(|| Ok(()));

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, 2);

        let actual = change_pin_session.continue_change_pin("123789".to_string()).await;

        assert_matches!(actual, Ok(()));
    }

    #[tokio::test]
    async fn continue_change_pin_commit_error() {
        let mut change_pin_client = MockChangePinClient::new();
        change_pin_client
            .expect_commit_new_pin()
            .times(1)
            .returning(|_| Err(ChangePinClientTestError { is_network: false }));

        let mut change_pin_storage = MockChangePinStorage::new();
        change_pin_storage
            .expect_get_change_pin_state()
            .times(1)
            .returning(|| Ok(Some(State::Commit)));

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, 2);

        let actual = change_pin_session.continue_change_pin("123789".to_string()).await;

        assert_matches!(
            actual,
            Err(ChangePinError::Instruction(InstructionError::InstructionValidation))
        );
    }

    #[tokio::test]
    async fn continue_change_pin_rollback_success() {
        let mut change_pin_client = MockChangePinClient::new();
        change_pin_client
            .expect_rollback_new_pin()
            .times(1)
            .returning(|_| Ok(()));

        let mut change_pin_storage = MockChangePinStorage::new();
        change_pin_storage
            .expect_get_change_pin_state()
            .times(1)
            .returning(|| Ok(Some(State::Rollback)));
        change_pin_storage
            .expect_clear_change_pin_state()
            .times(1)
            .returning(|| Ok(()));

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, 2);

        let actual = change_pin_session.continue_change_pin("123789".to_string()).await;

        assert_matches!(actual, Ok(()));
    }

    #[tokio::test]
    async fn continue_change_pin_rollback_network_error() {
        let mut change_pin_client = MockChangePinClient::new();
        change_pin_client
            .expect_rollback_new_pin()
            .times(2)
            .returning(|_| Err(ChangePinClientTestError { is_network: true }));

        let mut change_pin_storage = MockChangePinStorage::new();
        change_pin_storage
            .expect_get_change_pin_state()
            .times(1)
            .returning(|| Ok(Some(State::Rollback)));

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, 2);

        let actual = change_pin_session.continue_change_pin("123789".to_string()).await;

        assert_matches!(
            actual,
            Err(ChangePinError::Instruction(InstructionError::Timeout {
                timeout_millis: 15
            }))
        );
    }

    #[tokio::test]
    async fn continue_change_pin_rollback_one_network_error() {
        let mut change_pin_client = MockChangePinClient::new();
        // First return a network error
        change_pin_client
            .expect_rollback_new_pin()
            .times(1)
            .returning(|_| Err(ChangePinClientTestError { is_network: true }));
        // Then return successfully
        change_pin_client
            .expect_rollback_new_pin()
            .times(1)
            .returning(|_| Ok(()));

        let mut change_pin_storage = MockChangePinStorage::new();
        change_pin_storage
            .expect_get_change_pin_state()
            .times(1)
            .returning(|| Ok(Some(State::Rollback)));
        change_pin_storage
            .expect_clear_change_pin_state()
            .times(1)
            .returning(|| Ok(()));

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, 2);

        let actual = change_pin_session.continue_change_pin("123789".to_string()).await;

        assert_matches!(actual, Ok(()));
    }

    #[tokio::test]
    async fn continue_change_pin_rollback_error() {
        let mut change_pin_client = MockChangePinClient::new();
        change_pin_client
            .expect_rollback_new_pin()
            .times(1)
            .returning(|_| Err(ChangePinClientTestError { is_network: false }));

        let mut change_pin_storage = MockChangePinStorage::new();
        change_pin_storage
            .expect_get_change_pin_state()
            .times(1)
            .returning(|| Ok(Some(State::Rollback)));

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, 2);

        let actual = change_pin_session.continue_change_pin("123789".to_string()).await;

        assert_matches!(
            actual,
            Err(ChangePinError::Instruction(InstructionError::InstructionValidation))
        );
    }

    #[tokio::test]
    async fn continue_change_pin_begin_success() {
        let mut change_pin_client = MockChangePinClient::new();
        change_pin_client
            .expect_rollback_new_pin()
            .times(1)
            .returning(|_| Ok(()));

        let mut change_pin_storage = MockChangePinStorage::new();
        change_pin_storage
            .expect_get_change_pin_state()
            .times(1)
            .returning(|| Ok(Some(State::Begin)));
        change_pin_storage
            .expect_clear_change_pin_state()
            .times(1)
            .returning(|| Ok(()));

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, 2);

        let actual = change_pin_session.continue_change_pin("123789".to_string()).await;

        assert_matches!(actual, Ok(()));
    }

    #[tokio::test]
    async fn continue_change_pin_begin_network_error() {
        let mut change_pin_client = MockChangePinClient::new();
        change_pin_client
            .expect_rollback_new_pin()
            .times(2)
            .returning(|_| Err(ChangePinClientTestError { is_network: true }));

        let mut change_pin_storage = MockChangePinStorage::new();
        change_pin_storage
            .expect_get_change_pin_state()
            .times(1)
            .returning(|| Ok(Some(State::Begin)));

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, 2);

        let actual = change_pin_session.continue_change_pin("123789".to_string()).await;

        assert_matches!(
            actual,
            Err(ChangePinError::Instruction(InstructionError::Timeout {
                timeout_millis: 15
            }))
        );
    }

    #[tokio::test]
    async fn continue_change_pin_begin_one_network_error() {
        let mut change_pin_client = MockChangePinClient::new();
        // First return a network error
        change_pin_client
            .expect_rollback_new_pin()
            .times(1)
            .returning(|_| Err(ChangePinClientTestError { is_network: true }));
        // Then return successfully
        change_pin_client
            .expect_rollback_new_pin()
            .times(1)
            .returning(|_| Ok(()));

        let mut change_pin_storage = MockChangePinStorage::new();
        change_pin_storage
            .expect_get_change_pin_state()
            .times(1)
            .returning(|| Ok(Some(State::Begin)));
        change_pin_storage
            .expect_clear_change_pin_state()
            .times(1)
            .returning(|| Ok(()));

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, 2);

        let actual = change_pin_session.continue_change_pin("123789".to_string()).await;

        assert_matches!(actual, Ok(()));
    }

    #[tokio::test]
    async fn continue_change_pin_begin_error() {
        let mut change_pin_client = MockChangePinClient::new();
        change_pin_client
            .expect_rollback_new_pin()
            .times(1)
            .returning(|_| Err(ChangePinClientTestError { is_network: false }));

        let mut change_pin_storage = MockChangePinStorage::new();
        change_pin_storage
            .expect_get_change_pin_state()
            .times(1)
            .returning(|| Ok(Some(State::Begin)));

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, 2);

        let actual = change_pin_session.continue_change_pin("123789".to_string()).await;

        assert_matches!(
            actual,
            Err(ChangePinError::Instruction(InstructionError::InstructionValidation))
        );
    }

    #[tokio::test]
    async fn continue_change_pin_no_change_pin_in_progress() {
        let change_pin_client = MockChangePinClient::new();

        let mut change_pin_storage = MockChangePinStorage::new();
        change_pin_storage
            .expect_get_change_pin_state()
            .times(1)
            .returning(|| Ok(None));

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, 2);

        let actual = change_pin_session.continue_change_pin("123789".to_string()).await;

        assert_matches!(actual, Err(ChangePinError::NoChangePinInProgress));
    }
}
