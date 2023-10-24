use std::time::Duration;

use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tracing::log::LevelFilter;
use wallet_provider_database_settings::ConnectionString;

use wallet_provider_domain::repository::PersistenceError;

use crate::PersistenceConnection;

const DB_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct Db(DatabaseConnection);

impl Db {
    pub async fn new(connection_string: ConnectionString) -> Result<Db, PersistenceError> {
        let mut connection_options = ConnectOptions::new(connection_string);
        connection_options
            .connect_timeout(DB_CONNECT_TIMEOUT)
            .sqlx_logging(true)
            .sqlx_logging_level(LevelFilter::Trace);

        let db = Database::connect(connection_options)
            .await
            .map_err(|e| PersistenceError::Connection(e.into()))?;

        Ok(Db(db))
    }
}

impl PersistenceConnection<DatabaseConnection> for Db {
    fn connection(&self) -> &DatabaseConnection {
        &self.0
    }
}
