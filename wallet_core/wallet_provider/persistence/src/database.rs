use sea_orm::Database;
use sea_orm::DatabaseConnection;
use tracing::log::LevelFilter;
use wallet_provider_database_settings::ConnectionOptions;
use wallet_provider_database_settings::ConnectionString;

use wallet_provider_domain::repository::PersistenceError;

use crate::PersistenceConnection;

pub struct Db(DatabaseConnection);

impl Db {
    pub async fn new(
        connection_string: ConnectionString,
        connection_options: ConnectionOptions,
    ) -> Result<Db, PersistenceError> {
        let mut connect_options = sea_orm::ConnectOptions::new(connection_string);
        connect_options
            .connect_timeout(connection_options.connect_timeout)
            .max_connections(connection_options.max_connections.into())
            .sqlx_logging(true)
            .sqlx_logging_level(LevelFilter::Trace);

        let db = Database::connect(connect_options)
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
