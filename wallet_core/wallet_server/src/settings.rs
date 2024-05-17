use std::{collections::HashMap, env, net::IpAddr, num::NonZeroU64, path::PathBuf, time::Duration};

use config::{Config, ConfigError, Environment, File};
use nutype::nutype;
use p256::{ecdsa::SigningKey, pkcs8::DecodePrivateKey};
use serde::Deserialize;
use serde_with::{base64::Base64, hex::Hex, serde_as};
use url::Url;

use nl_wallet_mdoc::{
    server_state::SessionStoreTimeouts,
    utils::x509::Certificate,
    verifier::{SessionTypeReturnUrl, UseCase, UseCases},
};
use wallet_common::{
    config::wallet_config::{BaseUrl, DEFAULT_UNIVERSAL_LINK_BASE},
    trust_anchor::DerTrustAnchor,
};

#[cfg(feature = "issuance")]
use {indexmap::IndexMap, wallet_common::reqwest::deserialize_certificates};

const MIN_KEY_LENGTH_BYTES: usize = 16;

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
    pub log_requests: bool,

    pub storage: Storage,

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
pub struct Storage {
    /// Supported schemes are: `memory://` (default) and `postgres://`.
    pub url: Url,
    pub expiration_minutes: NonZeroU64,
    pub successful_deletion_minutes: NonZeroU64,
    pub failed_deletion_minutes: NonZeroU64,
}

#[nutype(validate(predicate = |v| v.len() >= MIN_KEY_LENGTH_BYTES), derive(Clone, TryFrom, Deserialize))]
pub struct EhpemeralIdSecret(Vec<u8>);

#[derive(Deserialize, Clone)]
pub struct Server {
    pub ip: IpAddr,
    pub port: u16,
}

#[cfg(feature = "issuance")]
#[derive(Deserialize, Clone)]
pub struct Digid {
    pub issuer_url: BaseUrl,
    pub bsn_privkey: String,
    #[serde(deserialize_with = "deserialize_certificates", default)]
    pub trust_anchors: Vec<reqwest::Certificate>,
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

    pub brp_server: BaseUrl,
}

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct KeyPair {
    #[serde_as(as = "Base64")]
    pub certificate: Vec<u8>,
    #[serde_as(as = "Base64")]
    pub private_key: Vec<u8>,
}

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct Verifier {
    pub usecases: VerifierUseCases,
    pub trust_anchors: Vec<DerTrustAnchor>,
    #[serde_as(as = "Hex")]
    pub ephemeral_id_secret: EhpemeralIdSecret,
}

#[nutype(derive(Clone, Deserialize))]
pub struct VerifierUseCases(HashMap<String, VerifierUseCase>);

#[derive(Clone, Deserialize)]
pub struct VerifierUseCase {
    #[serde(with = "SessionTypeReturnUrlDef", default)]
    pub session_type_return_url: SessionTypeReturnUrl,
    #[serde(flatten)]
    pub key_pair: KeyPair,
}

#[derive(Deserialize)]
#[serde(remote = "SessionTypeReturnUrl", rename_all = "snake_case")]
pub enum SessionTypeReturnUrlDef {
    Neither,
    SameDevice,
    Both,
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

#[cfg(feature = "issuance")]
impl Issuer {
    pub fn certificates(&self) -> IndexMap<String, Certificate> {
        self.private_keys
            .iter()
            .map(|(doctype, privkey)| (doctype.clone(), privkey.certificate.clone().into()))
            .collect()
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

impl TryFrom<VerifierUseCases> for UseCases {
    type Error = p256::pkcs8::Error;

    fn try_from(value: VerifierUseCases) -> Result<Self, Self::Error> {
        let use_cases = value
            .into_inner()
            .into_iter()
            .map(|(id, use_case)| {
                let use_case = UseCase::try_from(&use_case)?;

                Ok((id, use_case))
            })
            .collect::<Result<HashMap<_, _>, Self::Error>>()?
            .into();

        Ok(use_cases)
    }
}

impl TryFrom<&VerifierUseCase> for UseCase {
    type Error = p256::pkcs8::Error;

    fn try_from(value: &VerifierUseCase) -> Result<Self, Self::Error> {
        let use_case = UseCase {
            key_pair: (&value.key_pair).try_into()?,
            session_type_return_url: value.session_type_return_url,
        };

        Ok(use_case)
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
            .set_default("universal_link_base_url", DEFAULT_UNIVERSAL_LINK_BASE)?
            .set_default("log_requests", false)?
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

        #[cfg(feature = "issuance")]
        let config_builder = config_builder
            .set_default(
                "issuer.wallet_client_ids",
                vec![openid4vc::NL_WALLET_CLIENT_ID.to_string()],
            )?
            .set_default("issuer.brp_server", "http://localhost:3007/")?
            .set_default("issuer.digid.trust_anchors", vec![] as Vec<String>)?;

        // Look for a config file that is in the same directory as Cargo.toml if run through cargo,
        // otherwise look in the current working directory.
        let config_path = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap_or_default();
        let config_source = config_path.join(config_file);

        config_builder
            .add_source(File::from(config_source).required(false))
            .add_source(File::from(PathBuf::from(config_file)).required(false))
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
