use std::{collections::HashMap, env, net::IpAddr, path::PathBuf};

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use url::Url;

#[derive(Deserialize, Clone)]
pub struct Settings {
    // used by the wallet
    pub wallet_server: Server,
    // used by the application, if not configured the wallet_server will be used
    // if it conflicts with wallet_server, the application will crash on startup
    pub requester_server: Option<Server>,
    pub usecases: HashMap<String, KeyPair>,
    pub trust_anchors: Vec<String>,
    pub public_url: Url,
    // used by the application, if not configured the public_url will be used
    pub internal_url: Option<Url>,
    // supported schemes are: memory:// (default) and postgres://
    pub store_url: Url,
}

#[derive(Deserialize, Clone)]
pub struct Server {
    pub ip: IpAddr,
    pub port: u16,
}

#[derive(Deserialize, Clone)]
pub struct KeyPair {
    pub certificate: String,
    pub private_key: String,
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
                    .prefix_separator("_"),
            )
            .build()?
            .try_deserialize()
    }
}
