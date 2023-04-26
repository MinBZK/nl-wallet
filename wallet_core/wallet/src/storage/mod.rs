pub mod data;
mod database;
mod database_storage;
mod key_file;
mod sql_cipher_key;
mod wallet_database;

#[cfg(test)]
mod mock_storage;

use anyhow::Result;
use data::Registration;

pub use database_storage::DatabaseStorage;
#[cfg(test)]
pub use mock_storage::MockStorage;

#[async_trait::async_trait]
trait Storage {
    async fn get_registration(&self) -> Result<Option<Registration>>;
    async fn save_registration(&mut self, registration: &Registration) -> Result<()>;
}
