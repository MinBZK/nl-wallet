use std::{
    env,
    net::IpAddr,
    path::{Path, PathBuf},
};

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use serde_with::{base64::Base64, serde_as};

use wallet_common::{reqwest::deserialize_certificate, sentry::Sentry, urls::BaseUrl};

use crate::gba::{
    self,
    client::{FileGbavClient, HttpGbavClient},
};

#[derive(Clone, Deserialize)]
pub struct Settings {
    pub ip: IpAddr,
    pub port: u16,

    #[serde(default)]
    pub structured_logging: bool,

    pub run_mode: RunMode,
    pub sentry: Option<Sentry>,
}

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct GbavSettings {
    pub adhoc_url: BaseUrl,
    pub username: String,
    pub password: String,

    #[serde_as(as = "Base64")]
    pub client_cert: Vec<u8>,

    #[serde_as(as = "Base64")]
    pub client_cert_key: Vec<u8>,

    #[serde(deserialize_with = "deserialize_certificate")]
    pub trust_anchor: reqwest::Certificate,

    pub ca_api_key: Option<String>,
}

impl HttpGbavClient {
    pub async fn from_settings(settings: GbavSettings) -> Result<Self, gba::error::Error> {
        Self::new(
            settings.adhoc_url,
            settings.username,
            settings.password,
            settings.trust_anchor,
            settings.client_cert,
            settings.client_cert_key,
            settings.ca_api_key,
        )
        .await
    }
}

#[derive(Clone, Deserialize)]
pub struct PreloadedSettings {
    pub xml_path: String,
}

impl<T> FileGbavClient<T> {
    pub fn from_settings(settings: &PreloadedSettings, client: T) -> Self {
        Self::new(Path::new(&settings.xml_path), client)
    }
}

#[derive(Clone, Deserialize)]
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
                    .list_separator("|"),
            )
            .build()?
            .try_deserialize()
    }
}
