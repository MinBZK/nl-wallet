use std::net::IpAddr;
use std::path::Path;

use config::Config;
use config::ConfigError;
use config::Environment;
use config::File;
use serde::Deserialize;

use http_utils::tls::server::TlsServerConfig;
use jwt::VerifiedJwt;
use status_lists::publish::PublishDir;
use utils::path::prefix_local_path;
use wallet_configuration::wallet_config::WalletConfiguration;

#[derive(Clone, Deserialize)]
pub struct Settings {
    pub ip: IpAddr,
    pub port: u16,
    pub tls_config: TlsServerConfig,

    #[serde(deserialize_with = "VerifiedJwt::dangerous_deserialize")] // we trust our own config file
    pub wallet_config_jwt: VerifiedJwt<WalletConfiguration>,
    pub wua_publish_dir: PublishDir,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        Config::builder()
            .set_default("ip", "0.0.0.0")?
            .set_default("port", 8001)?
            .add_source(File::from(prefix_local_path(Path::new("static_server.toml")).as_ref()).required(false))
            .add_source(
                Environment::with_prefix("static_server")
                    .separator("__")
                    .prefix_separator("__")
                    .list_separator(","),
            )
            .build()?
            .try_deserialize()
    }
}
