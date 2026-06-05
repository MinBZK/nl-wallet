use std::path::Path;

use config::Config;
use config::ConfigError;
use config::Environment;
use config::File;
use issuer_common::settings::IssuerSettings;
use issuer_common::settings::IssuerSettingsValidationError;
use openid4vc::server_state::SessionStoreTimeouts;
use serde::Deserialize;
use server_utils::settings::NL_WALLET_CLIENT_ID;
use server_utils::settings::ServerSettings;
use server_utils::settings::Settings;
use utils::path::prefix_local_path;

#[derive(Debug, Clone, Deserialize)]
pub struct PacfIssuanceServerSettings(pub IssuerSettings);

impl ServerSettings for PacfIssuanceServerSettings {
    type ValidationError = IssuerSettingsValidationError;

    fn new(config_file: &str, env_prefix: &str) -> Result<Self, ConfigError> {
        let default_store_timeouts = SessionStoreTimeouts::default();

        let config_builder = Config::builder()
            .set_default("wallet_server.ip", "0.0.0.0")?
            .set_default("wallet_server.port", 8001)?
            .set_default("internal_server.ip", "0.0.0.0")?
            .set_default("internal_server.port", 8002)?
            .set_default("public_url", "http://localhost:8001/")?
            .set_default("log_requests", false)?
            .set_default("structured_logging", false)?
            .set_default("status_lists.list_size", 100_000)?
            .set_default("status_lists.create_threshold_ratio", 0.1)?
            .set_default("status_lists.expiry_in_hours", 24)?
            .set_default("status_lists.refresh_threshold_ratio", 0.25)?
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
            .set_default("wallet_client_ids", vec![NL_WALLET_CLIENT_ID.to_string()])?;

        let config_source = prefix_local_path(Path::new(config_file));

        let environment_parser = Environment::with_prefix(env_prefix)
            .separator("__")
            .prefix_separator("__")
            .list_separator(",")
            .with_list_parse_key("issuer_trust_anchors")
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

    fn validate(&self) -> Result<(), IssuerSettingsValidationError> {
        self.0.validate()?;
        Ok(())
    }

    fn server_settings(&self) -> &Settings {
        &self.0.server_settings
    }
}
