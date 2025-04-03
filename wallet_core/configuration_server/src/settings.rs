use std::net::IpAddr;

use config::Config;
use config::ConfigError;
use config::Environment;
use config::File;
use serde::Deserialize;

use http_utils::http::TlsServerConfig;
use utils::utils::prefix_local_path;

#[derive(Clone, Deserialize)]
pub struct Settings {
    pub ip: IpAddr,
    pub port: u16,
    pub wallet_config_jwt: String,
    pub tls_config: TlsServerConfig,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        Config::builder()
            .set_default("ip", "0.0.0.0")?
            .set_default("port", 3005)?
            .add_source(File::from(prefix_local_path("config_server.toml".as_ref()).as_ref()).required(false))
            .add_source(
                Environment::with_prefix("config_server")
                    .separator("__")
                    .prefix_separator("__")
                    .list_separator(","),
            )
            .build()?
            .try_deserialize()
    }
}
