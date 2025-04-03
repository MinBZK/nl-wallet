mod config_file;
mod data;
mod file_repository;
mod http_repository;
#[cfg(any(test, feature = "mock"))]
mod mock;
mod updating_repository;

use error_category::ErrorCategory;
use http_utils::http::TlsPinningConfig;
use jwt::error::JwtError;

use crate::repository::FileStorageError;
use crate::repository::HttpClientError;

pub use self::data::default_config_server_config;
pub use self::data::default_wallet_config;
pub use self::data::init_universal_link_base_url;
pub use self::data::UNIVERSAL_LINK_BASE_URL;
pub use self::file_repository::FileStorageConfigurationRepository;
pub use self::http_repository::HttpConfigurationRepository;
pub use self::updating_repository::UpdatingConfigurationRepository;

pub type WalletConfigurationRepository =
    UpdatingConfigurationRepository<FileStorageConfigurationRepository<HttpConfigurationRepository<TlsPinningConfig>>>;

#[cfg(any(test, feature = "mock"))]
pub use self::mock::LocalConfigurationRepository;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum ConfigurationError {
    #[error("could not store or load configuration or etag file: {0}")]
    FileStorage(#[from] FileStorageError),
    #[error("could not validate JWT: {0}")]
    Jwt(#[from] JwtError),
    #[error("http client error: {0}")]
    HttpClient(#[from] HttpClientError),
}
