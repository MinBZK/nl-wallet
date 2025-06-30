use std::net::IpAddr;

use config::Config;
use config::ConfigError;
use config::Environment;
use config::File;
use indexmap::IndexMap;
use serde::Deserialize;

use attestation_data::attributes::AttributeValue;
use attestation_data::issuable_document::IssuableDocuments;
use http_utils::tls::server::TlsServerConfig;
use http_utils::urls::BaseUrl;
use http_utils::urls::DEFAULT_UNIVERSAL_LINK_BASE;
use utils::path::prefix_local_path;
use utils::vec_at_least::VecNonEmpty;

#[derive(Deserialize, Clone)]
pub struct Settings {
    pub webserver: Server,
    pub issuance_server: Server,
    pub issuance_server_tls_config: Option<TlsServerConfig>,
    pub issuance_server_url: BaseUrl,
    pub universal_link_base_url: BaseUrl,
    pub help_base_url: BaseUrl,
    pub structured_logging: bool,
    pub log_requests: bool,
    pub usecases: IndexMap<String, Usecase>,
}

#[derive(Deserialize, Clone)]
pub struct Server {
    pub ip: IpAddr,
    pub port: u16,
}

#[derive(Deserialize, Clone)]
pub struct Usecase {
    #[serde(flatten)]
    pub data: IndexMap<AttributeValue, IssuableDocuments>,
    pub client_id: String,
    pub disclosed: Disclosed,
}

#[derive(Deserialize, Clone)]
pub struct Disclosed {
    pub credential_type: String,
    pub path: VecNonEmpty<String>,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        Config::builder()
            .set_default("webserver.ip", "0.0.0.0")?
            .set_default("webserver.port", 8001)?
            .set_default("issuance_server.ip", "127.0.0.1")?
            .set_default("issuance_server.port", 8002)?
            .set_default("universal_link_base_url", DEFAULT_UNIVERSAL_LINK_BASE)?
            .set_default("structured_logging", false)?
            .set_default("log_requests", false)?
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
