use std::collections::HashMap;

use config::Config;
use config::ConfigError;
use config::Environment;
use config::File;
use serde::Deserialize;
use serde_with::base64::Base64;
use serde_with::serde_as;

use crypto::trust_anchor::BorrowingTrustAnchor;
use crypto::x509::CertificateUsage;
use issuer_settings::settings::IssuerSettings;
use issuer_settings::settings::IssuerSettingsError;
use mdoc::utils::x509::CertificateType;
use mdoc::verifier::ItemsRequests;
use openid4vc::server_state::SessionStoreTimeouts;
use rustls_pki_types::TrustAnchor;
use server_utils::settings::verify_key_pairs;
use server_utils::settings::KeyPair;
use server_utils::settings::ServerSettings;
use server_utils::settings::Settings;
use server_utils::settings::NL_WALLET_CLIENT_ID;
use wallet_common::generator::TimeGenerator;
use wallet_common::urls::BaseUrl;
use wallet_common::urls::DEFAULT_UNIVERSAL_LINK_BASE;
use wallet_common::utils;

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct IssuanceServerSettings {
    pub disclosure_settings: HashMap<String, AttestationSettings>,

    #[serde(flatten)]
    pub issuer_settings: IssuerSettings,

    /// Reader trust anchors are used to verify the keys and certificates in the `disclosure_settings` configuration on
    /// application startup.
    #[serde_as(as = "Vec<Base64>")]
    pub reader_trust_anchors: Vec<BorrowingTrustAnchor>,

    pub universal_link_base_url: BaseUrl,
}

#[derive(Clone, Deserialize)]
pub struct AttestationSettings {
    #[serde(flatten)]
    pub key_pair: KeyPair,
    pub to_disclose: ItemsRequests,
    pub attestation_url: BaseUrl,
}

impl ServerSettings for IssuanceServerSettings {
    type ValidationError = IssuerSettingsError;

    fn new(config_file: &str, env_prefix: &str) -> Result<Self, ConfigError> {
        let default_store_timeouts = SessionStoreTimeouts::default();

        let config_builder = Config::builder()
            .set_default("wallet_server.ip", "0.0.0.0")?
            .set_default("wallet_server.port", 3001)?
            .set_default("public_url", "http://localhost:3002/")?
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
            .set_default("universal_link_base_url", DEFAULT_UNIVERSAL_LINK_BASE)?;

        // Look for a config file that is in the same directory as Cargo.toml if run through cargo,
        // otherwise look in the current working directory.
        let config_source = utils::prefix_local_path(config_file.as_ref());

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

    fn validate(&self) -> Result<(), IssuerSettingsError> {
        self.issuer_settings.validate()?;

        let time = TimeGenerator;

        let trust_anchors: Vec<TrustAnchor<'_>> = self
            .reader_trust_anchors
            .iter()
            .map(BorrowingTrustAnchor::to_owned_trust_anchor)
            .collect::<Vec<_>>();

        let key_pairs: Vec<(&str, &KeyPair)> = self
            .disclosure_settings
            .iter()
            .map(|(id, settings)| (id.as_ref(), &settings.key_pair))
            .collect();

        verify_key_pairs(
            &key_pairs,
            &trust_anchors,
            CertificateUsage::ReaderAuth,
            &time,
            |certificate_type| matches!(certificate_type, CertificateType::ReaderAuth(Some(_))),
        )?;

        Ok(())
    }

    fn server_settings(&self) -> &Settings {
        &self.issuer_settings.server_settings
    }
}
