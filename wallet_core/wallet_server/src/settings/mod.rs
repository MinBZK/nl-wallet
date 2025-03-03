use std::net::IpAddr;
use std::num::NonZeroU64;
use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use config::Config;
use config::ConfigError;
use config::Environment;
use config::File;
use rustls_pki_types::TrustAnchor;
use serde::Deserialize;
use serde_with::base64::Base64;
use serde_with::serde_as;
use url::Url;

use hsm::service::Pkcs11Hsm;
use hsm::settings::Hsm;
use nl_wallet_mdoc::server_keys::KeyPair as ParsedKeyPair;
use nl_wallet_mdoc::utils::x509::BorrowingCertificate;
use nl_wallet_mdoc::utils::x509::CertificateError;
use nl_wallet_mdoc::utils::x509::CertificateType;
use nl_wallet_mdoc::utils::x509::CertificateUsage;
use openid4vc::server_state::SessionStoreTimeouts;
use openid4vc_server::urls::Urls;
use wallet_common::generator::Generator;
use wallet_common::generator::TimeGenerator;
use wallet_common::p256_der::DerSigningKey;
use wallet_common::trust_anchor::BorrowingTrustAnchor;
use wallet_common::utils;

use crate::server::keys::PrivateKeySettingsError;
use crate::server::keys::PrivateKeyVariant;

cfg_if::cfg_if! {
    if #[cfg(feature = "disclosure")] {
        mod disclosure;
        pub use disclosure::*;
        use wallet_common::urls::DEFAULT_UNIVERSAL_LINK_BASE;
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "issuance")] {
        mod issuance;
        pub use issuance::*;
    }
}

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct Settings {
    // used by the wallet, MUST be reachable from the public internet.
    pub wallet_server: Server,
    // used by the application, SHOULD be reachable only by the application.
    // if not configured the wallet_server will be used, but an api_key is required in that case
    // if it conflicts with wallet_server, the application will crash on startup
    #[cfg(feature = "disclosure")]
    pub requester_server: RequesterAuth,

    #[serde(flatten)]
    pub urls: Urls,

    pub log_requests: bool,
    pub structured_logging: bool,

    pub storage: Storage,

    #[cfg(feature = "issuance")]
    pub issuer: Issuer,

    /// Issuer trust anchors are used to validate the keys and certificates in the `issuer.private_keys` configuration
    /// on application startup and the issuer of the disclosed attributes during disclosure sessions.
    #[serde_as(as = "Vec<Base64>")]
    pub issuer_trust_anchors: Vec<BorrowingTrustAnchor>,

    #[cfg(feature = "disclosure")]
    pub verifier: Verifier,

    pub hsm: Option<Hsm>,

    /// Reader trust anchors are used to verify the keys and certificates in the `verifier.usecases` configuration on
    /// application startup.
    #[cfg(feature = "disclosure")]
    #[serde_as(as = "Vec<Base64>")]
    pub reader_trust_anchors: Vec<BorrowingTrustAnchor>,
}

#[derive(Clone, Deserialize)]
pub struct Server {
    pub ip: IpAddr,
    pub port: u16,
}

#[derive(Clone, Deserialize)]
pub enum RequesterAuth {
    #[serde(rename = "authentication")]
    Authentication(Authentication),
    #[serde(untagged)]
    ProtectedInternalEndpoint {
        authentication: Authentication,
        #[serde(flatten)]
        server: Server,
    },
    #[serde(untagged)]
    InternalEndpoint(Server),
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Authentication {
    ApiKey(String),
}

#[derive(Clone, Deserialize)]
pub struct Storage {
    /// Supported schemes are: `memory://` (default) and `postgres://`.
    pub url: Url,
    pub expiration_minutes: NonZeroU64,
    pub successful_deletion_minutes: NonZeroU64,
    pub failed_deletion_minutes: NonZeroU64,
}

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct KeyPair {
    #[serde_as(as = "Base64")]
    pub certificate: BorrowingCertificate,
    pub private_key: PrivateKey,
}

#[serde_as]
#[derive(Clone, Deserialize)]
#[serde(untagged)] // TODO: replace this with `#[serde(rename_all = "snake_case")]` when implementing PVW-4007
pub enum PrivateKey {
    Software(#[serde_as(as = "Base64")] DerSigningKey),
    Hardware(String),
}

impl From<&Storage> for SessionStoreTimeouts {
    fn from(value: &Storage) -> Self {
        SessionStoreTimeouts {
            expiration: Duration::from_secs(60 * value.expiration_minutes.get()),
            successful_deletion: Duration::from_secs(60 * value.successful_deletion_minutes.get()),
            failed_deletion: Duration::from_secs(60 * value.failed_deletion_minutes.get()),
        }
    }
}

pub trait TryFromKeySettings<SRC>: Sized {
    type Error;
    async fn try_from_key_settings(source: SRC, hsm: Option<Pkcs11Hsm>) -> Result<Self, Self::Error>;
}

impl TryFromKeySettings<KeyPair> for ParsedKeyPair<PrivateKeyVariant> {
    type Error = PrivateKeySettingsError;

    async fn try_from_key_settings(source: KeyPair, hsm: Option<Pkcs11Hsm>) -> Result<Self, Self::Error> {
        let private_key = PrivateKeyVariant::from_settings(source.private_key, hsm)?;
        let key_pair = ParsedKeyPair::new(private_key, source.certificate).await?;
        Ok(key_pair)
    }
}

#[cfg(feature = "integration_test")]
impl From<ParsedKeyPair> for KeyPair {
    fn from(value: ParsedKeyPair) -> Self {
        Self {
            certificate: value.certificate().clone(),
            private_key: PrivateKey::Software(value.private_key().clone().into()),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CertificateVerificationError {
    #[error("missing trust anchors, expected at least 1")]
    MissingTrustAnchors,
    #[error("invalid certificate `{1}`: {0}")]
    InvalidCertificate(#[source] CertificateError, String),
    #[error("invalid key pair `{1}`: {0}")]
    InvalidKeyPair(#[source] CertificateError, String),
    #[error("no CertificateType found in certificate `{1}`: {0}")]
    NoCertificateType(#[source] CertificateError, String),
    #[error("certificate `{0}` is missing CertificateType registration data")]
    IncompleteCertificateType(String),
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        Settings::new_custom("wallet_server.toml", "wallet_server")
    }

    pub fn new_custom(config_file: &str, env_prefix: &str) -> Result<Self, ConfigError> {
        let default_store_timeouts = SessionStoreTimeouts::default();

        let config_builder = Config::builder()
            .set_default("wallet_server.ip", "0.0.0.0")?
            .set_default("wallet_server.port", 3001)?
            .set_default("public_url", "http://localhost:3001/")?
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
            )?;

        #[cfg(feature = "disclosure")]
        let config_builder = config_builder
            .set_default("universal_link_base_url", DEFAULT_UNIVERSAL_LINK_BASE)?
            .set_default("requester_server.ip", "0.0.0.0")?
            .set_default("requester_server.port", 3002)?;

        #[cfg(feature = "issuance")]
        let config_builder = config_builder
            .set_default(
                "issuer.wallet_client_ids",
                vec![wallet_common::jwt::NL_WALLET_CLIENT_ID.to_string()],
            )?
            .set_default("issuer.brp_server", "http://localhost:3007/")?
            .set_default("issuer.valid_days", 365)?
            .set_default("issuer.copy_count", 4)?;

        // Look for a config file that is in the same directory as Cargo.toml if run through cargo,
        // otherwise look in the current working directory.
        let config_source = utils::prefix_local_path(config_file.as_ref());

        let environment_parser = Environment::with_prefix(env_prefix)
            .separator("__")
            .prefix_separator("__")
            .list_separator(",");

        let environment_parser = environment_parser.with_list_parse_key("issuer_trust_anchors");

        #[cfg(feature = "disclosure")]
        let environment_parser = environment_parser.with_list_parse_key("reader_trust_anchors");

        #[cfg(feature = "issuance")]
        let environment_parser = environment_parser.with_list_parse_key("issuer.digid.http_config.trust_anchors");
        #[cfg(feature = "issuance")]
        let environment_parser = environment_parser.with_list_parse_key("issuer.metadata");

        let environment_parser = environment_parser.try_parsing(true);

        let config = config_builder
            .add_source(File::from(config_source.as_ref()).required(false))
            .add_source(File::from(config_file.as_ref()).required(false))
            .add_source(environment_parser)
            .build()?
            .try_deserialize()?;

        Ok(config)
    }

    pub fn verify_key_pairs(&self) -> Result<(), CertificateVerificationError> {
        #[cfg(feature = "disclosure")]
        {
            tracing::debug!("verifying verifier.usecases certificates");
            self.verify_verifier_key_pairs()?;
        }

        #[cfg(feature = "issuance")]
        {
            tracing::debug!("verifying issuer.private_keys certificates");
            self.verify_issuer_key_pairs()?;
        }

        Ok(())
    }

    #[cfg(feature = "disclosure")]
    pub fn verify_verifier_key_pairs<'a>(&'a self) -> Result<(), CertificateVerificationError> {
        let time = TimeGenerator;

        let trust_anchors: Vec<TrustAnchor<'a>> = self
            .reader_trust_anchors
            .iter()
            .map(BorrowingTrustAnchor::to_owned_trust_anchor)
            .collect::<Vec<_>>();

        let key_pairs: Vec<(String, KeyPair)> = self
            .verifier
            .usecases
            .as_ref()
            .iter()
            .map(|(use_case_id, usecase)| (use_case_id.clone(), usecase.key_pair.clone()))
            .collect();

        verify_key_pairs(
            &key_pairs,
            &trust_anchors,
            CertificateUsage::ReaderAuth,
            &time,
            |certificate_type| matches!(certificate_type, CertificateType::ReaderAuth(Some(_))),
        )
    }

    #[cfg(feature = "issuance")]
    pub fn verify_issuer_key_pairs<'a>(&'a self) -> Result<(), CertificateVerificationError> {
        let time = TimeGenerator;

        let trust_anchors: Vec<TrustAnchor<'a>> = self
            .issuer_trust_anchors
            .iter()
            .map(BorrowingTrustAnchor::to_owned_trust_anchor)
            .collect::<Vec<_>>();

        let key_pairs: Vec<(String, KeyPair)> = self
            .issuer
            .private_keys
            .iter()
            .map(|(id, keypair)| (id.clone(), keypair.clone()))
            .collect();

        verify_key_pairs(
            &key_pairs,
            &trust_anchors,
            CertificateUsage::Mdl,
            &time,
            |certificate_type| matches!(certificate_type, CertificateType::Mdl(Some(_))),
        )
    }
}

fn verify_key_pairs<F>(
    key_pairs: &[(String, KeyPair)],
    trust_anchors: &[TrustAnchor<'_>],
    usage: CertificateUsage,
    time: &impl Generator<DateTime<Utc>>,
    has_usage_registration: F,
) -> Result<(), CertificateVerificationError>
where
    F: Fn(CertificateType) -> bool,
{
    if trust_anchors.is_empty() {
        return Err(CertificateVerificationError::MissingTrustAnchors);
    }

    for (key_pair_id, key_pair) in key_pairs {
        tracing::debug!("verifying certificate of {key_pair_id}");

        if !trust_anchors.is_empty() {
            key_pair
                .certificate
                .verify(usage, &[], time, trust_anchors)
                .map_err(|e| CertificateVerificationError::InvalidCertificate(e, key_pair_id.clone()))?;
        }

        let certificate_type = CertificateType::from_certificate(&key_pair.certificate)
            .map_err(|e| CertificateVerificationError::NoCertificateType(e, key_pair_id.clone()))?;

        if !has_usage_registration(certificate_type) {
            return Err(CertificateVerificationError::IncompleteCertificateType(
                key_pair_id.clone(),
            ));
        }
    }

    Ok(())
}
