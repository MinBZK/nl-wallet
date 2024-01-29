use serde_with::{base64::Base64, serde_as};
use std::{collections::HashMap, env, net::IpAddr, path::PathBuf};

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use url::Url;

#[derive(Deserialize, Clone)]
pub struct Settings {
    // used by the wallet, MUST be reachable from the public internet.
    pub wallet_server: Server,
    // used by the application, SHOULD be reachable only by the application.
    // if it conflicts with wallet_server, the application will crash on startup
    pub requester_server: Server,
    pub usecases: HashMap<String, KeyPair>,
    pub trust_anchors: Vec<String>,
    pub public_url: Url,
    // used by the application
    pub internal_url: Url,
    // supported schemes are: memory:// (default) and postgres://
    pub store_url: Url,
}

#[derive(Deserialize, Clone)]
pub struct Server {
    pub ip: IpAddr,
    pub port: u16,
}

#[serde_as]
#[derive(Deserialize, Clone)]
pub struct KeyPair {
    #[serde_as(as = "Base64")]
    pub certificate: Vec<u8>,
    #[serde_as(as = "Base64")]
    pub private_key: Vec<u8>,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        // Look for a config file that is in the same directory as Cargo.toml if run through cargo,
        // otherwise look in the current working directory.
        let config_path = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap_or_default();

        Config::builder()
            .set_default("wallet_server.ip", "0.0.0.0")?
            .set_default("wallet_server.port", 3001)?
            .set_default("requester_server.ip", "127.0.0.1")?
            .set_default("requester_server.port", 3002)?
            .set_default("public_url", "http://localhost:3001/")?
            .set_default("internal_url", "http://localhost:3002/")?
            .set_default("store_url", "memory://")?
            .add_source(File::from(config_path.join("wallet_server.toml")).required(false))
            .add_source(
                Environment::with_prefix("wallet_server")
                    .separator("__")
                    .prefix_separator("_")
                    .list_separator(",")
                    .with_list_parse_key("trust_anchors")
                    .try_parsing(true),
            )
            .build()?
            .try_deserialize()
    }
}
