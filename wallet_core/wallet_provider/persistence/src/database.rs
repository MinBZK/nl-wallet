use sea_orm::{Database, DatabaseConnection};

use wallet_provider_domain::repository::PersistenceError;

use crate::postgres::connection_string;

pub trait PersistenceConnection<T> {
    fn connection(&self) -> &T;
}

pub struct Db(DatabaseConnection);

impl Db {
    pub async fn new(
        host: &str,
        db_name: &str,
        username: Option<&str>,
        password: Option<&str>,
    ) -> Result<Db, PersistenceError> {
        let db = Database::connect(connection_string(host, db_name, username, password))
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
