use anyhow::Result;
use platform_support::{hw_keystore::PlatformEncryptionKey, utils::PlatformUtilities};
use tokio::try_join;

use crate::storage::key_file::delete_key_file;

use super::{
    database::{Database, SqliteUrl},
    key_file::get_or_create_key_file,
    sql_cipher_key::SqlCipherKey,
};

const KEY_FILE_SUFFIX: &str = "_db";

fn key_file_alias_for_name(name: &str) -> String {
    format!("{}{}", name, KEY_FILE_SUFFIX)
}

struct WalletDatabase {
    pub name: String,
    database: Database,
}

impl WalletDatabase {
    fn new(name: String, database: Database) -> Self {
        WalletDatabase { name, database }
    }

    pub async fn open<K: PlatformEncryptionKey, U: PlatformUtilities>(name: impl Into<String>) -> Result<Self> {
        // Get path to database, stored as "<storage_path>/<name>.db"
        let name = name.into();
        let path = U::storage_path()?.join(format!("{}.db", &name));

        // Get database key of the correct length including a salt, stored in encrypted file.
        let key_bytes =
            get_or_create_key_file::<K, U>(&key_file_alias_for_name(&name), SqlCipherKey::size_with_salt()).await?;
        let key = SqlCipherKey::try_from(key_bytes.as_slice())?;

        // Open database at the path, encrypted using the key
        let database = Database::open(SqliteUrl::File(path), key).await?;

        Ok(Self::new(name, database))
    }

    pub async fn close_and_delete<U: PlatformUtilities>(self) -> Result<()> {
        let key_file_alias = key_file_alias_for_name(&self.name);
        try_join!(self.database.close_and_delete(), delete_key_file::<U>(&key_file_alias)).map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use platform_support::{hw_keystore::software::SoftwareEncryptionKey, utils::software::SoftwareUtilities};
    use tokio::fs;

    use super::*;

    #[tokio::test]
    async fn test_wallet_database() {
        let name = "test_wallet_database";

        // Make sure we start with a clean slate.
        let database_path = SoftwareUtilities::storage_path().unwrap().join(format!("{}.db", name));
        let key_file_path = SoftwareUtilities::storage_path()
            .unwrap()
            .join(format!("{}_db.key", name));
        _ = try_join!(fs::remove_file(&database_path), fs::remove_file(&key_file_path));

        // The database file and key file should not exist.
        assert!(!fs::try_exists(&database_path).await.unwrap());
        assert!(!fs::try_exists(&key_file_path).await.unwrap());

        let database = WalletDatabase::open::<SoftwareEncryptionKey, SoftwareUtilities>(name)
            .await
            .expect("Could not open database");

        // The database file and key file should have been created.
        assert!(fs::try_exists(&database_path).await.unwrap());
        assert!(fs::try_exists(&key_file_path).await.unwrap());

        database
            .close_and_delete::<SoftwareUtilities>()
            .await
            .expect("Could not close and delete database");

        // The database file and key file should have been removed.
        assert!(!fs::try_exists(&database_path).await.unwrap());
        assert!(!fs::try_exists(&key_file_path).await.unwrap());
    }
}
