use std::{env, net::IpAddr, path::PathBuf};

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use url::Url;

#[cfg(feature = "mock_attributes")]
use crate::mock_attributes::{PersonAttributes, ResidentAttributes};

#[derive(Clone, Deserialize)]
pub struct Settings {
    pub webserver: Webserver,
    pub digid: Digid,
    pub issuer_key: IssuerKey,
    pub public_url: Url,
    #[cfg(feature = "mock_attributes")]
    pub mock_data: Option<Vec<MockAttributes>>,
}

#[derive(Clone, Deserialize)]
pub struct Digid {
    pub issuer_url: Url,
    pub bsn_privkey: String,
    pub client_id: String,
}

#[derive(Clone, Deserialize)]
pub struct Webserver {
    pub ip: IpAddr,
    pub port: u16,
}

#[derive(Clone, Deserialize)]
pub struct IssuerKey {
    pub private_key: String,
    pub certificate: String,
}

#[cfg(feature = "mock_attributes")]
#[derive(Deserialize, Clone)]
pub struct MockAttributes {
    pub person: PersonAttributes,
    pub resident: Option<ResidentAttributes>,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        // Look for a config file that is in the same directory as Cargo.toml if run through cargo,
        // otherwise look in the current working directory.
        let config_path = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap_or_default();

        // TODO: use separate client ID for mock PID issuer.
        Config::builder()
            .set_default("webserver.ip", "0.0.0.0")?
            .set_default("webserver.port", 3003)?
            .set_default("public_url", "http://localhost:3003/")?
            .set_default("digid.issuer_url", "https://localhost:8006/")?
            .set_default("digid.client_id", "37692967-0a74-4e91-85ec-a4250e7ad5e8")?
            .add_source(File::from(config_path.join("pid_issuer.toml")).required(false))
            .add_source(
                Environment::with_prefix("pid_issuer")
                    .separator("__")
                    .prefix_separator("__"),
            )
            .build()?
            .try_deserialize()
    }
}
