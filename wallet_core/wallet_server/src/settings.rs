use std::{collections::HashMap, env, net::IpAddr, path::PathBuf};

use config::{Config, ConfigError, Environment, File};
use openid4vc::NL_WALLET_CLIENT_ID;
use serde::Deserialize;
use url::Url;

use wallet_common::account::serialization::Base64Bytes;

#[derive(Deserialize, Clone)]
pub struct Settings {
    // used by the wallet, MUST be reachable from the public internet.
    pub wallet_server: Server,
    // used by the application, SHOULD be reachable only by the application.
    // if it conflicts with wallet_server, the application will crash on startup
    pub requester_server: Server,
    pub usecases: HashMap<String, KeyPair>,
    pub trust_anchors: Vec<String>,
    pub public_url: Url,
    // used by the application
    pub internal_url: Url,
    // supported schemes are: memory:// (default) and postgres://
    pub store_url: Url,
    pub issuer: Issuer,
}

#[derive(Deserialize, Clone)]
pub struct Server {
    pub ip: IpAddr,
    pub port: u16,
}

#[derive(Deserialize, Clone)]
pub struct KeyPair {
    pub certificate: Base64Bytes,
    pub private_key: Base64Bytes,
}

#[derive(Deserialize, Clone)]
pub struct Digid {
    pub issuer_url: Url,
    pub bsn_privkey: String,
    pub client_id: String,
}

#[derive(Deserialize, Clone)]
pub struct Issuer {
    // Issuer private keys index per doctype
    pub private_keys: HashMap<String, KeyPair>,

    /// URL identifying the issuer. The wallet must put this in the `aud` claim of its Proof of Possession JWTs.
    /// See https://openid.github.io/OpenID4VCI/openid-4-verifiable-credential-issuance-wg-draft.html#section-10.2.1
    pub credential_issuer_identifier: Url,

    /// `client_id` values that this server accepts, identifying the wallet implementation (not individual instances,
    /// i.e., the `client_id` value of a wallet implementation will be constant across all wallets of that
    /// implementation).
    /// The wallet sends this value in the authorization request and as the `iss` claim of its Proof of Possession JWTs.
    pub wallet_client_ids: Vec<String>,

    pub digid: Digid,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        // Look for a config file that is in the same directory as Cargo.toml if run through cargo,
        // otherwise look in the current working directory.
        let config_path = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap_or_default();

        Config::builder()
            .set_default("wallet_server.ip", "0.0.0.0")?
            .set_default("wallet_server.port", 3001)?
            .set_default("requester_server.ip", "127.0.0.1")?
            .set_default("requester_server.port", 3002)?
            .set_default("public_url", "http://localhost:3001/")?
            .set_default("internal_url", "http://localhost:3002/")?
            .set_default("store_url", "memory://")?
            .set_default("wallet_client_ids", vec![NL_WALLET_CLIENT_ID.to_string()])?
            .add_source(File::from(config_path.join("wallet_server.toml")).required(false))
            .add_source(
                Environment::with_prefix("wallet_server")
                    .separator("__")
                    .prefix_separator("_")
                    .list_separator(",")
                    .with_list_parse_key("trust_anchors")
                    .try_parsing(true),
            )
            .build()?
            .try_deserialize()
    }
}
