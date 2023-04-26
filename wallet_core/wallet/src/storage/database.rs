use std::path::PathBuf;

use anyhow::Result;
use sea_orm::{ConnectOptions, ConnectionTrait, DatabaseConnection};
use tokio::fs;
use wallet_migration::{Migrator, MigratorTrait};

use super::sql_cipher_key::SqlCipherKey;

const PRAGMA_KEY: &str = "key";

pub enum SqliteUrl {
    File(PathBuf),
    InMemory,
}

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

pub struct Database {
    pub url: SqliteUrl,
    connection: DatabaseConnection,
}

impl Database {
    fn new(url: SqliteUrl, connection: DatabaseConnection) -> Self {
        Database { url, connection }
    }

    pub async fn open(url: SqliteUrl, key: SqlCipherKey) -> Result<Self> {
        // Open database connection and set database key
        let connection_options = ConnectOptions::new((&url).into());
        let connection = sea_orm::Database::connect(connection_options).await?;

        // Set database password using PRAGMA statement
        connection
            .execute_unprepared(&format!(r#"PRAGMA {} = "{}";"#, PRAGMA_KEY, String::from(key)))
            .await?;

        // Execute all migrations
        Migrator::up(&connection, None).await?;

        Ok(Self::new(url, connection))
    }

    pub async fn close_and_delete(self) -> Result<()> {
        // Close the database connection
        self.connection.close().await?;

        // Remove the database file (if there is one) and ignore any errors.
        if let SqliteUrl::File(path) = self.url {
            _ = fs::remove_file(path).await;
        }

        Ok(())
    }

    pub fn get_connection(&self) -> &impl ConnectionTrait {
        &self.connection
    }
}

#[cfg(test)]
mod tests {
    use wallet_common::utils::random_bytes;

    use super::*;

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
        use sea_orm::{Statement, Value};

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

        // Finally, delete the test database.
        db.close_and_delete()
            .await
            .expect("Could not close and delete database");
    }

    #[tokio::test]
    async fn test_entities_database() {
        use sea_orm::{prelude::*, Set};
        use serde::{Deserialize, Serialize};
        use wallet_entity::keyed_data;

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
            .insert(db.get_connection())
            .await
            .expect("Could not insert keyed data");

        // Fetch all keyed data and check if our example data is present.
        let all_keyed_data = keyed_data::Entity::find()
            .all(db.get_connection())
            .await
            .expect("Could not query keyed data");

        assert_eq!(all_keyed_data.len(), 1);

        let keyed_data = all_keyed_data.into_iter().last().unwrap();

        assert_eq!(keyed_data.key, "config");
        assert_eq!(
            serde_json::from_value::<Configuration>(keyed_data.data).unwrap(),
            configuration
        );

        // Finally, delete the test database.
        db.close_and_delete()
            .await
            .expect("Could not close and delete database");
    }
}
