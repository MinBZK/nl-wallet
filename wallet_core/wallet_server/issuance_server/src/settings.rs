use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use axum::Router;
use config::Config;
use config::ConfigError;
use config::Environment;
use config::File;
use crypto::trust_anchor::TrustAnchors;
use crypto::x509::CertificateUsage;
use dcql::Query;
use dcql::normalized::UnsupportedDcqlFeatures;
use derive_more::Debug;
use futures::future::try_join_all;
use hsm::service::Pkcs11Hsm;
use http_utils::client::TlsPinningConfig;
use http_utils::urls::BaseUrl;
use http_utils::urls::DEFAULT_UNIVERSAL_LINK_BASE;
use issuer_common::IssuanceServerIssuer;
use issuer_common::settings::IssuerSettings;
use issuer_common::settings::IssuerSettingsValidationError;
use itertools::Itertools;
use openid4vc::credential_offer::OPENID4VCI_CREDENTIAL_OFFER_URL_SCHEME;
use openid4vc::server_state::SessionStoreTimeouts;
use openid4vc::verifier::DisclosureData;
use openid4vc::verifier::SessionTypeReturnUrl;
use openid4vc::verifier::WalletInitiatedUseCase;
use openid4vc::verifier::WalletInitiatedUseCases;
use openid4vc_server::verifier::VerifierFactory;
use serde::Deserialize;
use serde_with::serde_as;
use server_utils::keys::PrivateKeySettingsError;
use server_utils::settings::KeyPair;
use server_utils::settings::NL_WALLET_CLIENT_ID;
use server_utils::settings::ServerSettings;
use server_utils::settings::Settings;
use server_utils::settings::verify_key_pairs;
use server_utils::status_list_token_cache_settings::StatusListTokenCacheSettings;
use server_utils::store::SessionStoreVariant;
use token_status_list::verification::reqwest::HttpStatusListClient;
use token_status_list::verification::verifier::RevocationVerifier;
use utils::generator::TimeGenerator;
use utils::path::prefix_local_path;
use utils::vec_at_least::VecNonEmpty;

use crate::disclosure::HttpAttributesFetcher;
use crate::disclosure::IssuanceResultHandler;

#[derive(Debug, Clone, Deserialize)]
pub struct IssuanceServerSettings {
    #[serde(flatten)]
    pub issuer_settings: IssuerSettings,

    #[serde(flatten)]
    pub verifier_settings: VerifierSettings,

    /// Configuration for caching status list tokens.
    #[serde(default)]
    pub status_list_token_cache_settings: StatusListTokenCacheSettings,
}

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct VerifierSettings {
    pub disclosure_settings: HashMap<String, AttestationSettings>,

    /// Reader trust anchors are used to verify the keys and certificates in the `disclosure_settings` configuration on
    /// application startup.
    #[debug(skip)]
    pub reader_trust_anchors: TrustAnchors,

    pub universal_link_base_url: BaseUrl,

    /// Indicate per vct what extending vcts are accepted during disclosure.
    pub extending_vct_values: Option<HashMap<String, VecNonEmpty<String>>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AttestationSettings {
    #[serde(flatten)]
    #[debug(skip)]
    pub key_pair: KeyPair,
    pub dcql_query: Query,

    /// Endpoint to which the disclosed attributes get sent and which has to respond with the attestations to be issued
    /// (or an empty JSON array if none).
    #[debug(skip)]
    pub attestation_url_config: TlsPinningConfig,
}

impl IssuanceServerSettings {
    pub fn to_revocation_verifier(
        &self,
        status_list_client: HttpStatusListClient,
    ) -> RevocationVerifier<HttpStatusListClient> {
        RevocationVerifier::new(
            Arc::new(status_list_client),
            self.status_list_token_cache_settings.capacity,
            self.status_list_token_cache_settings.default_ttl,
            self.status_list_token_cache_settings.error_ttl,
            TimeGenerator,
        )
    }
}

impl ServerSettings for IssuanceServerSettings {
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
            .set_default("wallet_client_ids", vec![NL_WALLET_CLIENT_ID.to_string()])?
            .set_default("universal_link_base_url", DEFAULT_UNIVERSAL_LINK_BASE)?;

        // Look for a config file that is in the same directory as Cargo.toml if run through cargo,
        // otherwise look in the current working directory.
        let config_source = prefix_local_path(Path::new(config_file));

        let environment_parser = Environment::with_prefix(env_prefix)
            .separator("__")
            .prefix_separator("__")
            .list_separator(",")
            .with_list_parse_key("issuer_trust_anchors")
            .with_list_parse_key("reader_trust_anchors")
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
        self.issuer_settings.validate()?;

        self.verifier_settings.validate()?;

        Ok(())
    }

    fn server_settings(&self) -> &Settings {
        &self.issuer_settings.server_settings
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VerifierSettingsError {
    #[error("invalid disclosure private key: {0}")]
    PrivateKey(#[source] PrivateKeySettingsError),

    #[error("unsupported DCQL query: {0}")]
    Dcql(#[source] UnsupportedDcqlFeatures),

    #[error("could not initialize attributes fetcher: {0}")]
    AttributesFetcher(#[source] reqwest::Error),
}

impl VerifierSettings {
    fn validate(&self) -> Result<(), IssuerSettingsValidationError> {
        let time = TimeGenerator;

        let key_pairs: Vec<(&str, &KeyPair)> = self
            .disclosure_settings
            .iter()
            .map(|(id, settings)| (id.as_ref(), &settings.key_pair))
            .collect();

        verify_key_pairs(
            &key_pairs,
            &self.reader_trust_anchors,
            CertificateUsage::ReaderAuth,
            &time,
        )?;

        Ok(())
    }

    pub async fn into_disclosure_router(
        self,
        hsm: Option<Pkcs11Hsm>,
        issuer: Arc<IssuanceServerIssuer>,
        disclosure_sessions: SessionStoreVariant<DisclosureData>,
        revocation_verifier: RevocationVerifier<HttpStatusListClient>,
        server_settings: &Settings,
    ) -> Result<Router, VerifierSettingsError> {
        let use_case_count = self.disclosure_settings.len();
        let (use_case_futures, url_configs) = self
            .disclosure_settings
            .into_iter()
            .zip_eq(std::iter::repeat_n(hsm, use_case_count))
            .map(|((id, attestation), hsm)| {
                let use_case_id = id.clone();
                let use_case_future = async {
                    let key_pair = attestation
                        .key_pair
                        .parse(hsm)
                        .await
                        .map_err(VerifierSettingsError::PrivateKey)?;

                    let use_case = WalletInitiatedUseCase::new(
                        key_pair,
                        SessionTypeReturnUrl::Both,
                        attestation.dcql_query.try_into().map_err(VerifierSettingsError::Dcql)?,
                        format!("{OPENID4VCI_CREDENTIAL_OFFER_URL_SCHEME}://").parse().unwrap(),
                    );

                    Ok((use_case_id, use_case))
                };

                (use_case_future, (id, attestation.attestation_url_config))
            })
            .unzip::<_, _, Vec<_>, HashMap<_, _>>();

        let use_cases = try_join_all(use_case_futures)
            .await?
            .into_iter()
            .collect::<HashMap<_, _>>();
        let use_cases = WalletInitiatedUseCases::new(use_cases);

        let attributes_fetcher =
            HttpAttributesFetcher::try_new(url_configs).map_err(VerifierSettingsError::AttributesFetcher)?;

        let factory = VerifierFactory::new(
            issuer.issuer_identifier().as_base_url().join_base_url("disclosure"),
            self.universal_link_base_url,
            use_cases,
            server_settings.issuer_trust_anchors.clone(),
            issuer.accepted_wallet_client_ids().map(str::to_string).collect_vec(),
            self.extending_vct_values.unwrap_or_default(),
        );

        let result_handler = IssuanceResultHandler {
            issuer,
            attributes_fetcher,
        };

        let router = factory.create_wallet_router(
            Arc::new(disclosure_sessions),
            revocation_verifier,
            Some(Box::new(result_handler)),
        );

        Ok(router)
    }
}
