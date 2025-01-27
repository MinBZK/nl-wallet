use std::env;
use std::net::IpAddr;
use std::path::PathBuf;

use config::Config;
use config::ConfigError;
use config::Environment;
use config::File;
use serde::Deserialize;

use wallet_common::config::http::TlsServerConfig;

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
        // Look for a config file that is in the same directory as Cargo.toml if run through cargo,
        // otherwise look in the current working directory.
        let config_path = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap_or_default();

        Config::builder()
            .set_default("ip", "0.0.0.0")?
            .set_default("port", 3009)?
            .set_default("structured_logging", false)?
            .add_source(File::from(config_path.join("update_policy_server.toml")).required(false))
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
