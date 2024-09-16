#![allow(unused)]

use std::future::Future;

use thiserror::Error;

use wallet_common::account::messages::auth::WalletCertificate;

use crate::{
    errors::{InstructionError, StorageError},
    pin::key::{self as pin_key},
};

#[derive(Debug, Clone, PartialEq, Eq)]
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
    async fn get_state(&self) -> Result<Option<State>, StorageError>;
    async fn store_state(&self, state: State) -> Result<(), StorageError>;
    async fn clean_state(&self) -> Result<(), StorageError>;

    async fn change_pin(&self, new_pin_salt: &[u8], new_pin_certificate: WalletCertificate)
        -> Result<(), StorageError>;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait ChangePinConfiguration {
    async fn max_retries(&self) -> u8;
}

#[derive(Debug, Error)]
enum ChangePinError {
    #[error("pin_change transaction already in progress")]
    ChangePinAlreadyInProgress,
    #[error("no pin_change transaction in progress")]
    NoChangePinInProgress,
    #[error("instruction failed: {0}")]
    Instruction(#[from] InstructionError), // TODO: better errors, split in specific errors
    #[error("storage error: {0}")]
    Storage(#[from] StorageError), // TODO: better errors, split in specific errors
}

type ChangePinResult<T> = Result<T, ChangePinError>;

trait ChangePin {
    /// Begin the ChangePin transaction.
    /// After this operation, the user SHOULD immediately be notified about either the success or failure of the change pin, and after that invoke the [continue_change_pin].
    async fn begin_change_pin(&self, old_pin: String, new_pin: String) -> Result<(), ChangePinError>;
    /// Continue the ChangePin transaction.
    /// This will either Commit or Rollback the transaction that was started in [begin_change_pin].
    async fn continue_change_pin(&self, pin: String) -> Result<(), ChangePinError>;
}

struct ChangePinSession<C, S, R> {
    client: C,
    storage: S,
    config: R,
}

impl<C, S, R> ChangePinSession<C, S, R> {
    fn new(client: C, storage: S, config: R) -> Self {
        Self {
            client,
            storage,
            config,
        }
    }
}

impl<C, S, R, E> ChangePinSession<C, S, R>
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
        tracing::debug!("{operation_name} change pin transaction");

        let max_retries = self.config.max_retries().await;

        let mut retries = 0;
        loop {
            retries += 1;
            let result = operation();
            match result.await {
                Ok(()) => break,
                Err(error) if error.is_network_error() => {
                    if retries >= max_retries {
                        tracing::warn!("network error during {operation_name}: {error:?}");
                        tracing::debug!("too many network errors during {operation_name}");
                        return Err(error.into());
                    } else {
                        tracing::warn!("network error during {operation_name}, trying again: {error:?}")
                    }
                }
                Err(error) => {
                    tracing::error!("error during {operation_name}: {error:?}");
                    return Err(error.into());
                }
            }
        }
        self.storage.clean_state().await?;

        tracing::debug!("{operation_name} successful");
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
}

// TODO: implement ChangePinClient and ChangePinStorage on Wallet, so that this can also be implemented on Wallet.
impl<C: ChangePinClient, S: ChangePinStorage, R: ChangePinConfiguration> ChangePin for ChangePinSession<C, S, R>
where
    ChangePinError: From<<C as ChangePinClient>::Error>,
{
    async fn begin_change_pin(&self, old_pin: String, new_pin: String) -> ChangePinResult<()> {
        tracing::debug!("start change pin transaction");

        let new_pin_salt = pin_key::new_pin_salt();

        self.storage.store_state(State::Begin).await?;

        match self.client.start_new_pin(&old_pin, &new_pin, &new_pin_salt).await {
            Ok(new_pin_certificate) => {
                self.storage.store_state(State::Commit).await?;
                self.storage.change_pin(&new_pin_salt, new_pin_certificate).await?;
                Ok(())
            }
            Err(error) if error.is_network_error() => {
                self.storage.store_state(State::Rollback).await?;
                Err(error.into())
            }
            Err(error) => {
                self.storage.clean_state().await?;
                Err(error.into())
            }
        }
    }

    async fn continue_change_pin(&self, pin: String) -> ChangePinResult<()> {
        tracing::debug!("continue change pin transaction");

        match self.storage.get_state().await? {
            None => Err(ChangePinError::NoChangePinInProgress),
            Some(State::Commit) => self.commit(pin).await,
            Some(State::Begin) | Some(State::Rollback) => self.rollback(pin).await,
        }
    }
}

#[cfg(any(test, feature = "mock"))]
mod mock {
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
    use super::{mock::*, *};

    use assert_matches::assert_matches;
    use mockall::predicate::eq;

    #[tokio::test]
    async fn begin_change_pin_success() {
        let mut change_pin_client = MockChangePinClient::new();
        change_pin_client
            .expect_start_new_pin()
            .returning(|_, _, _| Ok(WalletCertificate::from("thisisdefinitelyvalid")));

        let mut change_pin_storage = MockChangePinStorage::new();
        change_pin_storage
            .expect_store_state()
            .with(eq(State::Begin))
            .returning(|_| Ok(()));
        change_pin_storage
            .expect_store_state()
            .with(eq(State::Commit))
            .returning(|_| Ok(()));
        change_pin_storage.expect_change_pin().times(1).returning(|_, _| Ok(()));

        let change_pin_config = MockChangePinConfiguration::new();

        let change_pin_session = ChangePinSession::new(change_pin_client, change_pin_storage, change_pin_config);

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
            .expect_store_state()
            .with(eq(State::Begin))
            .returning(|_| Ok(()));
        change_pin_storage
            .expect_store_state()
            .with(eq(State::Rollback))
            .returning(|_| Ok(()));

        let change_pin_config = MockChangePinConfiguration::new();

        let change_pin_session = ChangePinSession::new(change_pin_client, change_pin_storage, change_pin_config);

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
            .expect_store_state()
            .with(eq(State::Begin))
            .returning(|_| Ok(()));
        change_pin_storage.expect_clean_state().times(1).returning(|| Ok(()));

        let change_pin_config = MockChangePinConfiguration::new();

        let change_pin_session = ChangePinSession::new(change_pin_client, change_pin_storage, change_pin_config);

        let actual = change_pin_session
            .begin_change_pin("000111".to_string(), "123789".to_string())
            .await;

        assert_matches!(
            actual,
            Err(ChangePinError::Instruction(InstructionError::InstructionValidation))
        );
    }

    #[tokio::test]
    async fn continue_change_pin_commit_success() {
        let mut change_pin_client = MockChangePinClient::new();
        change_pin_client.expect_commit_new_pin().times(1).returning(|_| Ok(()));

        let mut change_pin_storage = MockChangePinStorage::new();
        change_pin_storage
            .expect_get_state()
            .times(1)
            .returning(|| Ok(Some(State::Commit)));
        change_pin_storage.expect_clean_state().times(1).returning(|| Ok(()));

        let mut change_pin_config = MockChangePinConfiguration::new();
        change_pin_config.expect_max_retries().times(1).returning(|| 2);

        let change_pin_session = ChangePinSession::new(change_pin_client, change_pin_storage, change_pin_config);

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
            .expect_get_state()
            .times(1)
            .returning(|| Ok(Some(State::Commit)));

        let mut change_pin_config = MockChangePinConfiguration::new();
        change_pin_config.expect_max_retries().times(1).returning(|| 2);

        let change_pin_session = ChangePinSession::new(change_pin_client, change_pin_storage, change_pin_config);

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
            .expect_get_state()
            .times(1)
            .returning(|| Ok(Some(State::Commit)));
        change_pin_storage.expect_clean_state().times(1).returning(|| Ok(()));

        let mut change_pin_config = MockChangePinConfiguration::new();
        change_pin_config.expect_max_retries().times(1).returning(|| 2);

        let change_pin_session = ChangePinSession::new(change_pin_client, change_pin_storage, change_pin_config);

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
            .expect_get_state()
            .times(1)
            .returning(|| Ok(Some(State::Commit)));

        let mut change_pin_config = MockChangePinConfiguration::new();
        change_pin_config.expect_max_retries().times(1).returning(|| 2);

        let change_pin_session = ChangePinSession::new(change_pin_client, change_pin_storage, change_pin_config);

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
            .expect_get_state()
            .times(1)
            .returning(|| Ok(Some(State::Rollback)));
        change_pin_storage.expect_clean_state().times(1).returning(|| Ok(()));

        let mut change_pin_config = MockChangePinConfiguration::new();
        change_pin_config.expect_max_retries().times(1).returning(|| 2);

        let change_pin_session = ChangePinSession::new(change_pin_client, change_pin_storage, change_pin_config);

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
            .expect_get_state()
            .times(1)
            .returning(|| Ok(Some(State::Rollback)));

        let mut change_pin_config = MockChangePinConfiguration::new();
        change_pin_config.expect_max_retries().times(1).returning(|| 2);

        let change_pin_session = ChangePinSession::new(change_pin_client, change_pin_storage, change_pin_config);

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
            .expect_get_state()
            .times(1)
            .returning(|| Ok(Some(State::Rollback)));
        change_pin_storage.expect_clean_state().times(1).returning(|| Ok(()));

        let mut change_pin_config = MockChangePinConfiguration::new();
        change_pin_config.expect_max_retries().times(1).returning(|| 2);

        let change_pin_session = ChangePinSession::new(change_pin_client, change_pin_storage, change_pin_config);

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
            .expect_get_state()
            .times(1)
            .returning(|| Ok(Some(State::Rollback)));

        let mut change_pin_config = MockChangePinConfiguration::new();
        change_pin_config.expect_max_retries().times(1).returning(|| 2);

        let change_pin_session = ChangePinSession::new(change_pin_client, change_pin_storage, change_pin_config);

        let actual = change_pin_session.continue_change_pin("123789".to_string()).await;

        assert_matches!(
            actual,
            Err(ChangePinError::Instruction(InstructionError::InstructionValidation))
        );
    }
}
