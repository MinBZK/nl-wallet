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

use nl_wallet_mdoc::server_keys::KeyPair as ParsedKeyPair;
use nl_wallet_mdoc::utils::x509::BorrowingCertificate;
use nl_wallet_mdoc::utils::x509::CertificateError;
use nl_wallet_mdoc::utils::x509::CertificateType;
use nl_wallet_mdoc::utils::x509::CertificateUsage;
use openid4vc::server_state::SessionStoreTimeouts;
use wallet_common::generator::Generator;
use wallet_common::generator::TimeGenerator;
use wallet_common::p256_der::DerSigningKey;
use wallet_common::trust_anchor::BorrowingTrustAnchor;
use wallet_common::urls::BaseUrl;
use wallet_common::utils;

cfg_if::cfg_if! {
    if #[cfg(feature = "disclosure")] {
        mod disclosure;
        pub use disclosure::*;
        use wallet_common::urls::DEFAULT_UNIVERSAL_LINK_BASE;
    }
}

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct Settings {
    // used by the wallet, MUST be reachable from the public internet.
    pub wallet_server: Server,

    /// Publically reachable URL used by the wallet during sessions
    pub public_url: BaseUrl,

    pub log_requests: bool,
    pub structured_logging: bool,

    pub storage: Storage,

    /// Issuer trust anchors are used to validate the keys and certificates in the `issuer.private_keys` configuration
    /// on application startup and the issuer of the disclosed attributes during disclosure sessions.
    #[serde_as(as = "Vec<Base64>")]
    pub issuer_trust_anchors: Vec<BorrowingTrustAnchor>,
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
    #[serde_as(as = "Base64")]
    pub private_key: DerSigningKey,
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

impl TryFrom<KeyPair> for ParsedKeyPair {
    type Error = CertificateError;

    fn try_from(value: KeyPair) -> Result<Self, Self::Error> {
        ParsedKeyPair::new_from_signing_key(value.private_key.into_inner(), value.certificate)
    }
}

#[cfg(feature = "integration_test")]
impl From<ParsedKeyPair> for KeyPair {
    fn from(value: ParsedKeyPair) -> Self {
        Self {
            certificate: value.certificate().clone(),
            private_key: DerSigningKey::from(value.private_key().clone()),
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

pub trait ServerSettings: Sized {
    fn new_custom(config_file: &str, env_prefix: &str) -> Result<Self, ConfigError>;
    fn verify_key_pairs(&self) -> Result<(), CertificateVerificationError>;
    fn structured_logging(&self) -> bool;
}

pub fn verify_key_pairs<F>(
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

        let key_pair: ParsedKeyPair = key_pair
            .clone()
            .try_into()
            .map_err(|error| CertificateVerificationError::InvalidKeyPair(error, key_pair_id.clone()))?;

        let certificate = key_pair.certificate();

        if !trust_anchors.is_empty() {
            certificate
                .verify(usage, &[], time, trust_anchors)
                .map_err(|e| CertificateVerificationError::InvalidCertificate(e, key_pair_id.clone()))?;
        }

        let certificate_type = CertificateType::from_certificate(certificate)
            .map_err(|e| CertificateVerificationError::NoCertificateType(e, key_pair_id.clone()))?;

        if !has_usage_registration(certificate_type) {
            return Err(CertificateVerificationError::IncompleteCertificateType(
                key_pair_id.clone(),
            ));
        }
    }

    Ok(())
}
