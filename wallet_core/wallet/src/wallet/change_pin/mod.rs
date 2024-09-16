#![allow(unused)] // TODO: remove once connected from api.rs

mod config;
mod storage;

use std::future::Future;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use wallet_common::account::messages::auth::WalletCertificate;

use crate::{
    errors::{InstructionError, StorageError},
    pin::{
        change::{
            ChangePin, ChangePinClient, ChangePinClientError, ChangePinConfiguration, ChangePinError, ChangePinResult,
            ChangePinStorage, State,
        },
        key::{self as pin_key},
    },
};

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
        self.storage.clear_change_pin_state().await?;

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
impl<C, S, R, E> ChangePin for ChangePinSession<C, S, R>
where
    C: ChangePinClient<Error = E>,
    S: ChangePinStorage,
    R: ChangePinConfiguration,
    E: ChangePinClientError + Into<ChangePinError>,
{
    async fn begin_change_pin(&self, old_pin: String, new_pin: String) -> ChangePinResult<()> {
        tracing::debug!("start change pin transaction");

        if let Some(_state) = self.storage.get_change_pin_state().await? {
            return Err(ChangePinError::ChangePinAlreadyInProgress);
        }

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

    async fn continue_change_pin(&self, pin: String) -> ChangePinResult<()> {
        tracing::debug!("continue change pin transaction");

        match self.storage.get_change_pin_state().await? {
            None => Err(ChangePinError::NoChangePinInProgress),
            Some(State::Commit) => self.commit(pin).await,
            Some(State::Begin) | Some(State::Rollback) => self.rollback(pin).await,
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
            mock::ChangePinClientTestError, ChangePinClient, ChangePinConfiguration, ChangePinError, ChangePinStorage,
            MockChangePinClient, MockChangePinConfiguration, MockChangePinStorage, State,
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
    async fn begin_change_pin_error_already_in_progress() {
        let change_pin_client = MockChangePinClient::new();

        let mut change_pin_storage = MockChangePinStorage::new();
        change_pin_storage
            .expect_get_change_pin_state()
            .times(1)
            .returning(|| Ok(Some(State::Begin)));

        let change_pin_config = MockChangePinConfiguration::new();

        let change_pin_session = ChangePinSession::new(change_pin_client, change_pin_storage, change_pin_config);

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
            .expect_get_change_pin_state()
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
            .expect_get_change_pin_state()
            .times(1)
            .returning(|| Ok(Some(State::Commit)));
        change_pin_storage
            .expect_clear_change_pin_state()
            .times(1)
            .returning(|| Ok(()));

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
            .expect_get_change_pin_state()
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
            .expect_get_change_pin_state()
            .times(1)
            .returning(|| Ok(Some(State::Rollback)));
        change_pin_storage
            .expect_clear_change_pin_state()
            .times(1)
            .returning(|| Ok(()));

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
            .expect_get_change_pin_state()
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
            .expect_get_change_pin_state()
            .times(1)
            .returning(|| Ok(Some(State::Rollback)));
        change_pin_storage
            .expect_clear_change_pin_state()
            .times(1)
            .returning(|| Ok(()));

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
            .expect_get_change_pin_state()
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

        let change_pin_session = ChangePinSession::new(change_pin_client, change_pin_storage, change_pin_config);

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

        let change_pin_session = ChangePinSession::new(change_pin_client, change_pin_storage, change_pin_config);

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

        let change_pin_session = ChangePinSession::new(change_pin_client, change_pin_storage, change_pin_config);

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

        let change_pin_session = ChangePinSession::new(change_pin_client, change_pin_storage, change_pin_config);

        let actual = change_pin_session.continue_change_pin("123789".to_string()).await;

        assert_matches!(actual, Err(ChangePinError::NoChangePinInProgress));
    }
}
