use std::{env, net::IpAddr, path::PathBuf};

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use url::Url;

#[derive(Deserialize)]
pub struct Settings {
    pub webserver: Webserver,
    pub digid: DigiD,
}

#[derive(Deserialize)]
pub struct DigiD {
    pub issuer_url: Url,
    pub bsn_privkey: PathBuf,
    pub wallet_client_id: String,
}

#[derive(Deserialize)]
pub struct Webserver {
    pub ip: IpAddr,
    pub port: u16,
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
            .set_default(
                "digid.issuer_url",
                "https://example.com/digid-connector",
            )?
            .set_default(
                "digid.bsn_privkey",
                config_path.join("secrets").join("private_key.jwk").to_str().unwrap(),
            )?
            .set_default("digid.wallet_client_id", "SSSS")?
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
