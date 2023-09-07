use std::{marker::PhantomData, path::PathBuf};

use async_trait::async_trait;
use platform_support::utils::PlatformUtilities;
use sea_orm::{sea_query::Expr, ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tokio::fs;

use entity::keyed_data;
use wallet_common::keys::SecureEncryptionKey;

use super::{
    data::KeyedData,
    database::{Database, SqliteUrl},
    key_file::{delete_key_file, get_or_create_key_file},
    sql_cipher_key::SqlCipherKey,
    Storage, StorageError, StorageState,
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
    pub async fn init<U>() -> Result<Self, StorageError>
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
    fn database(&self) -> Result<&Database, StorageError> {
        self.database.as_ref().ok_or(StorageError::NotOpened)
    }

    fn database_path_for_name(&self, name: &str) -> PathBuf {
        // Get path to database as "<storage_path>/<name>.db"
        self.storage_path.join(format!("{}.{}", name, DATABASE_FILE_EXT))
    }

    /// This helper method uses [`get_or_create_key_file`] and the utilities in [`platform_support`]
    /// to construct a [`SqliteUrl`] and a [`SqlCipherKey`], which in turn are used to create a [`Database`]
    /// instance.
    async fn open_encrypted_database(&self, name: &str) -> Result<Database, StorageError> {
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
}

#[async_trait]
impl<K> Storage for DatabaseStorage<K>
where
    K: SecureEncryptionKey + Send + Sync,
{
    /// Indicate whether there is no database on disk, there is one but it is unopened
    /// or the database is currently open.
    async fn state(&self) -> Result<StorageState, StorageError> {
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
    async fn open(&mut self) -> Result<(), StorageError> {
        if self.database.is_some() {
            return Err(StorageError::AlreadyOpened);
        }

        let database = self.open_encrypted_database(DATABASE_NAME).await?;
        self.database.replace(database);

        Ok(())
    }

    /// Clear the contents of the database by closing it and removing both database and key file.
    async fn clear(&mut self) -> Result<(), StorageError> {
        // Take the Database from the Option<> so that close_and_delete() can consume it.
        let database = self.database.take().ok_or(StorageError::NotOpened)?;
        let key_file_alias = key_file_alias_for_name(DATABASE_NAME);

        // Close and delete the database, only if this succeeds also delete the key file.
        database.close_and_delete().await?;
        delete_key_file(&self.storage_path, &key_file_alias).await;

        Ok(())
    }

    /// Get data entry from the key-value table, if present.
    async fn fetch_data<D: KeyedData>(&self) -> Result<Option<D>, StorageError> {
        let database = self.database()?;

        let data = keyed_data::Entity::find_by_id(D::KEY)
            .one(database.connection())
            .await?
            .map(|m| serde_json::from_value::<D>(m.data))
            .transpose()?;

        Ok(data)
    }

    /// Insert data entry in the key-value table, which will return an error when one is already present.
    async fn insert_data<D: KeyedData + Sync>(&mut self, data: &D) -> Result<(), StorageError> {
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
    async fn update_data<D: KeyedData + Sync>(&mut self, data: &D) -> Result<(), StorageError> {
        let database = self.database()?;

        keyed_data::Entity::update_many()
            .col_expr(keyed_data::Column::Data, Expr::value(serde_json::to_value(data)?))
            .filter(keyed_data::Column::Key.eq(D::KEY.to_string()))
            .exec(database.connection())
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use platform_support::utils::software::SoftwareUtilities;
    use tokio::fs;

    use wallet_common::{
        account::messages::auth::WalletCertificate, keys::software::SoftwareEncryptionKey, utils::random_bytes,
    };

    use crate::storage::data::RegistrationData;

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

    #[tokio::test]
    async fn test_database_storage() {
        let registration = RegistrationData {
            pin_salt: vec![1, 2, 3, 4].into(),
            wallet_certificate: WalletCertificate::from("thisisdefinitelyvalid"),
            instruction_sequence_number: 1,
        };

        let mut storage = DatabaseStorage::<SoftwareEncryptionKey>::init::<SoftwareUtilities>()
            .await
            .unwrap();

        // Create a test database, override the database field on Storage.
        let key_bytes = random_bytes(SqlCipherKey::size_with_salt());
        let database = Database::open(SqliteUrl::InMemory, key_bytes.as_slice().try_into().unwrap())
            .await
            .expect("Could not open in-memory database");
        storage.database = Some(database);

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
        assert_eq!(
            fetched_registration.instruction_sequence_number,
            registration.instruction_sequence_number
        );

        // Save the registration again, should result in an error.
        let save_result = storage.insert_data(&registration).await;
        assert!(save_result.is_err());

        // Update registration
        let updated_registration = RegistrationData {
            pin_salt: registration.pin_salt.clone(),
            wallet_certificate: registration.wallet_certificate.clone(),
            instruction_sequence_number: registration.instruction_sequence_number + 1,
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
        assert_eq!(fetched_after_update_registration.pin_salt.0, registration.pin_salt.0);
        assert_eq!(
            fetched_after_update_registration.wallet_certificate.0,
            registration.wallet_certificate.0
        );
        assert_eq!(
            fetched_after_update_registration.instruction_sequence_number,
            registration.instruction_sequence_number + 1,
        );

        // Clear database, state should be uninitialized.

        storage.clear().await.expect("Could not clear storage");

        let state = storage.state().await.unwrap();
        assert!(matches!(state, StorageState::Uninitialized));
    }
}
