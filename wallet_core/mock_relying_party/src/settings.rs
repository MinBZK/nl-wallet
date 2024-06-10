use std::{collections::HashMap, env, net::IpAddr, path::PathBuf};

use axum::{headers::HeaderValue, http::header::InvalidHeaderValue};
use config::{Config, ConfigError, Environment, File};
use nutype::nutype;
use serde::Deserialize;
use url::Url;

use nl_wallet_mdoc::verifier::ItemsRequests;
use wallet_common::{config::wallet_config::BaseUrl, sentry::Sentry};

#[derive(Deserialize, Clone)]
pub struct Settings {
    pub webserver: Server,
    pub internal_wallet_server_url: BaseUrl,
    pub public_wallet_server_url: BaseUrl,
    pub public_url: BaseUrl,
    #[serde(default)]
    pub allow_origins: Vec<Origin>,
    pub usecases: HashMap<String, Usecase>,
    pub sentry: Option<Sentry>,
}

#[derive(Deserialize, Clone)]
pub struct Server {
    pub ip: IpAddr,
    pub port: u16,
}

#[derive(Deserialize, Default, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ReturnUrlMode {
    #[default]
    Url,
    None,
}

#[derive(Deserialize, Clone)]
pub struct Usecase {
    #[serde(default)]
    pub return_url: ReturnUrlMode,
    pub items_requests: ItemsRequests,
}

#[nutype(validate(predicate = |u| Origin::is_valid(u)), derive(TryFrom, Deserialize, Clone))]
pub struct Origin(Url);

impl Origin {
    fn is_valid(u: &Url) -> bool {
        #[cfg(feature = "allow_http_return_url")]
        let allowed_schemes = ["https", "http"];
        #[cfg(not(feature = "allow_http_return_url"))]
        let allowed_schemes = ["https"];

        (allowed_schemes.contains(&u.scheme()))
            && u.has_host()
            && u.fragment().is_none()
            && u.query().is_none()
            && u.path() == "/"
        // trailing slash is stripped of when converting to `HeaderValue`
    }
}

impl TryFrom<Origin> for HeaderValue {
    type Error = InvalidHeaderValue;

    fn try_from(value: Origin) -> Result<Self, Self::Error> {
        let url = value.into_inner();
        let mut str = format!("{0}://{1}", url.scheme(), url.host_str().unwrap(),);
        if let Some(port) = url.port() {
            str += &format!(":{0}", port);
        }
        HeaderValue::try_from(str)
    }
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        // Look for a config file that is in the same directory as Cargo.toml if run through cargo,
        // otherwise look in the current working directory.
        let config_path = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap_or_default();

        Config::builder()
            .set_default("webserver.ip", "0.0.0.0")?
            .set_default("webserver.port", 3004)?
            .set_default("public_url", "http://localhost:3004/")?
            .add_source(File::from(config_path.join("mock_relying_party.toml")).required(false))
            .add_source(
                Environment::with_prefix("mock_relying_party")
                    .separator("__")
                    .prefix_separator("_"),
            )
            .build()?
            .try_deserialize()
    }
}
