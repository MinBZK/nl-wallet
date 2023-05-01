pub mod data;
mod database;
mod database_storage;
mod key_file;
mod sql_cipher_key;

#[cfg(test)]
mod mock_storage;

use anyhow::Result;

use self::data::Registration;

pub use self::database_storage::DatabaseStorage;
#[cfg(test)]
pub use self::mock_storage::MockStorage;

#[derive(Debug, Clone, Copy)]
pub enum StorageState {
    Uninitialized,
    Unopened,
    Opened,
}

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Storage database is not opened")]
    NotOpened,
}

#[async_trait::async_trait]
pub trait Storage {
    async fn get_state(&self) -> Result<StorageState>;
    async fn open(&mut self) -> Result<()>;
    async fn clear(&mut self) -> Result<()>;
    async fn get_registration(&self) -> Result<Option<Registration>>;
    async fn save_registration(&mut self, registration: &Registration) -> Result<()>;
}
