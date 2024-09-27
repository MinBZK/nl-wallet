use std::{env, net::IpAddr, path::PathBuf};

use config::{Config, ConfigError, Environment, File};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use nl_wallet_mdoc::verifier::ItemsRequests;
pub use wallet_common::urls::CorsOrigin;
use wallet_common::{sentry::Sentry, urls::BaseUrl};

#[derive(Deserialize, Clone)]
pub struct Settings {
    pub webserver: Server,
    pub internal_wallet_server_url: BaseUrl,
    pub public_wallet_server_url: BaseUrl,
    pub public_url: BaseUrl,
    pub structured_logging: bool,
    pub allow_origins: Option<CorsOrigin>,
    pub wallet_web: WalletWeb,
    pub usecases: IndexMap<String, Usecase>,
    pub sentry: Option<Sentry>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WalletWeb {
    // relative to /assets
    pub filename: PathBuf,
    pub sha256: String,
}

#[derive(Deserialize, Clone)]
pub struct Server {
    pub ip: IpAddr,
    pub port: u16,
}

#[derive(Deserialize, Default, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ReturnUrlMode {
    #[default]
    Url,
    None,
}

#[derive(Deserialize, Clone)]
pub struct Usecase {
    #[serde(default)]
    pub return_url: ReturnUrlMode,
    pub items_requests: ItemsRequests,
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
            .set_default("structured_logging", false)?
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
