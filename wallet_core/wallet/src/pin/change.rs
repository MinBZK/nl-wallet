#![allow(unused)]

use std::future::Future;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use wallet_common::account::messages::auth::WalletCertificate;

use crate::{
    errors::{InstructionError, StorageError},
    pin::key::{self as pin_key},
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

#[derive(Debug, Error)]
pub enum ChangePinError {
    #[error("pin_change transaction already in progress")]
    ChangePinAlreadyInProgress,
    #[error("no pin_change transaction in progress")]
    NoChangePinInProgress,
    #[error("instruction failed: {0}")]
    Instruction(#[from] InstructionError),
    #[error("storage error: {0}")]
    Storage(#[from] StorageError),
}

pub type ChangePinResult<T> = Result<T, ChangePinError>;

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
