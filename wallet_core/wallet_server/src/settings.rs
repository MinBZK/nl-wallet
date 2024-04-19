use std::{collections::HashMap, env, net::IpAddr, path::PathBuf};

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use serde_with::{base64::Base64, serde_as};
use url::Url;

use wallet_common::{
    config::wallet_config::{BaseUrl, DEFAULT_UNIVERSAL_LINK_BASE},
    trust_anchor::DerTrustAnchor,
};

#[cfg(feature = "issuance")]
use {indexmap::IndexMap, nl_wallet_mdoc::utils::x509::Certificate, wallet_common::reqwest::deserialize_certificates};

#[cfg(feature = "mock")]
use crate::pid::mock::{PersonAttributes, ResidentAttributes};

#[derive(Deserialize, Clone)]
pub struct Settings {
    // used by the wallet, MUST be reachable from the public internet.
    pub wallet_server: Server,
    // used by the application, SHOULD be reachable only by the application.
    // if not configured the wallet_server will be used, but an api_key is required in that case
    // if it conflicts with wallet_server, the application will crash on startup
    pub requester_server: RequesterAuth,
    // used by the wallet
    pub public_url: BaseUrl,
    // used by the application
    pub internal_url: BaseUrl,
    pub universal_link_base_url: BaseUrl,
    // supported schemes are: memory:// (default) and postgres://
    pub store_url: Url,

    #[cfg(feature = "issuance")]
    pub issuer: Issuer,

    pub verifier: Verifier,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Authentication {
    ApiKey(String),
}

#[derive(Deserialize, Clone)]
pub enum RequesterAuth {
    #[serde(rename = "authentication")]
    Authentication(Authentication),
    #[serde(untagged)]
    ProtectedInternalEndpoint {
        authentication: Authentication,
        #[serde(flatten)]
        server: Server,
    },
    #[serde(untagged)]
    InternalEndpoint(Server),
}

#[derive(Deserialize, Clone)]
pub struct Verifier {
    pub usecases: HashMap<String, KeyPair>,
    pub trust_anchors: Vec<DerTrustAnchor>,
}

#[derive(Deserialize, Clone)]
pub struct Server {
    pub ip: IpAddr,
    pub port: u16,
}

#[serde_as]
#[derive(Deserialize, Clone)]
pub struct KeyPair {
    #[serde_as(as = "Base64")]
    pub certificate: Vec<u8>,
    #[serde_as(as = "Base64")]
    pub private_key: Vec<u8>,
}

#[cfg(feature = "issuance")]
#[derive(Deserialize, Clone)]
pub struct Digid {
    pub issuer_url: BaseUrl,
    pub bsn_privkey: String,
    #[serde(deserialize_with = "deserialize_certificates", default)]
    pub trust_anchors: Vec<reqwest::Certificate>,
}

#[cfg(feature = "mock")]
#[derive(Deserialize, Clone)]
pub struct MockAttributes {
    pub person: PersonAttributes,
    pub resident: Option<ResidentAttributes>,
}

#[cfg(feature = "issuance")]
#[derive(Deserialize, Clone)]
pub struct Issuer {
    // Issuer private keys index per doctype
    pub private_keys: HashMap<String, KeyPair>,

    /// `client_id` values that this server accepts, identifying the wallet implementation (not individual instances,
    /// i.e., the `client_id` value of a wallet implementation will be constant across all wallets of that
    /// implementation).
    /// The wallet sends this value in the authorization request and as the `iss` claim of its Proof of Possession JWTs.
    pub wallet_client_ids: Vec<String>,

    pub digid: Digid,

    #[cfg(feature = "mock")]
    pub mock_data: Option<Vec<MockAttributes>>,

    pub brp_server: BaseUrl,
}

#[cfg(feature = "issuance")]
impl Issuer {
    pub fn certificates(&self) -> IndexMap<String, Certificate> {
        self.private_keys
            .iter()
            .map(|(doctype, privkey)| (doctype.clone(), privkey.certificate.clone().into()))
            .collect()
    }
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        Settings::new_custom("wallet_server.toml", "wallet_server")
    }

    pub fn new_custom(config_file: &str, env_prefix: &str) -> Result<Self, ConfigError> {
        let config_builder = Config::builder()
            .set_default("wallet_server.ip", "0.0.0.0")?
            .set_default("wallet_server.port", 3001)?
            .set_default("public_url", "http://localhost:3001/")?
            .set_default("universal_link_base_url", DEFAULT_UNIVERSAL_LINK_BASE)?
            .set_default("store_url", "memory://")?
            .set_default("issuer.brp_server", "http://localhost:5001/")?
            .set_default("issuer.trust_anchors", vec![] as Vec<String>)?;

        #[cfg(feature = "issuance")]
        let config_builder = config_builder.set_default(
            "issuer.wallet_client_ids",
            vec![openid4vc::NL_WALLET_CLIENT_ID.to_string()],
        )?;

        // Look for a config file that is in the same directory as Cargo.toml if run through cargo,
        // otherwise look in the current working directory.
        let config_path = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap_or_default();
        let config_source = config_path.join(config_file);

        // If `config_source` exists use that, otherwise try `config_file` as absolute path.
        let config_source = if config_source.exists() {
            config_source
        } else {
            PathBuf::from(config_file)
        };

        config_builder
            .add_source(File::from(config_source).required(false))
            .add_source(
                Environment::with_prefix(env_prefix)
                    .separator("__")
                    .prefix_separator("_")
                    .list_separator(",")
                    .with_list_parse_key("verifier.trust_anchors")
                    .try_parsing(true),
            )
            .build()?
            .try_deserialize()
    }
}
