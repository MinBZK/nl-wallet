use std::io;
use std::path::PathBuf;

use derive_more::Constructor;
use sea_orm::ConnectOptions;
use sea_orm::ConnectionTrait;
use sea_orm::DatabaseConnection;
use sea_orm::DbErr;
use sea_orm::RuntimeErr;
use sea_orm::SqlxSqliteConnector;
use sea_orm::TransactionTrait;
use sea_orm::sqlx::ConnectOptions as _;
use sea_orm::sqlx::sqlite::SqliteConnectOptions;
use sea_orm::sqlx::sqlite::SqlitePoolOptions;
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
            // sqlx ensures that `:memory:` works in a connection pool, it is
            // translated to a named in memory database with a shared cache.
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
        let connection = match &url {
            // File-backed (production) database: a standard Sea-ORM connection pool.
            SqliteUrl::File(_) => {
                let mut connection_options = ConnectOptions::new(&url);
                connection_options.sqlx_logging_level(LevelFilter::Trace);
                connection_options.sqlcipher_key(format!("\"{}\"", String::from(key)));

                sea_orm::Database::connect(connection_options).await?
            }
            // In-memory (test) database: `sqlite::memory:` resolves to a *shared-cache* in-memory
            // database, which SQLite destroys the instant no connection to it remains open. A
            // default pool is allowed both to drop to zero connections (`min_connections == 0`)
            // and to recycle connections on a timer (`idle_timeout` / `max_lifetime`), either of
            // which silently wipes the database and surfaces later as spurious "no such table" errors.
            // Pin the pool to a single connection that is never reaped, so the in-memory database lives
            // exactly as long as this `Database` instance.
            SqliteUrl::InMemory => {
                let connect_options = String::from(&url)
                    .parse::<SqliteConnectOptions>()
                    .map_err(|error| DbErr::Conn(RuntimeErr::SqlxError(error)))?
                    .pragma("key", format!("\"{}\"", String::from(key)))
                    .log_statements(LevelFilter::Trace);

                let pool = SqlitePoolOptions::new()
                    .min_connections(1)
                    .max_connections(1)
                    .idle_timeout(None)
                    .max_lifetime(None)
                    .connect_with(connect_options)
                    .await
                    .map_err(|error| DbErr::Conn(RuntimeErr::SqlxError(error)))?;

                SqlxSqliteConnector::from_sqlx_sqlite_pool(pool)
            }
        };

        // Execute all migrations
        Migrator::up(&connection, None).await?;

        Ok(Self::new(url, connection))
    }

    pub async fn close(self) -> Result<(), DbErr> {
        self.connection.close().await
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
    use super::*;

    pub async fn down(db: &Database) -> Result<(), DbErr> {
        Migrator::down(&db.connection, None).await
    }

    // Regression test for the in-memory connection-pool teardown race: a `sqlite::memory:`
    // database is a shared-cache in-memory database that SQLite destroys as soon as no
    // connection to it remains open. The migrated tables therefore survive only as long as the
    // pool keeps a connection open and never recycles it; otherwise the database is wiped and
    // queries fail with spurious "no such table" errors (e.g. `no such table: keyed_data`).
    //
    // We assert the pool configuration directly, rather than waiting out a (multi-minute) reaper
    // interval at runtime, so the test is deterministic and fails immediately if either safeguard
    // is weakened:
    //   * a `min_connections` floor of 1, so the pool always holds a connection (and, should a timeout ever be
    //     reintroduced, re-establishes one rather than leaving zero); and
    //   * no `idle_timeout`/`max_lifetime`, so the connection is never reaped on a timer.
    #[tokio::test]
    async fn test_in_memory_pool_is_pinned() {
        let key = SqlCipherKey::new_random_with_salt();
        let db = Database::open(SqliteUrl::InMemory, key)
            .await
            .expect("Could not open database");

        let pool = db.connection.get_sqlite_connection_pool();

        assert_eq!(
            pool.options().get_min_connections(),
            1,
            "in-memory pool must always keep a connection"
        );
        assert_eq!(
            pool.options().get_max_connections(),
            1,
            "in-memory pool is single-connection"
        );
        assert!(
            pool.options().get_idle_timeout().is_none(),
            "in-memory pool connection must not be idle-reaped"
        );
        assert!(
            pool.options().get_max_lifetime().is_none(),
            "in-memory pool connection must not be lifetime-recycled"
        );
        assert!(
            pool.size() >= 1,
            "in-memory pool should have eagerly opened its connection"
        );
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
        let key = SqlCipherKey::new_random_with_salt();
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
        use entity::keyed_data;
        use sea_orm::Set;
        use sea_orm::prelude::*;
        use serde::Deserialize;
        use serde::Serialize;

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
        let key = SqlCipherKey::new_random_with_salt();
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

        let keyed_data = all_keyed_data.into_iter().next_back().unwrap();

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
