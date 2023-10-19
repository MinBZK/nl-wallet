use std::{env, net::IpAddr, path::PathBuf};

use chrono::Duration;
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use serde_with::{serde_as, DurationMilliSeconds};

use wallet_common::account::serialization::Base64Bytes;
use wallet_provider_database_settings::{Database, DatabaseDefaults};

#[serde_as]
#[derive(Deserialize)]
pub struct Settings {
    pub certificate_signing_key_identifier: String,
    pub instruction_result_signing_key_identifier: String,
    pub pin_hash_salt: Base64Bytes,
    pub database: Database,
    pub webserver: Webserver,
    pub hsm: Hsm,
    pub pin_policy: PinPolicySettings,
    pub structured_logging: bool,
    #[serde_as(as = "DurationMilliSeconds<i64>")]
    pub instruction_challenge_timeout_in_ms: Duration,
}

#[derive(Deserialize)]
pub struct Webserver {
    pub ip: IpAddr,
    pub port: u16,
}

#[derive(Deserialize)]
pub struct PinPolicySettings {
    pub rounds: u8,
    pub attempts_per_round: u8,
    pub timeouts_in_ms: Vec<u32>,
}

#[derive(Deserialize)]
pub struct Hsm {
    pub library_path: PathBuf,
    pub user_pin: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        // Look for a config file that is in the same directory as Cargo.toml if run through cargo,
        // otherwise look in the current working directory.
        let config_path = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap_or_default();

        Config::builder()
            .database_defaults()?
            .set_default("certificate_signing_key_identifier", "certificate_signing_key")?
            .set_default(
                "instruction_result_signing_key_identifier",
                "instruction_result_signing_key",
            )?
            .set_default("webserver.ip", "0.0.0.0")?
            .set_default("webserver.port", 3000)?
            .set_default("pin_policy.rounds", 4)?
            .set_default("pin_policy.attempts_per_round", 4)?
            .set_default("pin_policy.timeouts_in_ms", vec![60_000, 300_000, 3_600_000])?
            .set_default("structured_logging", false)?
            .set_default("instruction_challenge_timeout_in_ms", 15_000)?
            .add_source(File::from(config_path.join("wallet_provider.toml")).required(false))
            .add_source(
                Environment::with_prefix("wallet_provider")
                    .separator("__")
                    .prefix_separator("_"),
            )
            .build()?
            .try_deserialize()
    }
}
