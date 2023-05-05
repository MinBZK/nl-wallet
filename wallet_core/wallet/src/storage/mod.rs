pub mod data;
mod database;
mod database_storage;
mod key_file;
mod sql_cipher_key;

#[cfg(test)]
mod mock_storage;

use anyhow::Result;

use self::data::KeyedData;

pub use self::database_storage::DatabaseStorage;
#[cfg(test)]
pub use self::mock_storage::MockStorage;

/// This represents the current start of [`Storage`].
#[derive(Debug, Clone, Copy)]
pub enum StorageState {
    /// There is no database connection and no file on disk.
    Uninitialized,
    /// There is no database connection, but there is a file on disk.
    Unopened,
    /// There is an open database connection.
    Opened,
}

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Storage database is not opened")]
    NotOpened,
    #[error("Storage database is already opened")]
    AlreadyOpened,
}

/// This trait abstracts the persistent storage for the wallet.
#[async_trait::async_trait]
pub trait Storage {
    async fn state(&self) -> Result<StorageState>;

    async fn open(&mut self) -> Result<()>;
    async fn clear(&mut self) -> Result<()>;

    async fn fetch_data<D: KeyedData>(&self) -> Result<Option<D>>;
    async fn insert_data<D: KeyedData>(&mut self, data: &D) -> Result<()>;
}
