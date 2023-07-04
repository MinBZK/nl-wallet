use std::{env, net::IpAddr, path::PathBuf};

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

use wallet_common::account::serialization::Base64Bytes;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub signing_private_key: Base64Bytes,
    pub database: Database,
    pub webserver: Webserver,
}

#[derive(Debug, Deserialize)]
pub struct Database {
    pub host: String,
    pub name: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Webserver {
    pub ip: IpAddr,
    pub port: u16,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        // Look for a config file that is in the same directory as Cargo.toml if run through cargo,
        // otherwise look in the current working directory.
        let config_path = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap_or_default();

        Config::builder()
            .set_default("database.host", "localhost")?
            .set_default("database.username", "postgres")?
            .set_default("database.password", "postgres")?
            .set_default("database.name", "wallet_provider")?
            .set_default("webserver.ip", "0.0.0.0")?
            .set_default("webserver.port", 3000)?
            .add_source(File::from(config_path.join("config")).required(false))
            .add_source(
                Environment::with_prefix("wallet_provider")
                    .separator("__")
                    .prefix_separator("_"),
            )
            .build()?
            .try_deserialize()
    }
}
