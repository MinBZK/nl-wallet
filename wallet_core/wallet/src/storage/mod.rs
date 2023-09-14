mod data;
mod database;
mod database_storage;
mod key_file;
mod sql_cipher_key;

#[cfg(any(test, feature = "mock"))]
mod mock_storage;

use std::{array::TryFromSliceError, io};

use async_trait::async_trait;
use sea_orm::DbErr;

use platform_support::utils::UtilitiesError;

pub use self::{
    data::{InstructionData, KeyedData, RegistrationData},
    database_storage::DatabaseStorage,
    key_file::KeyFileError,
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
    #[error("storage database is not opened")]
    NotOpened,
    #[error("storage database is already opened")]
    AlreadyOpened,
    #[error("storage database I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("storage database error: {0}")]
    Database(#[from] DbErr),
    #[error("storage database JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("storage database SQLCipher key error: {0}")]
    SqlCipherKey(#[from] TryFromSliceError),
    #[error("{0}")]
    KeyFile(#[from] KeyFileError),
    #[error("storage database platform utilities error: {0}")]
    PlatformUtilities(#[from] UtilitiesError),
}

/// This trait abstracts the persistent storage for the wallet.
#[async_trait]
pub trait Storage {
    async fn state(&self) -> Result<StorageState, StorageError>;

    async fn open(&mut self) -> Result<(), StorageError>;
    async fn clear(&mut self) -> Result<(), StorageError>;

    async fn fetch_data<D: KeyedData>(&self) -> Result<Option<D>, StorageError>;
    async fn insert_data<D: KeyedData + Sync>(&mut self, data: &D) -> Result<(), StorageError>;
    async fn update_data<D: KeyedData + Sync>(&mut self, data: &D) -> Result<(), StorageError>;
}
