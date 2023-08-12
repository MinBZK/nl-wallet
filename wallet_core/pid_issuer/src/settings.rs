use std::{
    env,
    net::IpAddr,
    path::{Path, PathBuf},
};

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use url::Url;

#[derive(Deserialize)]
pub struct Settings {
    pub webserver: Webserver,
    pub digid: Digid,
    pub issuer_key: IssuerKey,
    pub public_url: Url,
}

#[derive(Deserialize)]
pub struct Digid {
    pub issuer_url: Url,
    pub bsn_privkey: PathBuf,
    pub client_id: String,
}

#[derive(Deserialize)]
pub struct Webserver {
    pub ip: IpAddr,
    pub port: u16,
}

#[derive(Deserialize)]
pub struct IssuerKey {
    pub private_key: PathBuf,
    pub certificate: PathBuf,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        // Look for a config file that is in the same directory as Cargo.toml if run through cargo,
        // otherwise look in the current working directory.
        let config_path = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap_or_default();

        // TODO: use separate client ID for mock PID issuer.
        Config::builder()
            .set_default("webserver.ip", "0.0.0.0")?
            .set_default("webserver.port", 3003)?
            .set_default("public_url", "http://localhost:3003/mdoc/")?
            .set_default(
                "digid.issuer_url",
                "https://example.com/digid-connector",
            )?
            .set_default("digid.bsn_privkey", secrets_path(&config_path, "bsn_private_key.jwk"))?
            .set_default("digid.client_id", "SSSS")?
            .set_default(
                "issuer_key.private_key",
                secrets_path(&config_path, "issuer_privkey.pem"),
            )?
            .set_default("issuer_key.certificate", secrets_path(&config_path, "issuer_cert.pem"))?
            .add_source(File::from(config_path.join("config")).required(false))
            .add_source(
                Environment::with_prefix("pid_issuer")
                    .separator("__")
                    .prefix_separator("_"),
            )
            .build()?
            .try_deserialize()
    }
}

fn secrets_path(config_path: &Path, filename: &str) -> String {
    config_path.join("secrets").join(filename).to_str().unwrap().to_string()
}
