use std::{path::PathBuf, str::FromStr};

use anyhow::Result;
use platform_support::{hw_keystore::PlatformEncryptionKey, utils::PlatformUtilities};
use rusqlite::Connection;
use tokio::{fs, task::block_in_place};

use super::{
    key_file::{delete_key_file, get_or_create_key_file},
    sql_cipher_key::Aes256SqlCipherKey,
};

const PRAGMA_KEY: &str = "key";

pub struct Database {
    pub name: String,
    connection: Connection,
}

impl Database {
    fn new(name: String, connection: Connection) -> Self {
        Database { name, connection }
    }

    pub async fn open<K: PlatformEncryptionKey, U: PlatformUtilities>(name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        // Get path to database, stored as "<storage_path>/<name>.db"
        let path = U::storage_path()?.join(format!("{}.db", &name));
        // Get database key of the correct length including a salt, stored in encrypted file.
        let key_bytes = get_or_create_key_file::<K, U>(&name, Aes256SqlCipherKey::size_with_salt()).await?;
        let key = Aes256SqlCipherKey::try_from(key_bytes.as_slice())?;
        // Open database connection
        let connection = block_in_place(|| Connection::open(path))?;

        // Set database password using PRAGMA statement
        block_in_place(|| connection.pragma_update(None, PRAGMA_KEY, key))?;

        Ok(Self::new(name, connection))
    }

    /// If the database could not be closed for some reason, this will return
    /// another instance of [`Database`] as the first entry of the Result error tuple.
    /// Closing and deleting the database may then be tried at another point in time.
    pub async fn close_and_delete<U: PlatformUtilities>(
        self,
    ) -> std::result::Result<(), (Option<Self>, anyhow::Error)> {
        // Get the path from the database connection, assume it has one.
        let path = PathBuf::from_str(self.connection.path().unwrap()).unwrap();

        // Close the database connection, return a new Database instance if we could not close.
        let result = block_in_place(|| self.connection.close());
        if result.is_err() {
            return result
                .map_err(|(connection, error)| (Some(Self::new(self.name, connection)), anyhow::Error::new(error)));
        }

        // Remove the database file and ignore any errors.
        _ = fs::remove_file(&path).await;

        // Delete the password file and remap errors (note that deletion errors will be ignored).
        delete_key_file::<U>(&self.name).await.map_err(|e| (None, e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use platform_support::{hw_keystore::software::SoftwareEncryptionKey, utils::software::SoftwareUtilities};
    use rusqlite::params;

    use super::*;

    async fn delete_database<U: PlatformUtilities>(name: &str) -> Result<()> {
        let path = U::storage_path()?.join(format!("{}.db", name));
        _ = fs::remove_file(&path).await;
        delete_key_file::<U>(name).await?;

        Ok(())
    }

    struct Person {
        id: i32,
        name: String,
        data: Option<Vec<u8>>,
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_database() {
        let db_name = "test_db";

        // Make sure we start with a clean slate.
        delete_database::<SoftwareUtilities>(db_name).await.unwrap();

        // Create a new (encrypted) database.
        let db = Database::open::<SoftwareEncryptionKey, SoftwareUtilities>(db_name)
            .await
            .expect("Could not open database");

        // Create a table for our [Person] model.
        db.connection
            .execute(
                "CREATE TABLE person (
                    id    INTEGER PRIMARY KEY,
                    name  TEXT NOT NULL,
                    data  BLOB
                )",
                [],
            )
            .expect("Could not create table");

        // Create and insert our test Person.
        let me = Person {
            id: 1337,
            name: "Willeke".to_string(),
            data: None,
        };
        db.connection
            .execute(
                "INSERT INTO person (id, name, data) VALUES (?1, ?2, ?3)",
                params![&me.id, &me.name, &me.data],
            )
            .expect("Could not insert person");

        {
            // Query our person table for any [Person]s.
            let mut stmt = db
                .connection
                .prepare("SELECT id, name, data FROM person")
                .expect("Could not execute select statement");

            // Map our query results back to our [Person] model.
            let person_iter = stmt
                .query_map([], |row| {
                    Ok(Person {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        data: row.get(2)?,
                    })
                })
                .expect("Could not create iterator");

            // Verify our test [Person] was correctly inserted.
            let mut person_count = 0;
            for person in person_iter {
                let result = person.unwrap();
                assert_eq!(1337, result.id);
                assert_eq!("Willeke", result.name);
                assert_eq!(None, result.data);
                person_count += 1;
            }

            // Verify we really checked our test person (and did not loop over empty iterator).
            assert_eq!(person_count, 1);
        }

        db.close_and_delete::<SoftwareUtilities>()
            .await
            .map_err(|(_, e)| e)
            .expect("Could not close and delete database");
    }
}
