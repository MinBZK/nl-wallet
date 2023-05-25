use std::{env, path::PathBuf};

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

use wallet_common::account::serialization::Base64Bytes;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub signing_private_key: Base64Bytes,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        // Look for a config file that is in the same directory as Cargo.toml if run through cargo,
        // otherwise look in the current working directory.
        let config_path = env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .unwrap_or_default()
            .join("config");

        Config::builder()
            .add_source(File::from(config_path).required(false))
            .add_source(Environment::with_prefix("wallet_provider"))
            .build()?
            .try_deserialize()
    }
}
