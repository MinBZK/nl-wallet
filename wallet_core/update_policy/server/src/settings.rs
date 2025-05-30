use std::net::IpAddr;

use config::Config;
use config::ConfigError;
use config::Environment;
use config::File;
use serde::Deserialize;

use http_utils::tls::TlsServerConfig;
use utils::path::prefix_local_path;

use crate::config::UpdatePolicyConfig;

#[derive(Clone, Deserialize)]
pub struct Settings {
    pub ip: IpAddr,
    pub port: u16,
    pub tls_config: Option<TlsServerConfig>,
    pub structured_logging: bool,

    #[serde(default)]
    pub update_policy: UpdatePolicyConfig,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        Config::builder()
            .set_default("ip", "0.0.0.0")?
            .set_default("port", 8001)?
            .set_default("structured_logging", false)?
            .add_source(File::from(prefix_local_path("update_policy_server.toml".as_ref()).as_ref()).required(false))
            .add_source(
                Environment::with_prefix("update_policy_server")
                    .separator("__")
                    .prefix_separator("__")
                    .list_separator("|"),
            )
            .build()?
            .try_deserialize()
    }
}
