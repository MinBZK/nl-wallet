mod attestation_copy;
mod data;
mod database;
mod database_storage;
mod event_log;
mod key_file;
mod revocation_info;
mod sql_cipher_key;

use std::array::TryFromSliceError;
use std::collections::HashSet;
use std::io;

use attestation_data::auth::Organization;
use attestation_data::disclosure_type::DisclosureType;
use attestation_types::credential_kind::CredentialKind;
use chrono::DateTime;
use chrono::Utc;
#[cfg(any(test, feature = "test"))]
pub use database_storage::test_storage::MockHardwareDatabaseStorage;
use derive_more::Constructor;
use error_category::ErrorCategory;
use mdoc::utils::cose::CoseError;
use mdoc::utils::serialization::CborError;
use openid4vc::wallet_issuance::credential::CredentialWithMetadata;
use openid4vc::wallet_issuance::credential::IssuedCredentialCopies;
use sd_jwt_vc_metadata::TypeMetadataChainError;
use sea_orm::DbErr;
use serde::Deserialize;
use serde::Serialize;
use tempfile::NamedTempFile;
use token_status_list::verification::verifier::RevocationStatus;
use utils::generator::Generator;
use uuid::Uuid;

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
use crate::AttestationPresentation;
use crate::storage::sql_cipher_key::SqlCipherKey;

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
    async fn has_any_attestations_with_credential_kinds(
        &self,
        credential_kinds: &HashSet<CredentialKind>,
    ) -> StorageResult<bool>;

    async fn fetch_unique_attestations(&self) -> StorageResult<Vec<StoredAttestationCopy>>;

    /// Returns a single attestation copy for each stored attestation whose attestation type and format match one of the
    /// requested [`CredentialKind`] instances. Attestations that merely extend one of the requested attestation types
    /// are not returned.
    async fn fetch_unique_attestations_by_credential_kinds(
        &self,
        credential_kinds: &HashSet<CredentialKind>,
    ) -> StorageResult<Vec<StoredAttestationCopy>>;

    /// Returns a single valid attestation copy for each stored attestation whose attestation type and format match one
    /// of the requested [`CredentialKind`] instances. Valid in this context describes the revocation status and the
    /// validity window of the attestation.
    ///
    /// In addition to the attestations of the requested attestation types themselves, the returned attestation copies
    /// include those of which the type metadata extends one of the requested attestation types. Note that this can only
    /// apply to attestations in the SD-JWT format, as mdoc doc types have no extension mechanism.
    #[cfg_attr(test, mockall::concretize)]
    async fn fetch_valid_unique_attestations_by_credential_kinds<T>(
        &self,
        credential_kinds: &HashSet<CredentialKind>,
        time_generator: T,
    ) -> StorageResult<Vec<StoredAttestationCopy>>
    where
        T: Generator<DateTime<Utc>> + Send + Send + Sync + 'static;

    async fn log_disclosure_event(
        &mut self,
        timestamp: DateTime<Utc>,
        proposed_attestation_presentations: Vec<AttestationPresentation>,
        organization: &Organization,
        status: DisclosureStatus,
        r#type: DisclosureType,
    ) -> StorageResult<()>;

    async fn fetch_wallet_events(&self) -> StorageResult<Vec<WalletEvent>>;
    async fn fetch_recent_wallet_events(&self) -> StorageResult<Vec<WalletEvent>>;
    async fn fetch_wallet_events_by_attestation_id(&self, attestation_id: Uuid) -> StorageResult<Vec<WalletEvent>>;
    async fn did_share_data_with_relying_party(&self, organization: &Organization) -> StorageResult<bool>;

    async fn fetch_all_revocation_info<T>(&self, time_generator: &T) -> StorageResult<Vec<RevocationInfo>>
    where
        T: Generator<DateTime<Utc>> + Send + Send + Sync + 'static;
    async fn update_revocation_statuses(&self, updates: Vec<(Uuid, RevocationStatus)>) -> StorageResult<()>;

    /// Returns the attestation kind and the key identifiers of all copies of the attestation with the given id.
    /// Returns `None` if no attestation with that id exists.
    async fn fetch_credential_kind_and_key_identifiers_by_attestation_id(
        &self,
        attestation_id: Uuid,
    ) -> StorageResult<Option<(CredentialKind, Vec<String>)>>;

    /// Deletes all copies of the attestation with the given id, severs the links from history events
    /// to the attestation, and deletes the attestation itself. Does nothing if no attestation with
    /// that id exists.
    async fn delete_attestation(&mut self, timestamp: DateTime<Utc>, attestation_id: Uuid) -> StorageResult<()>;
}
