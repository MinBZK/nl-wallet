use std::{env, net::IpAddr, num::NonZeroU64, path::PathBuf, time::Duration};

use config::{Config, ConfigError, Environment, File};
use p256::{ecdsa::SigningKey, pkcs8::DecodePrivateKey};
use serde::Deserialize;
use serde_with::{base64::Base64, serde_as};
use url::Url;

use nl_wallet_mdoc::utils::x509::Certificate;
use openid4vc::server_state::SessionStoreTimeouts;
use wallet_common::{config::wallet_config::BaseUrl, sentry::Sentry};

cfg_if::cfg_if! {
    if #[cfg(feature = "disclosure")] {
        mod disclosure;
        pub use disclosure::*;
        use wallet_common::config::wallet_config::DEFAULT_UNIVERSAL_LINK_BASE;
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "issuance")] {
        mod issuance;
        pub use issuance::*;
    }
}

#[derive(Clone, Deserialize)]
pub struct Urls {
    // used by the wallet
    pub public_url: BaseUrl,

    #[cfg(feature = "disclosure")]
    pub universal_link_base_url: BaseUrl,
}

#[derive(Clone, Deserialize)]
pub struct Settings {
    // used by the wallet, MUST be reachable from the public internet.
    pub wallet_server: Server,
    // used by the application, SHOULD be reachable only by the application.
    // if not configured the wallet_server will be used, but an api_key is required in that case
    // if it conflicts with wallet_server, the application will crash on startup
    #[cfg(feature = "disclosure")]
    pub requester_server: RequesterAuth,

    #[serde(flatten)]
    pub urls: Urls,

    pub log_requests: bool,
    pub structured_logging: bool,

    pub storage: Storage,

    #[cfg(feature = "issuance")]
    pub issuer: Issuer,

    #[cfg(feature = "disclosure")]
    pub verifier: Verifier,

    pub sentry: Option<Sentry>,
}

#[derive(Clone, Deserialize)]
pub struct Server {
    pub ip: IpAddr,
    pub port: u16,
}

#[derive(Clone, Deserialize)]
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

#[derive(Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Authentication {
    ApiKey(String),
}

#[derive(Clone, Deserialize)]
pub struct Storage {
    /// Supported schemes are: `memory://` (default) and `postgres://`.
    pub url: Url,
    pub expiration_minutes: NonZeroU64,
    pub successful_deletion_minutes: NonZeroU64,
    pub failed_deletion_minutes: NonZeroU64,
}

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct KeyPair {
    #[serde_as(as = "Base64")]
    pub certificate: Vec<u8>,
    #[serde_as(as = "Base64")]
    pub private_key: Vec<u8>,
}

impl From<&Storage> for SessionStoreTimeouts {
    fn from(value: &Storage) -> Self {
        SessionStoreTimeouts {
            expiration: Duration::from_secs(60 * value.expiration_minutes.get()),
            successful_deletion: Duration::from_secs(60 * value.successful_deletion_minutes.get()),
            failed_deletion: Duration::from_secs(60 * value.failed_deletion_minutes.get()),
        }
    }
}

impl TryFrom<&KeyPair> for nl_wallet_mdoc::server_keys::KeyPair {
    type Error = p256::pkcs8::Error;

    fn try_from(value: &KeyPair) -> Result<Self, Self::Error> {
        let key_pair = Self::new(
            SigningKey::from_pkcs8_der(&value.private_key)?,
            Certificate::from(&value.certificate),
        );

        Ok(key_pair)
    }
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        Settings::new_custom("wallet_server.toml", "wallet_server")
    }

    pub fn new_custom(config_file: &str, env_prefix: &str) -> Result<Self, ConfigError> {
        let default_store_timeouts = SessionStoreTimeouts::default();

        let config_builder = Config::builder()
            .set_default("wallet_server.ip", "0.0.0.0")?
            .set_default("wallet_server.port", 3001)?
            .set_default("public_url", "http://localhost:3001/")?
            .set_default("log_requests", false)?
            .set_default("structured_logging", false)?
            .set_default("storage.url", "memory://")?
            .set_default(
                "storage.expiration_minutes",
                default_store_timeouts.expiration.as_secs() / 60,
            )?
            .set_default(
                "storage.successful_deletion_minutes",
                default_store_timeouts.successful_deletion.as_secs() / 60,
            )?
            .set_default(
                "storage.failed_deletion_minutes",
                default_store_timeouts.failed_deletion.as_secs() / 60,
            )?;

        #[cfg(feature = "disclosure")]
        let config_builder = config_builder.set_default("universal_link_base_url", DEFAULT_UNIVERSAL_LINK_BASE)?;

        #[cfg(feature = "issuance")]
        let config_builder = config_builder
            .set_default(
                "issuer.wallet_client_ids",
                vec![openid4vc::NL_WALLET_CLIENT_ID.to_string()],
            )?
            .set_default("issuer.brp_server", "http://localhost:3007/")?;

        // Look for a config file that is in the same directory as Cargo.toml if run through cargo,
        // otherwise look in the current working directory.
        let config_path = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap_or_default();
        let config_source = config_path.join(config_file);

        let environment_parser = Environment::with_prefix(env_prefix)
            .separator("__")
            .prefix_separator("_");

        #[cfg(feature = "disclosure")]
        let environment_parser = environment_parser
            .list_separator(",")
            .with_list_parse_key("verifier.trust_anchors");

        let environment_parser = environment_parser.try_parsing(true);

        config_builder
            .add_source(File::from(config_source).required(false))
            .add_source(File::from(PathBuf::from(config_file)).required(false))
            .add_source(environment_parser)
            .build()?
            .try_deserialize()
    }
}
