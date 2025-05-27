mod data;
mod database;
mod database_storage;
mod event_log;
mod key_file;
mod sql_cipher_key;

#[cfg(any(test, feature = "mock"))]
mod mock_storage;

#[cfg(test)]
pub use mock_storage::KeyedDataResult;

use std::array::TryFromSliceError;
use std::collections::HashSet;
use std::io;

use sea_orm::DbErr;
use uuid::Uuid;

use crypto::x509::BorrowingCertificate;
use error_category::ErrorCategory;
use mdoc::holder::Mdoc;
use mdoc::utils::serialization::CborError;
use openid4vc::issuance_session::IssuedCredentialCopies;

pub use self::data::ChangePinData;
pub use self::data::InstructionData;
pub use self::data::KeyData;
pub use self::data::KeyedData;
pub use self::data::RegistrationData;
pub use self::data::UnlockData;
pub use self::data::UnlockMethod;
pub use self::database_storage::DatabaseStorage;
pub use self::event_log::disclosure_type_for_proposed_attributes;
pub use self::event_log::DataDisclosureStatus;
pub use self::event_log::DisclosureStatus;
pub use self::event_log::DisclosureType;
pub use self::event_log::WalletEvent;
pub use self::key_file::KeyFileError;

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

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum StorageError {
    #[error("storage database is not opened")]
    #[category(critical)]
    NotOpened,
    #[error("storage database is already opened")]
    #[category(critical)]
    AlreadyOpened,
    #[error("storage database I/O error: {0}")]
    #[category(critical)]
    Io(#[from] io::Error),
    #[error("storage database error: {0}")]
    #[category(critical)]
    Database(#[from] DbErr),
    #[error("storage database JSON error: {0}")]
    #[category(pd)]
    Json(#[from] serde_json::Error),
    #[error("storage database CBOR error: {0}")]
    Cbor(#[from] CborError),
    #[error("storage database SQLCipher key error: {0}")]
    #[category(pd)] // we don't want to leak the key
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

    async fn open_if_needed(&mut self) -> StorageResult<()> {
        let StorageState::Opened = self.state().await? else {
            return self.open().await;
        };

        Ok(())
    }

    async fn fetch_data<D: KeyedData>(&self) -> StorageResult<Option<D>>;
    async fn insert_data<D: KeyedData>(&mut self, data: &D) -> StorageResult<()>;
    async fn upsert_data<D: KeyedData>(&mut self, data: &D) -> StorageResult<()>;
    async fn delete_data<D: KeyedData>(&mut self) -> StorageResult<()>;

    async fn insert_credentials(&mut self, credentials: Vec<IssuedCredentialCopies>) -> StorageResult<()>;

    async fn increment_mdoc_copies_usage_count(&mut self, mdoc_copy_ids: Vec<Uuid>) -> StorageResult<()>;
    async fn fetch_unique_mdocs(&self) -> StorageResult<Vec<StoredMdocCopy>>;
    async fn fetch_unique_mdocs_by_doctypes(&self, doc_types: &HashSet<&str>) -> StorageResult<Vec<StoredMdocCopy>>;
    async fn has_any_mdocs_with_doctype(&self, doc_type: &str) -> StorageResult<bool>;

    async fn log_wallet_event(&mut self, event: WalletEvent) -> StorageResult<()>;
    async fn fetch_wallet_events(&self) -> StorageResult<Vec<WalletEvent>>;
    async fn fetch_recent_wallet_events(&self) -> StorageResult<Vec<WalletEvent>>;
    async fn fetch_wallet_events_by_attestation_type(&self, attestation_type: &str) -> StorageResult<Vec<WalletEvent>>;
    async fn did_share_data_with_relying_party(&self, certificate: &BorrowingCertificate) -> StorageResult<bool>;
}
