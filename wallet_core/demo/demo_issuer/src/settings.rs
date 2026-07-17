use std::collections::HashMap;
use std::net::IpAddr;
use std::path::Path;

use attestation_data::attributes::AttributeValue;
use attestation_data::attributes::Attributes;
use attestation_types::credential_kind::CredentialKind;
use config::Config;
use config::ConfigError;
use config::Environment;
use config::File;
use derive_more::Into;
use http_utils::server::TlsServerConfig;
use http_utils::urls::BaseUrl;
use http_utils::urls::DEFAULT_UNIVERSAL_LINK_BASE;
use openid4vc::issuable_document::IssuableDocument;
use openid4vc::issuer_identifier::IssuerIdentifier;
use openid4vc::metadata::issuer_metadata::CredentialConfigurationId;
use serde::Deserialize;
use serde_valid::Validate;
use utils::path::prefix_local_path;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_at_least::VecNonEmptyUnique;

#[derive(Deserialize, Clone)]
pub struct Settings {
    pub webserver: Server,
    pub issuance_server: Server,
    pub issuance_server_tls_config: Option<TlsServerConfig>,
    pub issuance_server_url: BaseUrl,
    pub pacf_issuance_server_url: BaseUrl,
    /// Base URL (Credential Issuer Identifier) of the acf_demo_issuer, used to build static
    /// authorization-code credential offers.
    pub acf_demo_issuer_url: IssuerIdentifier,
    pub universal_link_base_url: BaseUrl,
    pub help_base_url: BaseUrl,
    pub structured_logging: bool,
    pub log_requests: bool,
    pub usecases: HashMap<String, Usecase>,
}

#[derive(Deserialize, Clone)]
pub struct Server {
    pub ip: IpAddr,
    pub port: u16,
}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum Usecase {
    PreAuthorized {
        data: IssuableDocumentTemplates,
    },
    DisclosureBased {
        data: HashMap<AttributeValue, IssuableDocumentTemplates>,
        client_id: String,
        disclosed: Disclosed,
    },
    /// Authorization-code flow: a static credential offer pointing at the acf_demo_issuer. The offer
    /// carries no secret and no per-session code, so the same QR serves every wallet (each binds via
    /// its own PKCE during `/authorize`). The usecase is identified to the issuer by `issuer_state`,
    /// defaulting to this usecase's id when omitted.
    AuthorizationCode {
        credential_configuration_ids: VecNonEmptyUnique<CredentialConfigurationId>,
        #[serde(default)]
        issuer_state: Option<String>,
    },
}

pub type IssuableDocumentTemplates = VecNonEmpty<IssuableDocumentTemplate>;

#[derive(Deserialize, Clone, Validate, Into)]
pub struct IssuableDocumentTemplate {
    #[serde(flatten)]
    credential_format: CredentialKind,
    #[validate(custom = IssuableDocument::validate_attributes)]
    attributes: Attributes,
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
            .add_source(File::from(prefix_local_path(Path::new("demo_issuer.json")).as_ref()).required(false))
            .add_source(
                Environment::with_prefix("demo_issuer")
                    .separator("__")
                    .prefix_separator("__"),
            )
            .build()?
            .try_deserialize()
    }
}
