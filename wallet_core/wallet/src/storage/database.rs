use std::io;
use std::path::PathBuf;

use derive_more::Constructor;
use sea_orm::ConnectOptions;
use sea_orm::ConnectionTrait;
use sea_orm::DatabaseConnection;
use sea_orm::DbErr;
use sea_orm::TransactionTrait;
use tokio::fs;

use tracing::log::LevelFilter;

use wallet_migrations::Migrator;
use wallet_migrations::MigratorTrait;

use super::sql_cipher_key::SqlCipherKey;

/// This represents a URL to a SQLite database, either on the filesystem or in memory.
#[derive(Debug, Clone)]
pub enum SqliteUrl {
    File(PathBuf),
    InMemory,
}

/// For some reason Sea-ORM requires the URL to the database as a string intermediary,
/// rather than programmatically. This URL string is encoded by implementing the [`From`] trait.
/// for [`String`] (and conversely the [`Into`] trait for [`SqliteUrl`]).
impl From<&SqliteUrl> for String {
    fn from(value: &SqliteUrl) -> Self {
        match value {
            SqliteUrl::File(path) => format!("sqlite://{}?mode=rwc", path.to_string_lossy()),
            SqliteUrl::InMemory => "sqlite::memory:".to_string(),
        }
    }
}

impl From<SqliteUrl> for String {
    fn from(value: SqliteUrl) -> Self {
        Self::from(&value)
    }
}

/// This struct wraps a SQLite database connection, it has the following responsibilities:
///
/// * Setting up a connection to an encrypted database, based on a [`SqliteUrl`] and [`SqlCipherKey`]
/// * Tearing down a connection to the database, either by falling out of scope or by having [`close_and_delete`] called
///   on it in order to also delete the database file.
/// * Exposing a reference to the database connection as [`ConnectionTrait`], so that a consumer of this struct can run
///   queries on the database.
#[derive(Debug, Constructor)]
pub struct Database {
    pub url: SqliteUrl,
    connection: DatabaseConnection,
}

impl Database {
    pub async fn open(url: SqliteUrl, key: SqlCipherKey) -> Result<Self, DbErr> {
        // Open database connection and set database key
        let mut connection_options = ConnectOptions::new(url.clone());
        connection_options.sqlx_logging_level(LevelFilter::Trace);
        connection_options.sqlcipher_key(format!("\"{}\"", String::from(key)));
        let connection = sea_orm::Database::connect(connection_options).await?;

        // Execute all migrations
        Migrator::up(&connection, None).await?;

        Ok(Self::new(url, connection))
    }

    pub async fn close_and_delete(self) -> Result<(), io::Error> {
        // Close the database connection and ignore any errors.
        let _ = self.connection.close().await;

        // Remove the database file if there is one. Note that this should be safe
        // even if closing the database failed, as the only failure possible is
        // that the connection is already closed.
        if let SqliteUrl::File(path) = self.url {
            return fs::remove_file(path).await;
        }

        Ok(())
    }

    pub fn connection(&self) -> &(impl ConnectionTrait + TransactionTrait) {
        &self.connection
    }
}

#[cfg(test)]
mod tests {
    use crypto::utils::random_bytes;

    use super::*;

    pub async fn down(db: &Database) -> Result<(), DbErr> {
        Migrator::down(&db.connection, None).await
    }

    #[test]
    fn test_sqlite_url() {
        assert_eq!(
            String::from(SqliteUrl::File(PathBuf::from("/foo/bar/database.db"))),
            "sqlite:///foo/bar/database.db?mode=rwc"
        );
        assert_eq!(String::from(SqliteUrl::InMemory), "sqlite::memory:");
    }

    #[tokio::test]
    async fn test_raw_sql_database() {
        use sea_orm::Statement;
        use sea_orm::Value;

        #[derive(Debug, PartialEq, Eq)]
        struct Person {
            id: i32,
            name: String,
            data: Option<Vec<u8>>,
        }

        // Create a new (encrypted) database.
        let key = SqlCipherKey::try_from(random_bytes(SqlCipherKey::size()).as_slice()).unwrap();
        let db = Database::open(SqliteUrl::InMemory, key)
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
                db.connection.get_database_backend(),
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
                db.connection.get_database_backend(),
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

        // Verify whether migrations can be undone
        down(&db).await.expect("Could not undo the migrations");

        // Finally, delete the test database.
        db.close_and_delete()
            .await
            .expect("Could not close and delete database");
    }

    #[tokio::test]
    async fn test_entities_database() {
        use sea_orm::prelude::*;
        use sea_orm::Set;
        use serde::Deserialize;
        use serde::Serialize;

        use entity::keyed_data;

        // Define example JSON data
        #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
        struct Configuration {
            id: u32,
            name: String,
        }
        let configuration = Configuration {
            id: 1234,
            name: "My wallet app".to_string(),
        };

        // Create a new (encrypted) database.
        let key = SqlCipherKey::try_from(random_bytes(SqlCipherKey::size_with_salt()).as_slice()).unwrap();
        let db = Database::open(SqliteUrl::InMemory, key)
            .await
            .expect("Could not open database");

        // Insert example data.
        let configuration_model = keyed_data::ActiveModel {
            key: Set("config".to_string()),
            data: Set(serde_json::to_value(&configuration).unwrap()),
        };
        configuration_model
            .insert(db.connection())
            .await
            .expect("Could not insert keyed data");

        // Fetch all keyed data and check if our example data is present.
        let all_keyed_data = keyed_data::Entity::find()
            .all(db.connection())
            .await
            .expect("Could not query keyed data");

        assert_eq!(all_keyed_data.len(), 1);

        let keyed_data = all_keyed_data.into_iter().last().unwrap();

        assert_eq!(keyed_data.key, "config");
        assert_eq!(
            serde_json::from_value::<Configuration>(keyed_data.data).unwrap(),
            configuration
        );

        // Verify whether migrations can be undone
        down(&db).await.expect("Could not undo the migrations");

        // Finally, delete the test database.
        db.close_and_delete()
            .await
            .expect("Could not close and delete database");
    }
}
