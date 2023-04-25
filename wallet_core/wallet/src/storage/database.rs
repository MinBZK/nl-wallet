use std::path::PathBuf;

use anyhow::Result;
use platform_support::{hw_keystore::PlatformEncryptionKey, utils::PlatformUtilities};
use sea_orm::{ConnectOptions, ConnectionTrait, DatabaseConnection};
use tokio::fs;
use wallet_migration::{Migrator, MigratorTrait};

use super::{
    key_file::{delete_key_file, get_or_create_key_file},
    sql_cipher_key::SqlCipherKey,
};

const PRAGMA_KEY: &str = "key";

pub struct Database {
    pub name: String,
    path: PathBuf,
    connection: DatabaseConnection,
}

impl Database {
    fn new(name: String, path: PathBuf, connection: DatabaseConnection) -> Self {
        Database { name, path, connection }
    }

    pub async fn open<K: PlatformEncryptionKey, U: PlatformUtilities>(name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        // Get path to database, stored as "<storage_path>/<name>.db"
        let path = U::storage_path()?.join(format!("{}.db", &name));
        // Open database connection and set database key
        let connection_options = ConnectOptions::new(format!("sqlite://{}?mode=rwc", path.to_str().unwrap()));
        let connection = sea_orm::Database::connect(connection_options).await?;

        // Get database key of the correct length including a salt, stored in encrypted file.
        let key_bytes = get_or_create_key_file::<K, U>(&name, SqlCipherKey::size_with_salt()).await?;
        let key = SqlCipherKey::try_from(key_bytes.as_slice())?;
        // Set database password using PRAGMA statement
        connection
            .execute_unprepared(&format!(r#"PRAGMA {} = "{}";"#, PRAGMA_KEY, String::from(key)))
            .await?;

        // Execute all migrations
        Migrator::up(&connection, None).await?;

        Ok(Self::new(name, path, connection))
    }

    /// If the database could not be closed for some reason, this will return
    /// another instance of [`Database`] as the first entry of the Result error tuple.
    /// Closing and deleting the database may then be tried at another point in time.
    pub async fn close_and_delete<U: PlatformUtilities>(self) -> Result<()> {
        // Close the database connection
        self.connection.close().await?;

        // Remove the database file and ignore any errors.
        _ = fs::remove_file(&self.path).await;

        // Delete the password file and remap errors (note that deletion errors will be ignored).
        delete_key_file::<U>(&self.name).await
    }
}

#[cfg(test)]
mod tests {
    use platform_support::{hw_keystore::software::SoftwareEncryptionKey, utils::software::SoftwareUtilities};
    use sea_orm::{DatabaseBackend, DbBackend, Statement, Value};

    use super::*;

    async fn delete_database<U: PlatformUtilities>(name: &str) -> Result<()> {
        let path = U::storage_path()?.join(format!("{}.db", name));
        _ = fs::remove_file(&path).await;
        delete_key_file::<U>(name).await?;

        Ok(())
    }

    #[derive(Debug, PartialEq, Eq)]
    struct Person {
        id: i32,
        name: String,
        data: Option<Vec<u8>>,
    }

    #[tokio::test]
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
            .execute_unprepared(
                "CREATE TABLE person (
                    id    INTEGER PRIMARY KEY,
                    name  TEXT NOT NULL,
                    data  BLOB
                )",
            )
            .await
            .expect("Could not create table");

        // Create and insert our test Person.
        let me = Person {
            id: 1337,
            name: "Willeke".to_string(),
            data: None,
        };
        db.connection
            .execute(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                "INSERT INTO person (id, name, data) VALUES ($1, $2, $3)",
                [
                    me.id.into(),
                    me.name.clone().into(),
                    Value::Bytes(me.data.clone().map(Box::new)),
                ],
            ))
            .await
            .expect("Could not insert person");

        // Query our person table for any [Person]s.
        let person_query_results = db
            .connection
            .query_all(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT id, name, data FROM person".to_string(),
            ))
            .await
            .expect("Could not execute select statement");

        // Map our query results back to our [Person] model.
        let persons: Vec<Person> = person_query_results
            .into_iter()
            .map(|row| Person {
                id: row.try_get("", "id").unwrap(),
                name: row.try_get("", "name").unwrap(),
                data: row.try_get("", "data").unwrap(),
            })
            .collect();

        // Verify our test [Person] was correctly inserted.
        assert_eq!(persons, [me]);

        db.close_and_delete::<SoftwareUtilities>()
            .await
            .expect("Could not close and delete database");
    }
}
