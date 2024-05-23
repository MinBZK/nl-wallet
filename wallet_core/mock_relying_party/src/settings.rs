use std::{collections::HashMap, env, net::IpAddr, path::PathBuf};

use axum::{headers::HeaderValue, http::header::InvalidHeaderValue};
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use url::Url;

use nl_wallet_mdoc::verifier::ItemsRequests;
use wallet_common::config::wallet_config::BaseUrl;

#[derive(Deserialize, Clone)]
pub struct Settings {
    pub webserver: Server,
    pub wallet_server_url: BaseUrl,
    pub public_url: BaseUrl,
    #[serde(default)]
    pub allow_origins: Vec<Origin>,
    pub usecases: HashMap<String, ItemsRequests>,
}

#[derive(Deserialize, Clone)]
pub struct Server {
    pub ip: IpAddr,
    pub port: u16,
}

#[derive(Deserialize, Clone)]
pub struct Origin(Url);

impl TryFrom<Origin> for HeaderValue {
    type Error = InvalidHeaderValue;

    fn try_from(value: Origin) -> Result<Self, Self::Error> {
        HeaderValue::try_from(value.0.as_str())
    }
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        // Look for a config file that is in the same directory as Cargo.toml if run through cargo,
        // otherwise look in the current working directory.
        let config_path = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap_or_default();

        Config::builder()
            .set_default("webserver.ip", "0.0.0.0")?
            .set_default("webserver.port", 3004)?
            .set_default("public_url", "http://localhost:3004/")?
            .add_source(File::from(config_path.join("mock_relying_party.toml")).required(false))
            .add_source(
                Environment::with_prefix("mock_relying_party")
                    .separator("__")
                    .prefix_separator("_"),
            )
            .build()?
            .try_deserialize()
    }
}
