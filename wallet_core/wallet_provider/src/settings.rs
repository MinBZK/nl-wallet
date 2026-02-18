use std::collections::HashMap;
use std::collections::HashSet;
use std::net::IpAddr;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use config::Config;
use config::ConfigError;
use config::Environment;
use config::File;
use crypto::server_keys::KeyPair;
use crypto::x509::CertificateError;
use derive_more::From;
use derive_more::Into;
use serde::Deserialize;
use serde_with::DurationMilliSeconds;
use serde_with::base64::Base64;
use serde_with::hex::Hex;
use serde_with::serde_as;
use url::Url;

use android_attest::play_integrity::verification::InstallationMethod;
use android_attest::root_public_key::RootPublicKey;
use apple_app_attest::AttestationEnvironment;
use crypto::trust_anchor::BorrowingTrustAnchor;
use crypto::x509::BorrowingCertificate;
use hsm::keys::HsmEcdsaKey;
use hsm::service::Pkcs11Hsm;
use hsm::settings::Hsm;
use http_utils::tls::server::TlsServerConfig;
use http_utils::urls::BaseUrl;
use status_lists::config::StatusListConfig;
use status_lists::publish::PublishDir;
use status_lists::settings::ExpiryLessThanTtl;
use status_lists::settings::StatusListsSettings;
use utils::path::prefix_local_path;
use utils::vec_at_least::VecNonEmpty;
use wallet_provider_persistence::database::ConnectionOptions;

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct Settings {
    pub certificate_signing_key_identifier: String,
    pub instruction_result_signing_key_identifier: String,
    pub attestation_wrapping_key_identifier: String,
    pub pin_pubkey_encryption_key_identifier: String,
    pub pin_public_disclosure_protection_key_identifier: String,
    pub revocation_code_key_identifier: String,
    pub wua_signing_key_identifier: String,
    pub wua_issuer_identifier: String,
    pub wua_valid_days: u64,
    pub recovery_code_paths: HashMap<String, VecNonEmpty<String>>,
    pub database: DatabaseSettings,
    pub audit_log: DatabaseSettings,
    pub webserver: Webserver,
    pub tls_config: Option<TlsServerConfig>,
    pub hsm: Hsm,
    pub pin_policy: PinPolicySettings,
    pub structured_logging: bool,
    pub capture_and_redirect_logging: Option<PathBuf>,
    pub max_transfer_upload_size_in_bytes: usize,

    pub wua_status_list: WuaStatusListsSettings,

    #[serde(rename = "instruction_challenge_timeout_in_ms")]
    #[serde_as(as = "DurationMilliSeconds")]
    pub instruction_challenge_timeout: Duration,

    /// Issuer trust anchors are used to validate the received PID SD JWT with Recovery Code disclosure
    #[serde_as(as = "Vec<Base64>")]
    pub pid_issuer_trust_anchors: Vec<BorrowingTrustAnchor>,

    pub ios: Ios,
    pub android: Android,
}

#[derive(Clone, Deserialize)]
pub struct DatabaseSettings {
    pub url: Url,
    pub options: ConnectionOptions,
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
pub struct WuaStatusListsSettings {
    #[serde(flatten)]
    pub list_settings: StatusListsSettings,

    /// Base url for the status list
    pub base_url: BaseUrl,
    /// Path to directory for the published status list
    pub publish_dir: PublishDir,

    // HSM key identifier used to sign
    pub key_identifier: String,
    // X509 certificate containing the public key
    #[serde_as(as = "Base64")]
    pub key_certificate: BorrowingCertificate,
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
            .set_default("revocation_code_key_identifier", "revocation_code_key")?
            .set_default("wua_status_list.list_size", 100_000)?
            .set_default("wua_status_list.create_threshold_ratio", 0.01)?
            .set_default("wua_status_list.expiry_in_hours", 24)?
            .set_default("wua_status_list.refresh_threshold_ratio", 0.25)?
            .set_default("wua_status_list.key_identifier", "wua_tsl_key")?
            .set_default("wua_signing_key_identifier", "wua_signing_key")?
            .set_default("wua_issuer_identifier", "wua-issuer.example.com")?
            .set_default("wua_valid_days", 365)?
            .set_default("audit_log.options.connect_timeout_in_sec", "3")?
            .set_default("audit_log.options.max_connections", "10")?
            .set_default("database.options.connect_timeout_in_sec", "3")?
            .set_default("database.options.max_connections", "10")?
            .set_default("webserver.port", 8001)?
            .set_default("webserver.ip", "0.0.0.0")?
            .set_default("webserver.port", 8001)?
            .set_default("pin_policy.rounds", 4)?
            .set_default("pin_policy.attempts_per_round", 4)?
            .set_default("pin_policy.timeouts_in_ms", vec![60_000, 300_000, 3_600_000])?
            .set_default("structured_logging", false)?
            .set_default("instruction_challenge_timeout_in_ms", 60_000)?
            .set_default("hsm.max_sessions", 10)?
            .set_default("hsm.max_session_lifetime_in_sec", 900)?
            .set_default("android.allow_sideloading", false)?
            .set_default("android.credentials_file", "google-cloud-service-account.json")?
            .set_default("android.play_store_certificate_hashes", Vec::<String>::new())?
            .set_default("max_transfer_upload_size_in_bytes", 100_000_000)?
            .add_source(File::from(prefix_local_path(Path::new("wallet_provider.toml")).as_ref()).required(false))
            .add_source(
                Environment::with_prefix("wallet_provider")
                    .separator("__")
                    .prefix_separator("__")
                    .list_separator(",")
                    .with_list_parse_key("ios.root_certificates")
                    .with_list_parse_key("android.root_public_keys")
                    .with_list_parse_key("android.play_store_certificate_hashes")
                    .with_list_parse_key("pid_issuer_trust_anchors")
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

#[derive(Debug, thiserror::Error)]
pub enum WuaStatusListsSettingsError {
    #[error("incorrectly configured WUA expiration: {0}")]
    ExpiryLessThanTtl(#[from] ExpiryLessThanTtl),

    #[error("incorrectly configured WUA status list key identifier or certificate: {0}")]
    PrivateKey(#[from] CertificateError),
}

impl WuaStatusListsSettings {
    pub async fn into_config(
        self,
        hsm: Pkcs11Hsm,
    ) -> Result<StatusListConfig<HsmEcdsaKey>, WuaStatusListsSettingsError> {
        let key_pair = KeyPair::new(HsmEcdsaKey::new(self.key_identifier, hsm), self.key_certificate).await?;

        let config = self
            .list_settings
            .to_config(self.base_url, self.publish_dir, key_pair)?;

        Ok(config)
    }
}
