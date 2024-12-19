use std::env;
use std::net::IpAddr;
use std::path::Path;
use std::path::PathBuf;

use config::Config;
use config::ConfigError;
use config::Environment;
use config::File;
use crypto_common::Key;
use crypto_common::KeySizeUser;
use serde::de;
use serde::Deserialize;
use serde_with::base64::Base64;
use serde_with::serde_as;

use wallet_common::reqwest::ReqwestTrustAnchor;
use wallet_common::urls::BaseUrl;

use crate::gba::client::FileGbavClient;
use crate::gba::client::HttpGbavClient;
use crate::gba::{self};

#[derive(Deserialize)]
pub struct Settings {
    pub ip: IpAddr,
    pub port: u16,

    #[serde(default)]
    pub structured_logging: bool,

    pub run_mode: RunMode,
}

#[serde_as]
#[derive(Deserialize)]
pub struct GbavSettings {
    pub adhoc_url: BaseUrl,
    pub username: String,
    pub password: String,

    #[serde_as(as = "Base64")]
    pub client_cert: Vec<u8>,

    #[serde_as(as = "Base64")]
    pub client_cert_key: Vec<u8>,

    #[serde_as(as = "Base64")]
    pub trust_anchor: ReqwestTrustAnchor,

    pub ca_api_key: Option<String>,
}

impl HttpGbavClient {
    pub async fn from_settings(settings: GbavSettings) -> Result<Self, gba::error::Error> {
        Self::new(
            settings.adhoc_url,
            settings.username,
            settings.password,
            settings.trust_anchor.into_certificate(),
            settings.client_cert,
            settings.client_cert_key,
            settings.ca_api_key,
        )
        .await
    }
}

pub struct SymmetricKey {
    bytes: Vec<u8>,
}

impl SymmetricKey {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    pub fn key<B>(&self) -> &Key<B>
    where
        B: KeySizeUser,
    {
        Key::<B>::from_slice(self.bytes.as_slice())
    }
}

impl<'de> Deserialize<'de> for SymmetricKey {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)
            .map(hex::decode)?
            .map(Self::new)
            .map_err(de::Error::custom)
    }
}

#[derive(Deserialize)]
pub struct PreloadedSettings {
    pub encryption_key: SymmetricKey,
    pub hmac_key: SymmetricKey,
    pub xml_path: String,
}

impl<T> FileGbavClient<T> {
    pub fn try_from_settings(settings: PreloadedSettings, client: T) -> Result<Self, ConfigError> {
        Ok(Self::new(
            Path::new(&settings.xml_path),
            settings.encryption_key,
            settings.hmac_key,
            client,
        ))
    }
}

#[derive(Deserialize)]
#[serde(rename_all(deserialize = "lowercase"))]
#[derive(strum::Display)]
pub enum RunMode {
    Gbav(GbavSettings),
    Preloaded(PreloadedSettings),
    All {
        gbav: GbavSettings,
        preloaded: PreloadedSettings,
    },
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        // Look for a config file that is in the same directory as Cargo.toml if run through cargo,
        // otherwise look in the current working directory.
        let config_path = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap_or_default();

        Config::builder()
            .set_default("ip", "0.0.0.0")?
            .set_default("port", 3008)?
            .add_source(File::from(config_path.join("gba_hc_converter.toml")).required(false))
            .add_source(
                Environment::with_prefix("gba_hc_converter")
                    .separator("__")
                    .prefix_separator("_")
                    .list_separator(","),
            )
            .build()?
            .try_deserialize()
    }
}
