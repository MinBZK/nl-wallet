use std::path::PathBuf;

use async_trait::async_trait;
use futures::TryFutureExt;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use tokio::{fs, try_join};

use entity::keyed_data;
use platform_support::{
    hw_keystore::PlatformEncryptionKey,
    preferred,
    utils::{PlatformUtilities, UtilitiesError},
};

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

fn database_path_for_name<U: PlatformUtilities>(name: &str) -> Result<PathBuf, UtilitiesError> {
    // Get path to database as "<storage_path>/<name>.db"
    let storage_path = U::storage_path()?;
    let database_path = storage_path.join(format!("{}.{}", name, DATABASE_FILE_EXT));

    Ok(database_path)
}

/// This helper function uses [`get_or_create_key_file`] and the utilities in [`platform_support`]
/// to construct a [`SqliteUrl`] and a [`SqlCipherKey`], which in turn are used to create a [`Database`]
/// instance.
async fn open_encrypted_database<K: PlatformEncryptionKey, U: PlatformUtilities>(
    name: &str,
) -> Result<Database, StorageError> {
    let key_file_alias = key_file_alias_for_name(name);
    let database_path = database_path_for_name::<U>(name)?;

    // Get database key of the correct length including a salt, stored in encrypted file.
    let key_bytes = get_or_create_key_file::<K, U>(&key_file_alias, SqlCipherKey::size_with_salt()).await?;
    let key = SqlCipherKey::try_from(key_bytes.as_slice())?;

    // Open database at the path, encrypted using the key
    let database = Database::open(SqliteUrl::File(database_path), key).await?;

    Ok(database)
}

/// This is the implemtation of [`Storage`] as used by the [`crate::Wallet`]. Its responsibilities are:
///
/// * Managing the lifetime of one or more [`Database`] instances by combining its functionality with
///   encrypted key files. This also includes deleting the database and key file when the [`clear`]
///   method is called.
/// * Communicating the current state of the database through the [`state`] method.
/// * Executing queries on the database by accepting / returning data structures that are used by
///   [`crate::Wallet`].
#[derive(Debug)]
pub struct DatabaseStorage {
    database: Option<Database>,
}

impl DatabaseStorage {
    fn new(database: Option<Database>) -> Self {
        DatabaseStorage { database }
    }

    // Helper method, should be called before accessing database.
    fn database(&self) -> Result<&Database, StorageError> {
        self.database.as_ref().ok_or(StorageError::NotOpened)
    }
}

// The ::default() method is the primary way of instantiating DatabaseStorage.
impl Default for DatabaseStorage {
    fn default() -> Self {
        Self::new(None)
    }
}

#[async_trait]
impl Storage for DatabaseStorage {
    /// Indiciate whether there is no database on disk, there is one but it is unopened
    /// or the database is currently open.
    async fn state(&self) -> Result<StorageState, StorageError> {
        if self.database.is_some() {
            return Ok(StorageState::Opened);
        }

        let database_path = database_path_for_name::<preferred::PlatformUtilities>(DATABASE_NAME)?;

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

        let database =
            open_encrypted_database::<preferred::PlatformEncryptionKey, preferred::PlatformUtilities>(DATABASE_NAME)
                .await?;
        self.database.replace(database);

        Ok(())
    }

    /// Clear the contents of the database by closing it and removing both database and key file.
    async fn clear(&mut self) -> Result<(), StorageError> {
        // Take the Database from the Option<> so that close_and_delete() can consume it.
        let database = self.database.take().ok_or(StorageError::NotOpened)?;
        let key_file_alias = key_file_alias_for_name(DATABASE_NAME);

        // Delete the database and key file in parallel
        try_join!(
            database.close_and_delete().map_err(StorageError::from),
            delete_key_file::<preferred::PlatformUtilities>(&key_file_alias).map_err(StorageError::from)
        )
        .map(|_| ())
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
    async fn insert_data<D: KeyedData>(&mut self, data: &D) -> Result<(), StorageError> {
        let database = self.database()?;

        let _ = keyed_data::ActiveModel {
            key: Set(D::KEY.to_string()),
            data: Set(serde_json::to_value(data)?),
        }
        .insert(database.connection())
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use platform_support::{hw_keystore::software::SoftwareEncryptionKey, utils::software::SoftwareUtilities};
    use tokio::fs;

    use wallet_common::{account::auth::WalletCertificate, utils::random_bytes};

    use crate::storage::data::RegistrationData;

    use super::*;

    #[test]
    fn test_key_file_alias_for_name() {
        assert_eq!(key_file_alias_for_name("test_database"), "test_database_db");
    }

    #[tokio::test]
    async fn test_open_encrypted_database() {
        let name = "test_open_encrypted_database";
        let key_file_alias = key_file_alias_for_name(name);
        let database_path = database_path_for_name::<SoftwareUtilities>(name).unwrap();

        // Make sure we start with a clean slate.
        delete_key_file::<SoftwareUtilities>(&key_file_alias).await.unwrap();
        _ = fs::remove_file(database_path).await;

        let database = open_encrypted_database::<SoftwareEncryptionKey, SoftwareUtilities>(name)
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
        };

        // Create a test database, pass it to the private new() constructor.
        let key_bytes = random_bytes(SqlCipherKey::size_with_salt());
        let database = Database::open(SqliteUrl::InMemory, key_bytes.as_slice().try_into().unwrap())
            .await
            .expect("Could not open in-memory database");
        let mut storage = DatabaseStorage::new(Some(database));

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

        // Clear database, state should be uninitialized.

        storage.clear().await.expect("Could not clear storage");

        let state = storage.state().await.unwrap();
        assert!(matches!(state, StorageState::Uninitialized));
    }
}
