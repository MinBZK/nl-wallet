mod attestation_copy;
mod data;
mod database;
mod database_storage;
mod event_log;
mod key_file;
mod revocation_info;
mod sql_cipher_key;

#[cfg(any(test, feature = "test"))]
pub use database_storage::test_storage::MockHardwareDatabaseStorage;

use std::array::TryFromSliceError;
use std::collections::HashSet;
use std::io;

use chrono::DateTime;
use chrono::Utc;
use sea_orm::DbErr;

use derive_more::Constructor;
use serde::Deserialize;
use serde::Serialize;
use tempfile::NamedTempFile;
use uuid::Uuid;

use attestation_data::disclosure_type::DisclosureType;
use crypto::x509::BorrowingCertificate;
use dcql::CredentialFormat;
use error_category::ErrorCategory;
use mdoc::utils::cose::CoseError;
use mdoc::utils::serialization::CborError;
use openid4vc::issuance_session::CredentialWithMetadata;
use openid4vc::issuance_session::IssuedCredentialCopies;
use sd_jwt_vc_metadata::TypeMetadataChainError;
use token_status_list::verification::verifier::RevocationStatus;
use utils::generator::Generator;

use crate::AttestationPresentation;
use crate::storage::sql_cipher_key::SqlCipherKey;

pub use self::attestation_copy::DisclosableAttestation;
pub use self::attestation_copy::PartialAttestation;
pub use self::attestation_copy::StoredAttestation;
pub use self::attestation_copy::StoredAttestationCopy;
pub use self::data::ChangePinData;
pub use self::data::InstructionData;
pub use self::data::KeyData;
pub use self::data::KeyedData;
pub use self::data::PinRecoveryData;
pub use self::data::RegistrationData;
pub use self::data::TransferData;
pub use self::data::TransferKeyData;
pub use self::data::UnlockData;
pub use self::data::UnlockMethod;
pub use self::database_storage::DatabaseStorage;
pub use self::event_log::DisclosureStatus;
pub use self::event_log::WalletEvent;
pub use self::key_file::KeyFileError;
pub use self::revocation_info::RevocationInfo;

#[cfg(test)]
pub mod test {
    pub use crate::storage::sql_cipher_key::SqlCipherKey;
}

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

    #[error("could not decode stored metadata chain: {0}")]
    #[category(pd)]
    MetadataChain(#[from] TypeMetadataChainError),

    #[error("could not encode / decode mdoc CBOR: {0}")]
    Cbor(#[from] CborError),

    #[error("could not deserialize mdoc IssuerSigned: {0}")]
    IssuerSigned(#[from] CoseError),

    #[error("storage database SD-JWT error: {0}")]
    #[category(pd)]
    SdJwt(#[from] sd_jwt::error::DecoderError),

    #[error("cannot store attestation event having an ephemeral identity")]
    #[category(critical)]
    EventEphemeralIdentity,

    #[error("storage database SQLCipher key error: {0}")]
    #[category(pd)] // we don't want to leak the key
    SqlCipherKey(#[from] TryFromSliceError),

    #[error("{0}")]
    KeyFile(#[from] KeyFileError),

    #[error("sqlite error: {0}")]
    #[category(pd)]
    Sqlite(#[from] rusqlite::Error),

    #[error("join error: {0}")]
    #[category(pd)]
    Join(#[from] tokio::task::JoinError),

    #[error("only file storage can be exported")]
    #[category(pd)]
    OnlyFileStorageExport,
}

pub type StorageResult<T> = Result<T, StorageError>;

/// Database export with one time key and the data.
/// Using an encrypted database because SQLCipher exports to file system,
/// and we do not want an unencrypted file written to disk.
#[derive(Constructor, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatabaseExport {
    key: SqlCipherKey,
    data: Vec<u8>,
}

/// This trait abstracts the persistent storage for the wallet.
#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait Storage: Send {
    async fn state(&self) -> StorageResult<StorageState>;

    async fn open(&mut self) -> StorageResult<()>;
    async fn export(&mut self) -> StorageResult<DatabaseExport>;
    async fn prepare_import(&mut self, export: DatabaseExport, database_file: &NamedTempFile) -> StorageResult<()>;
    async fn commit_import(&mut self, database_file: NamedTempFile) -> StorageResult<()>;
    async fn clear(&mut self);

    async fn open_if_needed(&mut self) -> StorageResult<()> {
        let StorageState::Opened = self.state().await? else {
            return self.open().await;
        };

        Ok(())
    }

    async fn fetch_data<D: KeyedData + Sync + 'static>(&self) -> StorageResult<Option<D>>;
    async fn insert_data<D: KeyedData + Sync + 'static>(&mut self, data: &D) -> StorageResult<()>;
    async fn upsert_data<D: KeyedData + Sync + 'static>(&mut self, data: &D) -> StorageResult<()>;
    async fn delete_data<D: KeyedData + Sync + 'static>(&mut self) -> StorageResult<()>;

    async fn insert_credentials(
        &mut self,
        timestamp: DateTime<Utc>,
        credentials: Vec<(CredentialWithMetadata, AttestationPresentation)>,
    ) -> StorageResult<()>;

    async fn update_credentials(
        &mut self,
        timestamp: DateTime<Utc>,
        credentials: Vec<(IssuedCredentialCopies, AttestationPresentation)>,
    ) -> StorageResult<()>;

    async fn increment_attestation_copies_usage_count(&mut self, attestation_copy_ids: Vec<Uuid>) -> StorageResult<()>;

    async fn has_any_attestations(&self) -> StorageResult<bool>;
    async fn has_any_attestations_with_types(&self, attestation_types: &[String]) -> StorageResult<bool>;

    async fn fetch_unique_attestations(&self) -> StorageResult<Vec<StoredAttestationCopy>>;

    /// Returns a single attestation copy of each stored attestation for which the attestation type is equal to one of
    /// types requested. The format of the copy returned is undetermined.
    async fn fetch_unique_attestations_by_types<'a>(
        &'a self,
        attestation_types: &HashSet<&'a str>,
    ) -> StorageResult<Vec<StoredAttestationCopy>>;

    /// Returns a single attestation copy of each stored attestation for which the attestation type is equal to
    /// one of types requested and for which at least one copy of the requested format exists. The returned copy
    /// will be of the requested format.
    ///
    /// Additionally, if `CredentialFormat::SdJwt` is requested, the returned attestation copies will also include those
    /// that extend at least one of the requested attestation types.
    async fn fetch_unique_attestations_by_types_and_format<'a>(
        &self,
        attestation_types: &HashSet<&'a str>,
        format: CredentialFormat,
    ) -> StorageResult<Vec<StoredAttestationCopy>>;

    /// Returns a single valid attestation copy of each stored attestation for which the attestation type is equal to
    /// one of types requested and for which at least one copy of the requested format exists. The returned copy
    /// will be of the requested format. Valid in this context means describes the revocation status.
    ///
    /// Additionally, if `CredentialFormat::SdJwt` is requested, the returned attestation copies will also include those
    /// that extend at least one of the requested attestation types.
    #[cfg_attr(test, mockall::concretize)]
    async fn fetch_valid_unique_attestations_by_types_and_format<T>(
        &self,
        attestation_types: &HashSet<&str>,
        format: CredentialFormat,
        time_generator: T,
    ) -> StorageResult<Vec<StoredAttestationCopy>>
    where
        T: Generator<DateTime<Utc>> + Send + Send + Sync + 'static;

    async fn log_disclosure_event(
        &mut self,
        timestamp: DateTime<Utc>,
        proposed_attestation_presentations: Vec<AttestationPresentation>,
        reader_certificate: BorrowingCertificate,
        status: DisclosureStatus,
        r#type: DisclosureType,
    ) -> StorageResult<()>;

    async fn fetch_wallet_events(&self) -> StorageResult<Vec<WalletEvent>>;
    async fn fetch_recent_wallet_events(&self) -> StorageResult<Vec<WalletEvent>>;
    async fn fetch_wallet_events_by_attestation_id(&self, attestation_id: Uuid) -> StorageResult<Vec<WalletEvent>>;
    async fn did_share_data_with_relying_party(&self, certificate: &BorrowingCertificate) -> StorageResult<bool>;

    async fn fetch_all_revocation_info<T>(&self, time_generator: &T) -> StorageResult<Vec<RevocationInfo>>
    where
        T: Generator<DateTime<Utc>> + Send + Send + Sync + 'static;
    async fn update_revocation_statuses(&self, updates: Vec<(Uuid, RevocationStatus)>) -> StorageResult<()>;
}
