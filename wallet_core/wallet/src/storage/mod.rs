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
pub enum StorageOpenedError {
    #[error("Storage database is not opened")]
    NotOpened,
    #[error("Storage database is already opened")]
    AlreadyOpened,
}

/// This trait abstracts the persistent storage for the wallet.
#[async_trait]
pub trait Storage {
    type Error: Error + Send + Sync + 'static;

    async fn state(&self) -> Result<StorageState, Self::Error>;

    async fn open(&mut self) -> Result<(), Self::Error>;
    async fn clear(&mut self) -> Result<(), Self::Error>;

    async fn fetch_data<D: KeyedData>(&self) -> Result<Option<D>, Self::Error>;
    async fn insert_data<D: KeyedData>(&mut self, data: &D) -> Result<(), Self::Error>;
}
