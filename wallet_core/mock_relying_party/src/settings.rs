use std::{collections::HashMap, env, net::IpAddr, path::PathBuf};

use config::{Config, ConfigError, Environment, File};
use nl_wallet_mdoc::verifier::ItemsRequests;
use serde::Deserialize;
use wallet_common::config::wallet_config::BaseUrl;

#[derive(Deserialize, Clone)]
pub struct Settings {
    pub webserver: Server,
    pub wallet_server_url: BaseUrl,
    pub public_url: BaseUrl,
    pub usecases: HashMap<String, ItemsRequests>,
}

#[derive(Deserialize, Clone)]
pub struct Server {
    pub ip: IpAddr,
    pub port: u16,
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
