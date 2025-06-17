use std::net::IpAddr;

use config::Config;
use config::ConfigError;
use config::Environment;
use config::File;
use serde::Deserialize;

use http_utils::urls::BaseUrl;
use utils::path::prefix_local_path;

#[derive(Deserialize, Clone)]
pub struct Settings {
    pub webserver: Server,
    pub structured_logging: bool,
    pub log_requests: bool,
    pub demo_services: Vec<DemoService>,
}

#[derive(Deserialize, Clone)]
pub struct Server {
    pub ip: IpAddr,
    pub port: u16,
}

#[derive(Deserialize, Clone)]
pub struct DemoService {
    pub service_url: BaseUrl,
    pub usecases: Vec<String>,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        Config::builder()
            .set_default("webserver.ip", "0.0.0.0")?
            .set_default("webserver.port", 8001)?
            .set_default("structured_logging", false)?
            .set_default("log_requests", false)?
            .add_source(File::from(prefix_local_path("demo_index.toml".as_ref()).as_ref()).required(false))
            .add_source(
                Environment::with_prefix("demo_index")
                    .separator("__")
                    .prefix_separator("__"),
            )
            .build()?
            .try_deserialize()
    }
}
