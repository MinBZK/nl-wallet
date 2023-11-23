use std::{collections::HashSet, marker::PhantomData, path::PathBuf};

use async_trait::async_trait;
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter, QuerySelect, Select, Set,
    TransactionTrait,
};
use tokio::fs;
use tracing::info;
use uuid::Uuid;

use entity::{event_log, keyed_data, mdoc, mdoc_copy};
use nl_wallet_mdoc::{
    holder::{Mdoc, MdocCopies},
    utils::serialization::CborError,
    utils::serialization::{cbor_deserialize, cbor_serialize},
};
use platform_support::utils::PlatformUtilities;
use wallet_common::keys::SecureEncryptionKey;

use super::{
    data::KeyedData,
    database::{Database, SqliteUrl},
    key_file::{delete_key_file, get_or_create_key_file},
    sql_cipher_key::SqlCipherKey,
    Storage, StorageError, StorageResult, StorageState, WalletEvent,
};

const DATABASE_NAME: &str = "wallet";
const KEY_FILE_SUFFIX: &str = "_db";
const DATABASE_FILE_EXT: &str = "db";

fn key_file_alias_for_name(database_name: &str) -> String {
    // Append suffix to database name to get key file alias
    format!("{}{}", database_name, KEY_FILE_SUFFIX)
}

/// This is the implementation of [`Storage`] as used by the [`crate::Wallet`]. Its responsibilities are:
///
/// * Managing the lifetime of one or more [`Database`] instances by combining its functionality with
///   encrypted key files. This also includes deleting the database and key file when the [`clear`]
///   method is called.
/// * Communicating the current state of the database through the [`state`] method.
/// * Executing queries on the database by accepting / returning data structures that are used by
///   [`crate::Wallet`].
#[derive(Debug)]
pub struct DatabaseStorage<K> {
    storage_path: PathBuf,
    database: Option<Database>,
    _key: PhantomData<K>,
}

impl<K> DatabaseStorage<K> {
    pub async fn init<U>() -> StorageResult<Self>
    where
        U: PlatformUtilities,
    {
        let storage_path = U::storage_path().await?;

        let storage = DatabaseStorage {
            storage_path,
            database: None,
            _key: PhantomData,
        };

        Ok(storage)
    }
}

impl<K> DatabaseStorage<K>
where
    K: SecureEncryptionKey,
{
    // Helper method, should be called before accessing database.
    fn database(&self) -> StorageResult<&Database> {
        self.database.as_ref().ok_or(StorageError::NotOpened)
    }

    fn database_path_for_name(&self, name: &str) -> PathBuf {
        // Get path to database as "<storage_path>/<name>.db"
        self.storage_path.join(format!("{}.{}", name, DATABASE_FILE_EXT))
    }

    /// This helper method uses [`get_or_create_key_file`] and the utilities in [`platform_support`]
    /// to construct a [`SqliteUrl`] and a [`SqlCipherKey`], which in turn are used to create a [`Database`]
    /// instance.
    async fn open_encrypted_database(&self, name: &str) -> StorageResult<Database> {
        let key_file_alias = key_file_alias_for_name(name);
        let database_path = self.database_path_for_name(name);

        // Get database key of the correct length including a salt, stored in encrypted file.
        let key_bytes =
            get_or_create_key_file::<K>(&self.storage_path, &key_file_alias, SqlCipherKey::size_with_salt()).await?;
        let key = SqlCipherKey::try_from(key_bytes.as_slice())?;

        // Open database at the path, encrypted using the key
        let database = Database::open(SqliteUrl::File(database_path), key).await?;

        Ok(database)
    }

    async fn query_unique_mdocs<F>(&self, transform_select: F) -> StorageResult<Vec<(Uuid, Mdoc)>>
    where
        F: FnOnce(Select<mdoc_copy::Entity>) -> Select<mdoc_copy::Entity>,
    {
        let database = self.database()?;

        let select = mdoc_copy::Entity::find()
            .select_only()
            .column_as(mdoc_copy::Column::Id.min(), "id")
            .column(mdoc_copy::Column::MdocId)
            .column(mdoc_copy::Column::Mdoc)
            .group_by(mdoc_copy::Column::MdocId);

        let mdoc_copies = transform_select(select).all(database.connection()).await?;

        let mdocs = mdoc_copies
            .into_iter()
            .map(|model| {
                let mdoc = cbor_deserialize(model.mdoc.as_slice())?;

                Ok((model.mdoc_id, mdoc))
            })
            .collect::<Result<_, CborError>>()?;

        Ok(mdocs)
    }
}

#[async_trait]
impl<K> Storage for DatabaseStorage<K>
where
    K: SecureEncryptionKey + Send + Sync,
{
    /// Indicate whether there is no database on disk, there is one but it is unopened
    /// or the database is currently open.
    async fn state(&self) -> StorageResult<StorageState> {
        if self.database.is_some() {
            return Ok(StorageState::Opened);
        }

        let database_path = self.database_path_for_name(DATABASE_NAME);

        if fs::try_exists(database_path).await? {
            return Ok(StorageState::Unopened);
        }

        Ok(StorageState::Uninitialized)
    }

    /// Load a database, creating a new key file and database file if necessary.
    async fn open(&mut self) -> StorageResult<()> {
        if self.database.is_some() {
            return Err(StorageError::AlreadyOpened);
        }

        let database = self.open_encrypted_database(DATABASE_NAME).await?;
        self.database.replace(database);

        Ok(())
    }

    /// Clear the contents of the database by closing it and removing both database and key file.
    async fn clear(&mut self) -> StorageResult<()> {
        // Take the Database from the Option<> so that close_and_delete() can consume it.
        let database = self.database.take().ok_or(StorageError::NotOpened)?;
        let key_file_alias = key_file_alias_for_name(DATABASE_NAME);

        // Close and delete the database, only if this succeeds also delete the key file.
        database.close_and_delete().await?;
        delete_key_file(&self.storage_path, &key_file_alias).await;

        Ok(())
    }

    /// Get data entry from the key-value table, if present.
    async fn fetch_data<D: KeyedData>(&self) -> StorageResult<Option<D>> {
        let database = self.database()?;

        let data = keyed_data::Entity::find_by_id(D::KEY)
            .one(database.connection())
            .await?
            .map(|m| serde_json::from_value::<D>(m.data))
            .transpose()?;

        Ok(data)
    }

    /// Insert data entry in the key-value table, which will return an error when one is already present.
    async fn insert_data<D: KeyedData + Sync>(&mut self, data: &D) -> StorageResult<()> {
        let database = self.database()?;

        let _ = keyed_data::ActiveModel {
            key: Set(D::KEY.to_string()),
            data: Set(serde_json::to_value(data)?),
        }
        .insert(database.connection())
        .await?;

        Ok(())
    }

    /// Update data entry in the key-value table using the provided key.
    async fn update_data<D: KeyedData + Sync>(&mut self, data: &D) -> StorageResult<()> {
        let database = self.database()?;

        keyed_data::Entity::update_many()
            .col_expr(keyed_data::Column::Data, Expr::value(serde_json::to_value(data)?))
            .filter(keyed_data::Column::Key.eq(D::KEY.to_string()))
            .exec(database.connection())
            .await?;

        Ok(())
    }

    async fn insert_mdocs(&mut self, mdocs: Vec<MdocCopies>) -> StorageResult<()> {
        let database = self.database()?;

        let transaction = database.connection().begin().await?;

        // Construct a vec of tuples of 1 `mdoc` and 1 or more `mdoc_copy` models,
        // based on the unique `MdocCopies`, to be inserted into the database.
        let mdoc_models = mdocs
            .into_iter()
            .filter(|mdoc_copies| !mdoc_copies.cred_copies.is_empty())
            .map(|mdoc_copies| {
                let mdoc_id = Uuid::new_v4();

                let copy_models = mdoc_copies
                    .cred_copies
                    .iter()
                    .map(|mdoc| {
                        let model = mdoc_copy::ActiveModel {
                            id: Set(Uuid::new_v4()),
                            mdoc_id: Set(mdoc_id),
                            mdoc: Set(cbor_serialize(&mdoc)?),
                        };

                        Ok(model)
                    })
                    .collect::<Result<Vec<_>, CborError>>()?;

                // `mdoc_copies.cred_copies` is guaranteed to contain at least one value because of the filter() above.
                let doc_type = mdoc_copies.cred_copies.into_iter().next().unwrap().doc_type;
                let mdoc_model = mdoc::ActiveModel {
                    id: Set(mdoc_id),
                    doc_type: Set(doc_type),
                };

                Ok((mdoc_model, copy_models))
            })
            .collect::<Result<Vec<_>, CborError>>()?;

        // Make two separate vecs out of the vec of tuples.
        let (mdoc_models, copy_models): (Vec<_>, Vec<_>) = mdoc_models.into_iter().unzip();

        mdoc::Entity::insert_many(mdoc_models).exec(&transaction).await?;
        mdoc_copy::Entity::insert_many(copy_models.into_iter().flatten())
            .exec(&transaction)
            .await?;

        transaction.commit().await?;

        Ok(())
    }

    async fn fetch_unique_mdocs(&self) -> StorageResult<Vec<(Uuid, Mdoc)>> {
        self.query_unique_mdocs(|select| select).await
    }

    async fn fetch_unique_mdocs_by_doctypes(&self, doc_types: &HashSet<&str>) -> StorageResult<Vec<(Uuid, Mdoc)>> {
        let doc_types_iter = doc_types.iter().copied();

        self.query_unique_mdocs(move |select| {
            select
                .inner_join(mdoc::Entity)
                .filter(mdoc::Column::DocType.is_in(doc_types_iter))
        })
        .await
    }

    async fn log_wallet_events(&mut self, events: Vec<WalletEvent>) -> StorageResult<()> {
        let entities: Vec<_> = events
            .into_iter()
            .map(|event| {
                // log mdoc subject, when conversion to X509Certificate succeeds
                if let Ok(certificate) = event.remote_party_certificate.to_x509() {
                    let subject = certificate.subject().to_string();
                    info!("Logging PID issued by: {}", subject);
                }

                use ActiveValue::*;
                event_log::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    event_type: Set(event.event_type),
                    timestamp: Set(event.timestamp),
                    remote_party_certificate: Set(event.remote_party_certificate.as_bytes().to_owned()),
                    status_description: event.status.description().map(|d| Set(Some(d))).unwrap_or(NotSet),
                    status: Set(event.status.into()),
                }
            })
            .collect();
        event_log::Entity::insert_many(entities)
            .exec(self.database()?.connection())
            .await?;
        Ok(())
    }

    async fn fetch_wallet_events(&self) -> StorageResult<Vec<WalletEvent>> {
        let events = event_log::Entity::find().all(self.database()?.connection()).await?;
        Ok(events.into_iter().map(|e| e.into()).collect())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use tokio::fs;

    use entity::event_log::EventType;
    use nl_wallet_mdoc::{examples::Examples, mock as mdoc_mock, utils::x509::Certificate};
    use platform_support::utils::software::SoftwareUtilities;
    use wallet_common::{
        account::messages::auth::WalletCertificate, keys::software::SoftwareEncryptionKey, utils::random_bytes,
    };

    use crate::storage::{data::RegistrationData, Status};

    use super::*;

    #[test]
    fn test_key_file_alias_for_name() {
        assert_eq!(key_file_alias_for_name("test_database"), "test_database_db");
    }

    #[tokio::test]
    async fn test_database_open_encrypted_database() {
        let storage = DatabaseStorage::<SoftwareEncryptionKey>::init::<SoftwareUtilities>()
            .await
            .unwrap();

        let name = "test_open_encrypted_database";
        let key_file_alias = key_file_alias_for_name(name);
        let database_path = storage.database_path_for_name(name);

        // Make sure we start with a clean slate.
        delete_key_file(&storage.storage_path, &key_file_alias).await;
        _ = fs::remove_file(database_path).await;

        let database = storage
            .open_encrypted_database(name)
            .await
            .expect("Could not open encrypted database");

        assert!(matches!(&database.url, SqliteUrl::File(path)
            if path.to_str().unwrap().contains("test_open_encrypted_database.db")));

        database
            .close_and_delete()
            .await
            .expect("Could not close and delete database");
    }

    async fn open_test_database_storage() -> DatabaseStorage<SoftwareEncryptionKey> {
        let mut storage = DatabaseStorage::<SoftwareEncryptionKey>::init::<SoftwareUtilities>()
            .await
            .unwrap();

        // Create a test database, override the database field on Storage.
        let key_bytes = random_bytes(SqlCipherKey::size_with_salt());
        let database = Database::open(SqliteUrl::InMemory, key_bytes.as_slice().try_into().unwrap())
            .await
            .expect("Could not open in-memory database");
        storage.database = Some(database);

        storage
    }

    #[tokio::test]
    async fn test_database_storage() {
        let registration = RegistrationData {
            pin_salt: vec![1, 2, 3, 4].into(),
            wallet_certificate: WalletCertificate::from("thisisdefinitelyvalid"),
        };

        let mut storage = open_test_database_storage().await;

        // State should be Opened.
        let state = storage.state().await.unwrap();
        assert!(matches!(state, StorageState::Opened));

        // Try to fetch the registration, none should be there.
        let no_registration = storage
            .fetch_data::<RegistrationData>()
            .await
            .expect("Could not get registration");

        assert!(no_registration.is_none());

        // Save the registration and fetch it again.
        storage
            .insert_data(&registration)
            .await
            .expect("Could not save registration");

        let fetched_registration = storage
            .fetch_data::<RegistrationData>()
            .await
            .expect("Could not get registration");

        assert!(fetched_registration.is_some());
        let fetched_registration = fetched_registration.unwrap();
        assert_eq!(fetched_registration.pin_salt.0, registration.pin_salt.0);
        assert_eq!(
            fetched_registration.wallet_certificate.0,
            registration.wallet_certificate.0
        );

        // Save the registration again, should result in an error.
        let save_result = storage.insert_data(&registration).await;
        assert!(save_result.is_err());

        // Update registration
        let new_salt = random_bytes(64).into();
        let updated_registration = RegistrationData {
            pin_salt: new_salt,
            wallet_certificate: registration.wallet_certificate.clone(),
        };
        storage
            .update_data(&updated_registration)
            .await
            .expect("Could not update registration");

        let fetched_after_update_registration = storage
            .fetch_data::<RegistrationData>()
            .await
            .expect("Could not get registration");
        assert!(fetched_after_update_registration.is_some());
        let fetched_after_update_registration = fetched_after_update_registration.unwrap();
        assert_eq!(
            fetched_after_update_registration.pin_salt.0,
            updated_registration.pin_salt.0
        );
        assert_eq!(
            fetched_after_update_registration.wallet_certificate.0,
            registration.wallet_certificate.0
        );

        // Clear database, state should be uninitialized.
        storage.clear().await.expect("Could not clear storage");

        let state = storage.state().await.unwrap();
        assert!(matches!(state, StorageState::Uninitialized));
    }

    #[tokio::test]
    async fn test_mdoc_storage() {
        let mut storage = open_test_database_storage().await;

        // State should be Opened.
        let state = storage.state().await.unwrap();
        assert!(matches!(state, StorageState::Opened));

        // Create MdocsMap from example Mdoc
        let trust_anchors = Examples::iaca_trust_anchors();
        let mdoc = mdoc_mock::mdoc_from_example_device_response(trust_anchors);
        let mdoc_copies = MdocCopies::from([mdoc.clone(), mdoc.clone(), mdoc].to_vec());

        // Insert mdocs
        storage.insert_mdocs(vec![mdoc_copies.clone()]).await.unwrap();

        // Fetch unique mdocs
        let fetched_unique = storage.fetch_unique_mdocs().await.unwrap();

        // Only one unique `Mdoc` should be returned and it should match all copies.
        assert_eq!(fetched_unique.len(), 1);
        assert_eq!(
            &fetched_unique.first().unwrap().1,
            mdoc_copies.cred_copies.first().unwrap()
        );

        // Fetch unique mdocs based on doctype
        let fetched_unique_doctype = storage
            .fetch_unique_mdocs_by_doctypes(&HashSet::from(["foo", "org.iso.18013.5.1.mDL"]))
            .await
            .unwrap();

        // One matching `Mdoc` should be returned
        assert_eq!(fetched_unique_doctype.len(), 1);
        assert_eq!(
            &fetched_unique_doctype.first().unwrap().1,
            mdoc_copies.cred_copies.first().unwrap()
        );

        // Fetch unique mdocs based on non-existent doctype
        let fetched_unique_doctype_mismatch = storage
            .fetch_unique_mdocs_by_doctypes(&HashSet::from(["foo", "bar"]))
            .await
            .unwrap();

        // No entries should be returned
        assert!(fetched_unique_doctype_mismatch.is_empty());
    }

    #[tokio::test]
    async fn test_event_log_storage() {
        let mut storage = open_test_database_storage().await;

        // State should be Opened.
        let state = storage.state().await.unwrap();
        assert!(matches!(state, StorageState::Opened));

        let (certificate, _) = Certificate::new_ca("test-ca").unwrap();

        let events = vec![WalletEvent::new(
            EventType::Issuance,
            Utc::now(),
            certificate,
            Status::Success,
        )];

        // Insert events
        storage.log_wallet_events(events.clone()).await.unwrap();

        // Fetch and verify events
        assert_eq!(storage.fetch_wallet_events().await.unwrap(), events);
    }
}
