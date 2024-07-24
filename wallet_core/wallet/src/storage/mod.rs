mod data;
mod database;
mod database_storage;
mod event_log;
mod key_file;
mod sql_cipher_key;

#[cfg(any(test, feature = "mock"))]
mod mock_storage;

use std::{array::TryFromSliceError, collections::HashSet, io};

use sea_orm::DbErr;
use uuid::Uuid;

use nl_wallet_mdoc::{
    holder::{Mdoc, MdocCopies},
    utils::{serialization::CborError, x509::Certificate},
};

pub use self::{
    data::{InstructionData, KeyedData, RegistrationData},
    database_storage::DatabaseStorage,
    event_log::{EventDocuments, EventStatus, WalletEvent},
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
}

pub type StorageResult<T> = Result<T, StorageError>;

#[derive(Debug, Clone)]
pub struct StoredMdocCopy {
    pub mdoc_id: Uuid,
    pub mdoc_copy_id: Uuid,
    pub mdoc: Mdoc,
}

/// This trait abstracts the persistent storage for the wallet.
pub trait Storage {
    async fn state(&self) -> StorageResult<StorageState>;

    async fn open(&mut self) -> StorageResult<()>;
    async fn clear(&mut self);

    async fn fetch_data<D: KeyedData>(&self) -> StorageResult<Option<D>>;
    async fn insert_data<D: KeyedData>(&mut self, data: &D) -> StorageResult<()>;
    async fn update_data<D: KeyedData>(&mut self, data: &D) -> StorageResult<()>;

    async fn insert_mdocs(&mut self, mdocs: Vec<MdocCopies>) -> StorageResult<()>;
    async fn increment_mdoc_copies_usage_count(&mut self, mdoc_copy_ids: Vec<Uuid>) -> StorageResult<()>;
    async fn fetch_unique_mdocs(&self) -> StorageResult<Vec<StoredMdocCopy>>;
    async fn fetch_unique_mdocs_by_doctypes(&self, doc_types: &HashSet<&str>) -> StorageResult<Vec<StoredMdocCopy>>;
    async fn has_any_mdocs_with_doctype(&self, doc_type: &str) -> StorageResult<bool>;

    async fn log_wallet_event(&mut self, event: WalletEvent) -> StorageResult<()>;
    async fn fetch_wallet_events(&self) -> StorageResult<Vec<WalletEvent>>;
    async fn fetch_recent_wallet_events(&self) -> StorageResult<Vec<WalletEvent>>;
    async fn fetch_wallet_events_by_doc_type(&self, doc_type: &str) -> StorageResult<Vec<WalletEvent>>;
    async fn did_share_data_with_relying_party(&self, certificate: &Certificate) -> StorageResult<bool>;
}
