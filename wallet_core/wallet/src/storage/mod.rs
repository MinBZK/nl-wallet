pub mod data;
mod database;
mod database_storage;
mod key_file;
mod sql_cipher_key;

#[cfg(test)]
mod mock_storage;

use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};

use self::data::Keyed;

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
    #[error("Storage database is already opened")]
    AlreadyOpened,
}

#[async_trait::async_trait]
pub trait Storage {
    async fn state(&self) -> Result<StorageState>;

    async fn open(&mut self) -> Result<()>;
    async fn clear(&mut self) -> Result<()>;

    async fn fetch_data<D: Keyed + DeserializeOwned>(&self) -> Result<Option<D>>;
    async fn insert_data<D: Keyed + Serialize + Send + Sync>(&mut self, data: &D) -> Result<()>;
}
