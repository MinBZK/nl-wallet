use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

use wallet_common::account::serialization::Base64Bytes;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub signing_private_key: Base64Bytes,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(File::with_name("wallet_provider/config.toml").required(false))
            .add_source(Environment::with_prefix("wallet_provider"))
            .build()?
            .try_deserialize()
    }
}
