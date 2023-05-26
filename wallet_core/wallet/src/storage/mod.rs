mod data;
mod database;
mod database_storage;
mod key_file;
mod sql_cipher_key;

#[cfg(any(test, feature = "mock"))]
mod mock_storage;

use std::error::Error;

use async_trait::async_trait;

pub use self::{
    data::{KeyedData, RegistrationData},
    database_storage::DatabaseStorage,
};

#[cfg(any(test, feature = "mock"))]
pub use self::mock_storage::MockStorage;

/// This represents the current start of [`Storage`].
#[derive(Debug, Clone, Copy)]
pub enum StorageState {
    /// There is no database connection and no file on disk.
    Uninitialized,
    /// There is no database connection, but there is a file on disk.
    Unopened,
    /// There is an open database connection.
    Opened,
}

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Storage database is not opened")]
    NotOpened,
    #[error("Storage database is already opened")]
    AlreadyOpened,
    #[error(transparent)]
    Other(#[from] Box<dyn Error + Send + Sync>),
}

/// This trait abstracts the persistent storage for the wallet.
#[async_trait]
pub trait Storage {
    async fn state(&self) -> Result<StorageState, StorageError>;

    async fn open(&mut self) -> Result<(), StorageError>;
    async fn clear(&mut self) -> Result<(), StorageError>;

    async fn fetch_data<D: KeyedData>(&self) -> Result<Option<D>, StorageError>;
    async fn insert_data<D: KeyedData>(&mut self, data: &D) -> Result<(), StorageError>;
}
