use std::collections::HashSet;
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

use android_attest::play_integrity::verification::InstallationMethod;
use android_attest::root_public_key::RootPublicKey;
use apple_app_attest::AttestationEnvironment;
use configuration::http::TlsServerConfig;
use hsm::settings::Hsm;
use wallet_common::trust_anchor::BorrowingTrustAnchor;
use wallet_common::utils;
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
    pub allow_sideloading: bool,
    pub credentials_file: PathBuf,
    #[serde_as(as = "HashSet<Hex>")]
    pub play_store_certificate_hashes: HashSet<Vec<u8>>,
}

#[derive(Clone, From, Into)]
pub struct AndroidRootPublicKey(RootPublicKey);

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
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
            .set_default("android.allow_sideloading", false)?
            .set_default("android.credentials_file", "google-cloud-service-account.json")?
            .set_default("android.play_store_certificate_hashes", Vec::<String>::new())?
            .add_source(File::from(utils::prefix_local_path("wallet_provider.toml".as_ref()).as_ref()).required(false))
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
    pub fn installation_method(&self) -> InstallationMethod {
        if self.allow_sideloading {
            InstallationMethod::SideloadOrPlayStore
        } else {
            InstallationMethod::PlayStore
        }
    }
}

impl TryFrom<Vec<u8>> for AndroidRootPublicKey {
    type Error = spki::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let public_key = RootPublicKey::try_from(value.as_slice())?;
        Ok(AndroidRootPublicKey(public_key))
    }
}
