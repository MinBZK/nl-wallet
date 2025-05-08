use std::net::IpAddr;
use std::path::PathBuf;

use config::Config;
use config::ConfigError;
use config::Environment;
use config::File;
use indexmap::IndexMap;
use serde::Deserialize;
use serde::Serialize;

use http_utils::urls::BaseUrl;
use http_utils::urls::CorsOrigin;
use mdoc::verifier::ItemsRequests;
use utils::path::prefix_local_path;

#[derive(Deserialize, Clone)]
pub struct Settings {
    pub webserver: Server,
    pub internal_wallet_server_url: BaseUrl,
    pub public_wallet_server_url: BaseUrl,
    pub public_url: BaseUrl,
    pub help_base_url: BaseUrl,
    pub structured_logging: bool,
    pub allow_origins: Option<CorsOrigin>,
    pub wallet_web: WalletWeb,
    pub usecases: IndexMap<String, Usecase>,
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
        Config::builder()
            .set_default("webserver.ip", "0.0.0.0")?
            .set_default("webserver.port", 3008)?
            .set_default("public_url", "http://localhost:3008/")?
            .set_default("structured_logging", false)?
            .add_source(File::from(prefix_local_path("demo_relying_party.toml".as_ref()).as_ref()).required(false))
            .add_source(
                Environment::with_prefix("demo_relying_party")
                    .separator("__")
                    .prefix_separator("__"),
            )
            .build()?
            .try_deserialize()
    }
}
