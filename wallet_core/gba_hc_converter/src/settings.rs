use std::net::IpAddr;
use std::path::Path;

use config::Config;
use config::ConfigError;
use config::Environment;
use config::File;
use crypto_common::Key;
use crypto_common::KeySizeUser;
use derive_more::From;
use serde::Deserialize;
use serde::de;
use serde_with::base64::Base64;
use serde_with::serde_as;

use http_utils::reqwest::ReqwestTrustAnchor;
use http_utils::urls::BaseUrl;
use utils::path::prefix_local_path;

use crate::gba;
use crate::gba::client::FileGbavClient;
use crate::gba::client::HttpGbavClient;

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

    pub client_certificate_and_key: CertificateAndKey,

    #[serde_as(as = "Base64")]
    pub trust_anchor: ReqwestTrustAnchor,

    pub ca_api_key: Option<String>,
}

#[serde_as]
#[derive(Deserialize)]
pub struct CertificateAndKey {
    #[serde_as(as = "Base64")]
    pub certificate: Vec<u8>,

    #[serde_as(as = "Base64")]
    pub key: Vec<u8>,
}

impl HttpGbavClient {
    pub async fn from_settings(settings: GbavSettings) -> Result<Self, gba::error::Error> {
        Self::new(
            settings.adhoc_url,
            settings.username,
            settings.password,
            settings.trust_anchor.into_certificate(),
            settings.client_certificate_and_key,
            settings.ca_api_key,
        )
        .await
    }
}

#[derive(From)]
pub struct SymmetricKey {
    bytes: Vec<u8>,
}

impl SymmetricKey {
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
            .map(Into::into)
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
        Config::builder()
            .set_default("ip", "0.0.0.0")?
            .set_default("port", 8001)?
            .add_source(File::from(prefix_local_path("gba_hc_converter.toml".as_ref()).as_ref()).required(false))
            .add_source(
                Environment::with_prefix("gba_hc_converter")
                    .separator("__")
                    .prefix_separator("__")
                    .list_separator(","),
            )
            .build()?
            .try_deserialize()
    }
}
