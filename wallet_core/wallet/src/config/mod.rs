mod config_file;
mod data;
mod file_repository;
mod http_client;
mod http_repository;
#[cfg(any(test, feature = "mock"))]
mod mock;
mod updating_repository;

use std::sync::Arc;

use url::ParseError;

use error_category::ErrorCategory;
use wallet_common::config::http::TlsPinningConfig;
use wallet_common::config::wallet_config::WalletConfiguration;
use wallet_common::jwt::JwtError;

pub use self::data::default_config_server_config;
pub use self::data::default_wallet_config;
pub use self::data::init_universal_link_base_url;
pub use self::data::UNIVERSAL_LINK_BASE_URL;
pub use self::file_repository::FileStorageConfigurationRepository;
pub use self::http_repository::HttpConfigurationRepository;
pub use self::updating_repository::UpdatingConfigurationRepository;

pub type UpdatingFileHttpConfigurationRepository =
    UpdatingConfigurationRepository<FileStorageConfigurationRepository<HttpConfigurationRepository<TlsPinningConfig>>>;

#[cfg(any(test, feature = "mock"))]
pub use self::mock::LocalConfigurationRepository;

pub type ConfigCallback = Box<dyn FnMut(Arc<WalletConfiguration>) + Send + Sync>;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum ConfigurationError {
    #[error("networking error: {0}")]
    #[category(critical)]
    Networking(#[from] reqwest::Error),
    #[error("could not get config from config server: {0} - Response body: {1}")]
    #[category(critical)]
    Response(#[source] reqwest::Error, String),
    #[error("could not parse base URL: {0}")]
    #[category(critical)]
    BaseUrl(#[from] ParseError),
    #[error("could not store or load configuration: {0}")]
    ConfigFile(#[from] FileStorageError),
    #[error("could not validate JWT: {0}")]
    Jwt(#[from] JwtError),
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(pd)]
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

#[trait_variant::make(Send)]
pub trait UpdateableConfigurationRepository: ConfigurationRepository {
    async fn fetch(&self) -> Result<ConfigurationUpdateState, ConfigurationError>;
}

pub trait ObservableConfigurationRepository: ConfigurationRepository {
    fn register_callback_on_update(&self, callback: ConfigCallback) -> Option<ConfigCallback>;
    fn clear_callback(&self) -> Option<ConfigCallback>;
}
