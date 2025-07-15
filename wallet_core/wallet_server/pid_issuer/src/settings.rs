use config::Config;
use config::ConfigError;
use config::Environment;
use config::File;
use serde::Deserialize;
use serde_with::base64::Base64;
use serde_with::serde_as;

use crypto::p256_der::DerVerifyingKey;
use http_utils::tls::pinning::TlsPinningConfig;
use http_utils::urls::BaseUrl;
use issuer_settings::settings::IssuerSettings;
use issuer_settings::settings::IssuerSettingsError;
use openid4vc::server_state::SessionStoreTimeouts;
use server_utils::settings::SecretKey;
use server_utils::settings::ServerSettings;
use server_utils::settings::Settings;
use server_utils::settings::NL_WALLET_CLIENT_ID;
use utils::path::prefix_local_path;

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct PidIssuerSettings {
    pub digid: Digid,
    pub brp_server: BaseUrl,

    #[serde_as(as = "Base64")]
    pub wte_issuer_pubkey: DerVerifyingKey,

    pub recovery_code: SecretKey,

    #[serde(flatten)]
    pub issuer_settings: IssuerSettings,
}

#[derive(Clone, Deserialize)]
pub struct Digid {
    pub bsn_privkey: String,
    pub http_config: TlsPinningConfig,
}

impl ServerSettings for PidIssuerSettings {
    type ValidationError = IssuerSettingsError;

    fn new(config_file: &str, env_prefix: &str) -> Result<Self, ConfigError> {
        let default_store_timeouts = SessionStoreTimeouts::default();

        let config_builder = Config::builder()
            .set_default("wallet_server.ip", "0.0.0.0")?
            .set_default("wallet_server.port", 8001)?
            .set_default("public_url", "http://localhost:8001/")?
            .set_default("log_requests", false)?
            .set_default("structured_logging", false)?
            .set_default("storage.url", "memory://")?
            .set_default(
                "storage.expiration_minutes",
                default_store_timeouts.expiration.as_secs() / 60,
            )?
            .set_default(
                "storage.successful_deletion_minutes",
                default_store_timeouts.successful_deletion.as_secs() / 60,
            )?
            .set_default(
                "storage.failed_deletion_minutes",
                default_store_timeouts.failed_deletion.as_secs() / 60,
            )?
            .set_default("wallet_client_ids", vec![NL_WALLET_CLIENT_ID.to_string()])?
            .set_default("brp_server", "http://localhost:3011/")?;

        // Look for a config file that is in the same directory as Cargo.toml if run through cargo,
        // otherwise look in the current working directory.
        let config_source = prefix_local_path(config_file.as_ref());

        let environment_parser = Environment::with_prefix(env_prefix)
            .separator("__")
            .prefix_separator("__")
            .list_separator(",")
            .with_list_parse_key("issuer_trust_anchors")
            .with_list_parse_key("digid.http_config.trust_anchors")
            .with_list_parse_key("metadata")
            .try_parsing(true);

        let config = config_builder
            .add_source(File::from(config_source.as_ref()).required(false))
            .add_source(File::from(config_file.as_ref()).required(false))
            .add_source(environment_parser)
            .build()?
            .try_deserialize()?;

        Ok(config)
    }

    fn validate(&self) -> Result<(), IssuerSettingsError> {
        self.issuer_settings.validate()
    }

    fn server_settings(&self) -> &Settings {
        &self.issuer_settings.server_settings
    }
}
