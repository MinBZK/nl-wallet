use std::time::Duration;

use derive_more::AsRef;
use sea_orm::Database;
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use serde_with::DurationSeconds;
use serde_with::serde_as;
use tracing::log::LevelFilter;
use url::Url;

use wallet_provider_domain::repository::PersistenceError;

use crate::PersistenceConnection;

#[derive(AsRef)]
pub struct Db(DatabaseConnection);

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct ConnectionOptions {
    #[serde(rename = "connect_timeout_in_sec")]
    #[serde_as(as = "DurationSeconds")]
    pub connect_timeout: Duration,

    pub max_connections: u32,
}

impl Db {
    pub async fn new_connection(
        database_url: Url,
        connection_options: ConnectionOptions,
    ) -> Result<DatabaseConnection, PersistenceError> {
        let mut connect_options = sea_orm::ConnectOptions::new(database_url);
        connect_options
            .connect_timeout(connection_options.connect_timeout)
            .max_connections(connection_options.max_connections)
            .sqlx_logging(true)
            .sqlx_logging_level(LevelFilter::Trace);

        let db = Database::connect(connect_options)
            .await
            .map_err(PersistenceError::Connection)?;

        Ok(db)
    }

    pub async fn new(database_url: Url, connection_options: ConnectionOptions) -> Result<Db, PersistenceError> {
        Db::new_connection(database_url, connection_options).await.map(Db)
    }

    pub fn to_connection(&self) -> DatabaseConnection {
        self.0.clone()
    }
}

impl PersistenceConnection<DatabaseConnection> for Db {
    fn connection(&self) -> &DatabaseConnection {
        &self.0
    }
}
