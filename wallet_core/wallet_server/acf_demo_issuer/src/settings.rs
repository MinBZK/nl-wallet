use std::collections::HashMap;
use std::path::Path;

use attestation_data::attributes::Attributes;
use config::Config;
use config::ConfigError;
use config::Environment;
use config::File;
use issuer_common::settings::AuthorizingIssuerSettings;
use issuer_common::settings::IssuerSettingsValidationError;
use openid4vc::issuable_document::CredentialKind;
use openid4vc::server_state::SessionStoreTimeouts;
use serde::Deserialize;
use serde_with::serde_as;
use server_utils::settings::NL_WALLET_CLIENT_ID;
use server_utils::settings::ServerSettings;
use server_utils::settings::Settings;
use utils::path::prefix_local_path;
use utils::vec_at_least::VecNonEmpty;

#[derive(Debug, Clone, Deserialize)]
pub struct AcfDemoIssuerSettings {
    /// The configured demo usecases, keyed by the `issuer_state` value carried in the credential offer.
    pub usecases: HashMap<String, Usecase>,

    #[serde(flatten)]
    pub authorizing_issuer_settings: AuthorizingIssuerSettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Usecase {
    /// Determines how `/authorize` is handled for this usecase. Only [`UsecaseKind::Consent`] is
    /// implemented (phase 1); [`UsecaseKind::Immediate`] is reserved for the public-QR variant.
    pub kind: UsecaseKind,

    /// The documents to issue for this usecase.
    pub documents: VecNonEmpty<IssuableDocumentTemplate>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsecaseKind {
    /// `/authorize` redirects to a consent page; a callback completes the authorization.
    Consent,
    /// `/authorize` mints the authorization code immediately (phase 2, not yet implemented).
    Immediate,
}

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct IssuableDocumentTemplate {
    #[serde(flatten)]
    pub credential_kind: CredentialKind,
    pub attributes: Attributes,
}

impl IssuableDocumentTemplate {
    #[cfg(test)]
    pub(crate) fn new(credential_kind: CredentialKind, attributes: Attributes) -> Self {
        Self {
            credential_kind,
            attributes,
        }
    }
}

impl ServerSettings for AcfDemoIssuerSettings {
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
            .set_default("status_lists.serve", false)?
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

        // Look for a config file that is in the same directory as Cargo.toml if run through cargo,
        // otherwise look in the current working directory.
        let config_source = prefix_local_path(Path::new(config_file));

        let environment_parser = Environment::with_prefix(env_prefix)
            .separator("__")
            .prefix_separator("__")
            .list_separator(",")
            .with_list_parse_key("issuer_trust_anchors")
            .with_list_parse_key("wia_trust_anchors")
            .with_list_parse_key("metadata")
            .with_list_parse_key("wallet_redirect_uris")
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
        self.authorizing_issuer_settings.issuer_settings.validate()
    }

    fn server_settings(&self) -> &Settings {
        &self.authorizing_issuer_settings.issuer_settings.server_settings
    }
}
