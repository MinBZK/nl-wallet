use std::borrow::Cow;
use std::env;
use std::net::IpAddr;
use std::path::PathBuf;
use std::time::Duration;

use config::Config;
use config::ConfigError;
use config::Environment;
use config::File;
use derive_more::From;
use derive_more::Into;
use serde::Deserialize;
use serde_with::base64::Base64;
use serde_with::hex::Hex;
use serde_with::serde_as;
use serde_with::DurationMilliSeconds;
use serde_with::DurationSeconds;

use android_attest::root_public_key::RootPublicKey;
use apple_app_attest::AttestationEnvironment;
use wallet_common::config::http::TlsServerConfig;
use wallet_common::trust_anchor::BorrowingTrustAnchor;
use wallet_provider_database_settings::Database;

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct Settings {
    pub certificate_signing_key_identifier: String,
    pub instruction_result_signing_key_identifier: String,
    pub attestation_wrapping_key_identifier: String,
    pub pin_pubkey_encryption_key_identifier: String,
    pub pin_public_disclosure_protection_key_identifier: String,
    pub wte_signing_key_identifier: String,
    pub wte_issuer_identifier: String,
    pub database: Database,
    pub webserver: Webserver,
    pub tls_config: Option<TlsServerConfig>,
    pub hsm: Hsm,
    pub pin_policy: PinPolicySettings,
    pub structured_logging: bool,

    #[serde(rename = "instruction_challenge_timeout_in_ms")]
    #[serde_as(as = "DurationMilliSeconds")]
    pub instruction_challenge_timeout: Duration,

    pub ios: Ios,
    pub android: Android,
}

#[derive(Clone, Deserialize)]
pub struct Webserver {
    pub ip: IpAddr,
    pub port: u16,
}

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct PinPolicySettings {
    pub rounds: u8,
    pub attempts_per_round: u8,

    #[serde(rename = "timeouts_in_ms")]
    #[serde_as(as = "Vec<DurationMilliSeconds>")]
    pub timeouts: Vec<Duration>,
}

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct Hsm {
    pub library_path: PathBuf,
    pub user_pin: String,
    pub max_sessions: u8,

    #[serde(rename = "max_session_lifetime_in_sec")]
    #[serde_as(as = "DurationSeconds")]
    pub max_session_lifetime: Duration,
}

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct Ios {
    pub team_identifier: String,
    pub bundle_identifier: String,
    #[serde(default)]
    pub environment: AppleEnvironment,
    #[serde_as(as = "Vec<Base64>")]
    pub root_certificates: Vec<BorrowingTrustAnchor>,
}

#[derive(Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppleEnvironment {
    Development,
    #[default]
    Production,
}

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct Android {
    #[serde_as(as = "Vec<Base64>")]
    pub root_public_keys: Vec<AndroidRootPublicKey>,
    pub package_name: String,
    pub credentials_file: PathBuf,
    #[serde_as(as = "Vec<Hex>")]
    pub play_store_certificate_hashes: Vec<Vec<u8>>,
}

#[derive(Clone, From, Into)]
pub struct AndroidRootPublicKey(RootPublicKey);

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        // Look for a config file that is in the same directory as Cargo.toml if run through cargo,
        // otherwise look in the current working directory.
        let config_path = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap_or_default();

        Config::builder()
            .set_default("certificate_signing_key_identifier", "certificate_signing_key")?
            .set_default(
                "instruction_result_signing_key_identifier",
                "instruction_result_signing_key",
            )?
            .set_default("attestation_wrapping_key_identifier", "attestation_wrapping_key")?
            .set_default("pin_pubkey_encryption_key_identifier", "pin_pubkey_encryption_key")?
            .set_default(
                "pin_public_disclosure_protection_key_identifier",
                "pin_public_disclosure_protection_key",
            )?
            .set_default("wte_signing_key_identifier", "wte_signing_key")?
            .set_default("wte_issuer_identifier", "wte-issuer.example.com")?
            .set_default("webserver.ip", "0.0.0.0")?
            .set_default("webserver.port", 3000)?
            .set_default("pin_policy.rounds", 4)?
            .set_default("pin_policy.attempts_per_round", 4)?
            .set_default("pin_policy.timeouts_in_ms", vec![60_000, 300_000, 3_600_000])?
            .set_default("structured_logging", false)?
            .set_default("instruction_challenge_timeout_in_ms", 15_000)?
            .set_default("hsm.max_sessions", 10)?
            .set_default("hsm.max_session_lifetime_in_sec", 900)?
            .set_default("android.credentials_file", "google-cloud-service-account.json")?
            .set_default("android.play_store_certificate_hashes", Vec::<String>::new())?
            .add_source(File::from(config_path.join("wallet_provider.toml")).required(false))
            .add_source(
                Environment::with_prefix("wallet_provider")
                    .separator("__")
                    .prefix_separator("__")
                    .list_separator(",")
                    .with_list_parse_key("ios.root_certificates")
                    .with_list_parse_key("android.root_public_keys")
                    .with_list_parse_key("android.play_store_certificate_hashes")
                    .try_parsing(true),
            )
            .build()?
            .try_deserialize()
    }
}

impl From<AppleEnvironment> for AttestationEnvironment {
    fn from(value: AppleEnvironment) -> Self {
        match value {
            AppleEnvironment::Development => Self::Development,
            AppleEnvironment::Production => Self::Production,
        }
    }
}

impl Android {
    pub fn credentials_file_absolute(&self) -> Cow<'_, PathBuf> {
        if self.credentials_file.is_absolute() {
            return Cow::Borrowed(&self.credentials_file);
        }

        // If the path to the credentials file is not already absolute, either prepend the same directory
        // as the Cargo.toml file (if ran through cargo) or prepend the current working directory.
        let path_base = env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| env::current_dir().expect("could not get current working directory"));

        Cow::Owned(path_base.join(&self.credentials_file))
    }
}

impl TryFrom<Vec<u8>> for AndroidRootPublicKey {
    type Error = spki::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let public_key = RootPublicKey::try_from(value.as_slice())?;
        Ok(AndroidRootPublicKey(public_key))
    }
}
