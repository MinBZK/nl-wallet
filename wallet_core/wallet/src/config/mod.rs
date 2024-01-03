mod config_file;
mod data;
mod file_repository;
mod http_client;
mod http_repository;
#[cfg(any(test, feature = "mock"))]
mod mock;
mod updating_repository;

use std::sync::Arc;

use async_trait::async_trait;
use url::ParseError;

use wallet_common::config::wallet_config::WalletConfiguration;

pub use self::{
    data::{default_configuration, ConfigServerConfiguration},
    file_repository::FileStorageConfigurationRepository,
    http_repository::HttpConfigurationRepository,
    updating_repository::UpdatingConfigurationRepository,
};

pub type UpdatingFileHttpConfigurationRepository =
    UpdatingConfigurationRepository<FileStorageConfigurationRepository<HttpConfigurationRepository>>;

#[cfg(any(test, feature = "mock"))]
pub use self::mock::LocalConfigurationRepository;

#[derive(Debug, thiserror::Error)]
pub enum ConfigurationError {
    #[error("networking error: {0}")]
    Networking(#[from] reqwest::Error),
    #[error("could not get config from config server: {0} - Response body: {1}")]
    Response(#[source] reqwest::Error, String),
    #[error("could not parse base URL: {0}")]
    BaseUrl(#[from] ParseError),
    #[error("could not store or load configuration: {0}")]
    ConfigFile(#[from] FileStorageError),
}

#[derive(Debug, thiserror::Error)]
pub enum FileStorageError {
    #[error("config file I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug)]
pub enum ConfigurationUpdateState {
    Updated,
    Unmodified,
}

pub trait ConfigurationRepository {
    fn config(&self) -> Arc<WalletConfiguration>;
}

#[async_trait]
pub trait UpdateableConfigurationRepository: ConfigurationRepository {
    async fn fetch(&self) -> Result<ConfigurationUpdateState, ConfigurationError>;
}

#[async_trait]
pub trait ObservableConfigurationRepository: ConfigurationRepository {
    fn register_callback_on_update<F>(&self, callback: F)
    where
        F: Fn(Arc<WalletConfiguration>) + Send + Sync + 'static;

    fn clear_callback(&self);
}
