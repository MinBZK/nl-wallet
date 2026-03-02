use std::net::IpAddr;
use std::path::Path;

use config::Config;
use config::ConfigError;
use config::Environment;
use config::File;
use serde::Deserialize;

use crypto::SymmetricKey;
use http_utils::client::HttpServiceConfig;
use utils::path::prefix_local_path;

#[derive(Deserialize, Clone)]
pub struct Settings {
    pub webserver: Server,
    pub cookie_encryption_key: SymmetricKey,
    pub revocation_endpoint: HttpServiceConfig,
    pub structured_logging: bool,
    pub log_requests: bool,
}

#[derive(Deserialize, Clone)]
pub struct Server {
    pub ip: IpAddr,
    pub port: u16,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        Config::builder()
            .set_default("webserver.ip", "0.0.0.0")?
            .set_default("webserver.port", 8001)?
            .set_default("structured_logging", false)?
            .set_default("log_requests", false)?
            .add_source(File::from(prefix_local_path(Path::new("revocation_portal.toml")).as_ref()).required(false))
            .add_source(
                Environment::with_prefix("revocation_portal")
                    .separator("__")
                    .prefix_separator("__"),
            )
            .build()?
            .try_deserialize()
    }
}
