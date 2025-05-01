use std::net::IpAddr;
use std::path::PathBuf;

use config::Config;
use config::ConfigError;
use config::Environment;
use config::File;
use indexmap::IndexMap;
use serde::Deserialize;
use serde::Serialize;

use openid4vc::issuable_document::IssuableDocuments;
use utils::path::prefix_local_path;

#[derive(Deserialize, Clone)]
pub struct Settings {
    pub webserver: Server,
    pub issuance_server: Server,
    pub structured_logging: bool,
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

#[derive(Debug, Deserialize, Clone)]
pub struct Usecase {
    #[serde(flatten)]
    pub data: IndexMap<String, IssuableDocuments>,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        Config::builder()
            .set_default("webserver.ip", "0.0.0.0")?
            .set_default("webserver.port", 3005)?
            .set_default("issuance_server.port", 3006)?
            .set_default("structured_logging", false)?
            .add_source(File::from(prefix_local_path("demo_issuer.toml".as_ref()).as_ref()).required(false))
            .add_source(
                Environment::with_prefix("demo_issuer")
                    .separator("__")
                    .prefix_separator("__"),
            )
            .build()?
            .try_deserialize()
    }
}
