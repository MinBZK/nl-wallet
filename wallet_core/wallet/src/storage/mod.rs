mod data;
mod database;
mod database_storage;
mod event_log;
mod key_file;
mod sql_cipher_key;

#[cfg(any(test, feature = "mock"))]
mod mock_storage;

use std::{array::TryFromSliceError, collections::HashSet, io};

use async_trait::async_trait;
use sea_orm::DbErr;
use uuid::Uuid;

use nl_wallet_mdoc::{
    holder::{Mdoc, MdocCopies},
    utils::serialization::CborError,
};
use platform_support::utils::UtilitiesError;

pub use self::{
    data::{InstructionData, KeyedData, RegistrationData},
    database_storage::DatabaseStorage,
    event_log::{EventType, Status, WalletEvent},
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
    #[error("storage database CBOR error: {0}")]
    Cbor(#[from] CborError),
    #[error("storage database SQLCipher key error: {0}")]
    SqlCipherKey(#[from] TryFromSliceError),
    #[error("{0}")]
    KeyFile(#[from] KeyFileError),
    #[error("storage database platform utilities error: {0}")]
    PlatformUtilities(#[from] UtilitiesError),
}

pub type StorageResult<T> = Result<T, StorageError>;

/// This trait abstracts the persistent storage for the wallet.
#[async_trait]
pub trait Storage {
    async fn state(&self) -> StorageResult<StorageState>;

    async fn open(&mut self) -> StorageResult<()>;
    async fn clear(&mut self) -> StorageResult<()>;

    async fn fetch_data<D: KeyedData>(&self) -> StorageResult<Option<D>>;
    async fn insert_data<D: KeyedData + Sync>(&mut self, data: &D) -> StorageResult<()>;
    async fn update_data<D: KeyedData + Sync>(&mut self, data: &D) -> StorageResult<()>;

    async fn insert_mdocs(&mut self, mdocs: Vec<MdocCopies>) -> StorageResult<()>;
    async fn fetch_unique_mdocs(&self) -> StorageResult<Vec<(Uuid, Mdoc)>>;
    async fn fetch_unique_mdocs_by_doctypes(&self, doc_types: &HashSet<&str>) -> StorageResult<Vec<(Uuid, Mdoc)>>;

    async fn log_wallet_events(&mut self, events: Vec<WalletEvent>) -> StorageResult<()>;
    async fn fetch_wallet_events(&self) -> StorageResult<Vec<WalletEvent>>;
    async fn fetch_wallet_events_by_doc_type(&self, doc_type: &str) -> StorageResult<Vec<WalletEvent>>;
}
