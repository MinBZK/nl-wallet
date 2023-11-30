mod client;
mod data;
mod http;
mod local;

use std::sync::Arc;

use async_trait::async_trait;
use url::ParseError;

use wallet_common::config::wallet_config::WalletConfiguration;

pub use self::{
    data::{default_configuration, ConfigServerConfiguration},
    http::HttpConfigurationRepository,
    local::LocalConfigurationRepository,
};

#[derive(Debug, thiserror::Error)]
pub enum ConfigurationError {
    #[error("networking error: {0}")]
    Networking(#[from] reqwest::Error),
    #[error("could not get config from config server: {0} - Response body: {1}")]
    Response(#[source] reqwest::Error, String),
    #[error("could not parse base URL: {0}")]
    BaseUrl(#[from] ParseError),
}

#[async_trait]
pub trait ConfigurationRepository {
    fn config(&self) -> Arc<WalletConfiguration>;
    async fn fetch(&self) -> Result<(), ConfigurationError>;
}
