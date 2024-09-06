#![allow(unused)]

use thiserror::Error;

use wallet_common::account::messages::auth::WalletCertificate;

use crate::{
    errors::{InstructionError, StorageError},
    pin::key::{self as pin_key},
};

#[derive(Debug)]
pub enum State {
    Commit,
    Rollback,
}

pub trait ChangePinClientError: std::error::Error {
    fn is_already_done(&self) -> bool;
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum ChangePinStatus {
    /// Pin was changed successfully, use new Pin from now on
    Success,
    /// Pin change failed, keep using old Pin
    Failed,
    /// Commit or Rollback failed too many times, lock wallet and try later
    InProgress,
}

#[derive(Debug, Error)]
enum ChangePinError {
    #[error("instruction failed: {0}")]
    Instruction(#[from] InstructionError), // TODO: better errors, split in specific errors
    #[error("storage error: {0}")]
    Storage(#[from] StorageError), // TODO: better errors, split in specific errors
}

type ChangePinResult = Result<ChangePinStatus, ChangePinError>;

trait ChangePin {
    async fn status(&self) -> Result<Option<State>, ChangePinError>;

    async fn begin(&self, old_pin: String, new_pin: String) -> ChangePinResult;
    async fn commit(&self, new_pin: String) -> ChangePinResult;
    async fn rollback(&self, old_pin: String) -> ChangePinResult;
}

struct ChangePinSession<C, S> {
    client: C,
    storage: S,
}

impl<C, S> ChangePinSession<C, S> {
    fn new(client: C, storage: S) -> Self {
        Self { client, storage }
    }
}

// TODO: implement ChangePinClient and ChangePinStorage on Wallet, so that this can also be implemented on Wallet.
impl<C: ChangePinClient, S: ChangePinStorage> ChangePin for ChangePinSession<C, S>
where
    ChangePinError: From<<C as ChangePinClient>::Error>,
{
    async fn status(&self) -> Result<Option<State>, ChangePinError> {
        let result = self.storage.get_state().await?;
        Ok(result)
    }

    async fn begin(&self, old_pin: String, new_pin: String) -> ChangePinResult {
        tracing::debug!("start change pin transaction");

        let new_pin_salt = pin_key::new_pin_salt();

        match self.client.start_new_pin(&old_pin, &new_pin, &new_pin_salt).await {
            Ok(new_pin_certificate) => {
                self.storage.store_state(State::Commit).await?;
                self.storage.change_pin(&new_pin_salt, new_pin_certificate).await?;
                self.commit(new_pin).await
            }
            Err(error) if error.is_network_error() => {
                self.storage.store_state(State::Rollback).await?;
                self.rollback(old_pin).await
            }
            Err(_) => Ok(ChangePinStatus::Failed),
        }
    }

    async fn commit(&self, new_pin: String) -> ChangePinResult {
        tracing::debug!("commit change pin transaction");

        let mut retries = 0;
        loop {
            match self.client.commit_new_pin(&new_pin).await {
                Ok(()) => break,
                Err(error) if error.is_already_done() => break,
                Err(error) => tracing::debug!("error during commit, retry: {error:?}"),
            }
            retries += 1;
            if retries >= 3 {
                tracing::debug!("too many errors during rollback");
                return Ok(ChangePinStatus::InProgress);
            }
        }
        self.storage.clean_state().await?;
        Ok(ChangePinStatus::Success)
    }

    async fn rollback(&self, old_pin: String) -> ChangePinResult {
        tracing::debug!("rollback change pin transaction");

        let mut retries = 0;
        loop {
            match self.client.rollback_new_pin(&old_pin).await {
                Ok(()) => break,
                Err(error) if error.is_already_done() => break,
                Err(error) => tracing::debug!("error during rollback, retry: {error:?}"),
            }
            retries += 1;
            if retries >= 3 {
                tracing::debug!("too many errors during rollback");
                return Ok(ChangePinStatus::InProgress);
            }
        }
        Ok(ChangePinStatus::Failed)
    }
}

#[cfg(any(test, feature = "mock"))]
mod mock {
    use super::*;

    #[derive(Debug, thiserror::Error)]
    #[error("")]
    pub struct ChangePinClientTestError {
        pub is_done: bool,
        pub is_network: bool,
    }

    impl ChangePinClientError for ChangePinClientTestError {
        fn is_already_done(&self) -> bool {
            self.is_done
        }
        fn is_network_error(&self) -> bool {
            self.is_network
        }
    }

    impl From<ChangePinClientTestError> for ChangePinError {
        fn from(value: ChangePinClientTestError) -> Self {
            if value.is_network_error() {
                Self::Instruction(InstructionError::Timeout { timeout_millis: 15 })
            } else if value.is_already_done() {
                Self::Instruction(InstructionError::Blocked)
            } else {
                Self::Instruction(InstructionError::InstructionValidation)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;

    use super::{mock::*, *};

    #[tokio::test]
    async fn status_is_read_from_storage() {
        let change_pin_client = MockChangePinClient::new();

        let mut change_pin_storage = MockChangePinStorage::new();
        change_pin_storage
            .expect_get_state()
            .returning(|| Ok(Some(State::Commit)));

        let change_pin_session = ChangePinSession::new(change_pin_client, change_pin_storage);

        let actual = change_pin_session.status().await;

        assert_matches!(actual, Ok(Some(State::Commit)));
    }

    // start_new_pin succeeds, commit succeeds
    // returns Success
    #[tokio::test]
    async fn happy_flow() {
        let mut change_pin_client = MockChangePinClient::new();
        change_pin_client
            .expect_start_new_pin()
            .returning(|_, _, _| Ok(WalletCertificate::from("thisisdefinitelyvalid")));
        change_pin_client.expect_commit_new_pin().returning(|_| Ok(()));

        let mut change_pin_storage = MockChangePinStorage::new();
        change_pin_storage.expect_store_state().returning(|_| Ok(()));
        change_pin_storage.expect_change_pin().returning(|_, _| Ok(()));
        change_pin_storage.expect_clean_state().returning(|| Ok(()));

        let change_pin_session = ChangePinSession::new(change_pin_client, change_pin_storage);

        let actual = change_pin_session
            .begin("000111".to_string(), "123789".to_string())
            .await;

        assert_matches!(actual, Ok(ChangePinStatus::Success));
    }

    // start_new_pin fails, no rollback performed
    // returns Failed
    #[tokio::test]
    async fn immediate_failure() {
        let mut change_pin_client = MockChangePinClient::new();
        change_pin_client.expect_start_new_pin().returning(|_, _, _| {
            Err(ChangePinClientTestError {
                is_done: false,
                is_network: false,
            })
        });

        let change_pin_storage = MockChangePinStorage::new();

        let change_pin_session = ChangePinSession::new(change_pin_client, change_pin_storage);

        let actual = change_pin_session
            .begin("000111".to_string(), "123789".to_string())
            .await;

        assert_matches!(actual, Ok(ChangePinStatus::Failed));
    }

    // start_new_pin fails with a network error, rollback succeeds
    // returns Failed
    #[tokio::test]
    async fn immediate_rolback() {
        let mut change_pin_client = MockChangePinClient::new();
        change_pin_client.expect_start_new_pin().returning(|_, _, _| {
            Err(ChangePinClientTestError {
                is_done: false,
                is_network: true,
            })
        });
        change_pin_client.expect_rollback_new_pin().returning(|_| Ok(()));

        let mut change_pin_storage = MockChangePinStorage::new();
        change_pin_storage.expect_store_state().returning(|_| Ok(()));

        let change_pin_session = ChangePinSession::new(change_pin_client, change_pin_storage);

        let actual = change_pin_session
            .begin("000111".to_string(), "123789".to_string())
            .await;

        assert_matches!(actual, Ok(ChangePinStatus::Failed));
    }

    // start_new_pin succeeds, commit fails repeatedly
    // returns InProgress
    #[tokio::test]
    async fn pin_commit_network_errors() {
        let mut change_pin_client = MockChangePinClient::new();
        change_pin_client
            .expect_start_new_pin()
            .returning(|_, _, _| Ok(WalletCertificate::from("thisisdefinitelyvalid")));
        change_pin_client.expect_commit_new_pin().returning(|_| {
            Err(ChangePinClientTestError {
                is_done: false,
                is_network: true,
            })
        });

        let mut change_pin_storage = MockChangePinStorage::new();
        change_pin_storage.expect_store_state().returning(|_| Ok(()));
        change_pin_storage.expect_change_pin().returning(|_, _| Ok(()));

        let change_pin_session = ChangePinSession::new(change_pin_client, change_pin_storage);

        let actual = change_pin_session
            .begin("000111".to_string(), "123789".to_string())
            .await;

        assert_matches!(actual, Ok(ChangePinStatus::InProgress));
    }

    // start_new_pin fails with a network error, rollback fails repeatedly
    // returns InProgress
    #[tokio::test]
    async fn pin_rollback_network_errors() {
        let mut change_pin_client = MockChangePinClient::new();
        change_pin_client.expect_start_new_pin().returning(|_, _, _| {
            Err(ChangePinClientTestError {
                is_done: false,
                is_network: true,
            })
        });
        change_pin_client.expect_rollback_new_pin().returning(|_| {
            Err(ChangePinClientTestError {
                is_done: false,
                is_network: true,
            })
        });

        let mut change_pin_storage = MockChangePinStorage::new();
        change_pin_storage.expect_store_state().returning(|_| Ok(()));

        let change_pin_session = ChangePinSession::new(change_pin_client, change_pin_storage);

        let actual = change_pin_session
            .begin("000111".to_string(), "123789".to_string())
            .await;

        assert_matches!(actual, Ok(ChangePinStatus::InProgress));
    }
}
