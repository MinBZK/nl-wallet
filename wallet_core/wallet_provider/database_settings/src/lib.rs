use std::time::Duration;

use serde::Deserialize;
use serde_with::DurationSeconds;
use serde_with::serde_as;
use url::Url;

#[derive(Clone, Deserialize)]
pub struct Settings {
    pub url: Url,

    #[serde(default)]
    pub options: ConnectionOptions,
}

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct ConnectionOptions {
    #[serde(rename = "connect_timeout_in_sec")]
    #[serde_as(as = "DurationSeconds")]
    pub connect_timeout: Duration,

    pub max_connections: u32,
}

impl Default for ConnectionOptions {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(10),
            max_connections: 10,
        }
    }
}

#[cfg(feature = "test")]
impl Settings {
    pub fn new() -> Result<Self, config::ConfigError> {
        config::Config::builder()
            .add_source(
                config::File::from(
                    utils::path::prefix_local_path(std::path::Path::new("wallet_provider_database_settings.toml"))
                        .as_ref(),
                )
                .required(false),
            )
            .add_source(
                config::Environment::with_prefix("wallet_provider_database_settings")
                    .separator("__")
                    .prefix_separator("__"),
            )
            .build()?
            .try_deserialize()
    }
}
