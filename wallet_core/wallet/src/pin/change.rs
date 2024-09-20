use std::future::Future;

use serde::{Deserialize, Serialize};

use error_category::ErrorCategory;
use wallet_common::account::messages::auth::WalletCertificate;

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
        new_pin_salt: Vec<u8>,
        new_pin_certificate: WalletCertificate,
    ) -> Result<(), StorageError>;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait ChangePinConfiguration {
    async fn max_retries(&self) -> u8;
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
}

pub type ChangePinResult<T> = Result<T, ChangePinError>;

pub struct ChangePinSession<'a, C, S, R> {
    client: &'a C,
    storage: &'a S,
    config: &'a R,
}

impl<'a, C, S, R> ChangePinSession<'a, C, S, R> {
    pub fn new(client: &'a C, storage: &'a S, config: &'a R) -> Self {
        Self {
            client,
            storage,
            config,
        }
    }
}

impl<'a, C, S, R, E> ChangePinSession<'a, C, S, R>
where
    C: ChangePinClient<Error = E>,
    S: ChangePinStorage,
    R: ChangePinConfiguration,
    E: ChangePinClientError + Into<ChangePinError>,
{
    /// Perform [`operation`] and retry a number of times when a network error occurs.
    async fn with_retries<F, Fut>(&self, operation_name: &str, operation: F) -> ChangePinResult<()>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<(), E>>,
    {
        tracing::info!("{operation_name} change PIN transaction");

        let max_retries = self.config.max_retries().await;

        let mut retries = 0;
        loop {
            retries += 1;
            let result = operation();
            match result.await {
                Ok(()) => break,
                Err(error) if error.is_network_error() => {
                    if retries >= max_retries {
                        tracing::warn!("Network error during {operation_name}: {error:?}");
                        tracing::info!("Too many network errors during {operation_name}");
                        return Err(error.into());
                    } else {
                        tracing::warn!("Network error during {operation_name}, trying again: {error:?}")
                    }
                }
                Err(error) => {
                    tracing::error!("Error during {operation_name}: {error:?}");
                    return Err(error.into());
                }
            }
        }
        self.storage.clear_change_pin_state().await?;

        tracing::info!("{operation_name} successful");
        Ok(())
    }

    async fn commit(&self, new_pin: String) -> ChangePinResult<()> {
        self.with_retries("commit", || async { self.client.commit_new_pin(&new_pin).await })
            .await
    }

    async fn rollback(&self, old_pin: String) -> ChangePinResult<()> {
        self.with_retries("rollback", || async { self.client.rollback_new_pin(&old_pin).await })
            .await
    }

    pub async fn begin_change_pin(&self, old_pin: String, new_pin: String) -> ChangePinResult<()> {
        tracing::info!("Start change PIN transaction");

        tracing::info!("Ensure no PIN change is in progress");
        if let Some(_state) = self.storage.get_change_pin_state().await? {
            return Err(ChangePinError::ChangePinAlreadyInProgress);
        }

        tracing::info!("Validating new PIN");
        // Make sure the new PIN adheres to the requirements.
        validate_pin(&new_pin)?;

        let new_pin_salt = pin_key::new_pin_salt();

        self.storage.store_change_pin_state(State::Begin).await?;

        match self.client.start_new_pin(&old_pin, &new_pin, &new_pin_salt).await {
            Ok(new_pin_certificate) => {
                self.storage.store_change_pin_state(State::Commit).await?;
                self.storage.change_pin(new_pin_salt, new_pin_certificate).await?;
                Ok(())
            }
            Err(error) if error.is_network_error() => {
                self.storage.store_change_pin_state(State::Rollback).await?;
                Err(error.into())
            }
            Err(error) => {
                self.storage.clear_change_pin_state().await?;
                Err(error.into())
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

    use wallet_common::account::messages::auth::WalletCertificate;

    use super::*;

    use crate::{
        errors::InstructionError,
        pin::change::{
            mock::ChangePinClientTestError, ChangePinError, MockChangePinClient, MockChangePinConfiguration,
            MockChangePinStorage, State,
        },
    };

    #[tokio::test]
    async fn begin_change_pin_success() {
        let mut change_pin_client = MockChangePinClient::new();
        change_pin_client
            .expect_start_new_pin()
            .returning(|_, _, _| Ok(WalletCertificate::from("thisisdefinitelyvalid")));

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
            .with(eq(State::Commit))
            .returning(|_| Ok(()));
        change_pin_storage.expect_change_pin().times(1).returning(|_, _| Ok(()));

        let change_pin_config = MockChangePinConfiguration::new();

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, &change_pin_config);

        let actual = change_pin_session
            .begin_change_pin("000111".to_string(), "123789".to_string())
            .await;

        assert_matches!(actual, Ok(()));
    }

    #[tokio::test]
    async fn begin_change_pin_network_error() {
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

        let change_pin_config = MockChangePinConfiguration::new();

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, &change_pin_config);

        let actual = change_pin_session
            .begin_change_pin("000111".to_string(), "123789".to_string())
            .await;

        assert_matches!(
            actual,
            Err(ChangePinError::Instruction(InstructionError::Timeout {
                timeout_millis: 15
            }))
        );
    }

    #[tokio::test]
    async fn begin_change_pin_error() {
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

        let change_pin_config = MockChangePinConfiguration::new();

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, &change_pin_config);

        let actual = change_pin_session
            .begin_change_pin("000111".to_string(), "123789".to_string())
            .await;

        assert_matches!(
            actual,
            Err(ChangePinError::Instruction(InstructionError::InstructionValidation))
        );
    }

    #[tokio::test]
    async fn begin_change_pin_error_already_in_progress() {
        let change_pin_client = MockChangePinClient::new();

        let mut change_pin_storage = MockChangePinStorage::new();
        change_pin_storage
            .expect_get_change_pin_state()
            .times(1)
            .returning(|| Ok(Some(State::Begin)));

        let change_pin_config = MockChangePinConfiguration::new();

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, &change_pin_config);

        let actual = change_pin_session
            .begin_change_pin("000111".to_string(), "123789".to_string())
            .await;

        assert_matches!(actual, Err(ChangePinError::ChangePinAlreadyInProgress));
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

        let mut change_pin_config = MockChangePinConfiguration::new();
        change_pin_config.expect_max_retries().times(1).returning(|| 2);

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, &change_pin_config);

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

        let mut change_pin_config = MockChangePinConfiguration::new();
        change_pin_config.expect_max_retries().times(1).returning(|| 2);

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, &change_pin_config);

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

        let mut change_pin_config = MockChangePinConfiguration::new();
        change_pin_config.expect_max_retries().times(1).returning(|| 2);

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, &change_pin_config);

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

        let mut change_pin_config = MockChangePinConfiguration::new();
        change_pin_config.expect_max_retries().times(1).returning(|| 2);

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, &change_pin_config);

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

        let mut change_pin_config = MockChangePinConfiguration::new();
        change_pin_config.expect_max_retries().times(1).returning(|| 2);

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, &change_pin_config);

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

        let mut change_pin_config = MockChangePinConfiguration::new();
        change_pin_config.expect_max_retries().times(1).returning(|| 2);

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, &change_pin_config);

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

        let mut change_pin_config = MockChangePinConfiguration::new();
        change_pin_config.expect_max_retries().times(1).returning(|| 2);

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, &change_pin_config);

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

        let mut change_pin_config = MockChangePinConfiguration::new();
        change_pin_config.expect_max_retries().times(1).returning(|| 2);

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, &change_pin_config);

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

        let mut change_pin_config = MockChangePinConfiguration::new();
        change_pin_config.expect_max_retries().times(1).returning(|| 2);

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, &change_pin_config);

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

        let mut change_pin_config = MockChangePinConfiguration::new();
        change_pin_config.expect_max_retries().times(1).returning(|| 2);

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, &change_pin_config);

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

        let mut change_pin_config = MockChangePinConfiguration::new();
        change_pin_config.expect_max_retries().times(1).returning(|| 2);

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, &change_pin_config);

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

        let mut change_pin_config = MockChangePinConfiguration::new();
        change_pin_config.expect_max_retries().times(1).returning(|| 2);

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, &change_pin_config);

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

        let change_pin_config = MockChangePinConfiguration::new();

        let change_pin_session = ChangePinSession::new(&change_pin_client, &change_pin_storage, &change_pin_config);

        let actual = change_pin_session.continue_change_pin("123789".to_string()).await;

        assert_matches!(actual, Err(ChangePinError::NoChangePinInProgress));
    }
}
