use std::fmt::Debug;
use std::net::IpAddr;
use std::num::NonZeroU64;
use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use config::ConfigError;
use nutype::nutype;
use rustls_pki_types::TrustAnchor;
use serde::Deserialize;
use serde_with::base64::Base64;
use serde_with::hex::Hex;
use serde_with::serde_as;
use url::Url;

use attestation_data::x509::CertificateType;
use attestation_data::x509::CertificateTypeError;
use crypto::p256_der::DerSigningKey;
use crypto::server_keys::KeyPair as ParsedKeyPair;
use crypto::trust_anchor::BorrowingTrustAnchor;
use crypto::x509::BorrowingCertificate;
use crypto::x509::CertificateError;
use crypto::x509::CertificateUsage;
use hsm::service::Pkcs11Hsm;
use hsm::settings::Hsm;
use http_utils::urls::BaseUrl;
use openid4vc::server_state::SessionStoreTimeouts;
use utils::generator::Generator;

use crate::keys::PrivateKeySettingsError;
use crate::keys::PrivateKeyVariant;

/// Used as the `iss` field by the wallet in various JWTs.
pub const NL_WALLET_CLIENT_ID: &str = "https://wallet.edi.rijksoverheid.nl";

/// Settings shared by all variants of issuer/verifier servers.
#[serde_as]
#[derive(Clone, Deserialize)]
pub struct Settings {
    // used by the wallet, MUST be reachable from the public internet.
    pub wallet_server: Server,

    /// Publicly reachable URL used by the wallet during sessions
    pub public_url: BaseUrl,

    pub log_requests: bool,
    pub structured_logging: bool,

    pub storage: Storage,

    /// Issuer trust anchors are used to validate the keys and certificates in the issuer's private_keys configuration
    /// on application startup and the issuer of the disclosed attributes during disclosure sessions.
    #[serde_as(as = "Vec<Base64>")]
    pub issuer_trust_anchors: Vec<BorrowingTrustAnchor>,

    /// Optional HSM settings in which private keys can be stored
    pub hsm: Option<Hsm>,
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

    #[serde(flatten)]
    pub private_key: PrivateKey,
}

/// An ECDSA private (i.e. asymmetric) key, either in the HSM or configured directly.
#[serde_as]
#[derive(Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "private_key_type")]
pub enum PrivateKey {
    Software {
        #[serde_as(as = "Base64")]
        private_key: DerSigningKey,
    },
    Hsm {
        private_key: String,
    },
}

/// A secret (i.e. symmetric) key, e.g. for HMAC, either in the HSM or configured directly.
#[serde_as]
#[derive(Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "secret_key_type")]
pub enum SecretKey {
    Software {
        #[serde_as(as = "Hex")]
        secret_key: SecretKeyBytes,
    },
    Hsm {
        secret_key: String,
    },
}

const MIN_SECRET_KEY_LENGTH_BYTES: usize = 32;

#[nutype(validate(predicate = |v| v.len() >= MIN_SECRET_KEY_LENGTH_BYTES), derive(Clone, TryFrom, AsRef, Deserialize))]
pub struct SecretKeyBytes(Vec<u8>);

impl From<&Storage> for SessionStoreTimeouts {
    fn from(value: &Storage) -> Self {
        SessionStoreTimeouts {
            expiration: Duration::from_secs(60 * value.expiration_minutes.get()),
            successful_deletion: Duration::from_secs(60 * value.successful_deletion_minutes.get()),
            failed_deletion: Duration::from_secs(60 * value.failed_deletion_minutes.get()),
        }
    }
}

impl KeyPair {
    pub async fn parse(
        self,
        hsm: Option<Pkcs11Hsm>,
    ) -> Result<ParsedKeyPair<PrivateKeyVariant>, PrivateKeySettingsError> {
        let private_key = PrivateKeyVariant::from_settings(self.private_key, hsm)?;
        let key_pair = ParsedKeyPair::new(private_key, self.certificate).await?;
        Ok(key_pair)
    }
}

#[cfg(feature = "parsed_key_pair_conversion")]
impl From<ParsedKeyPair> for KeyPair {
    fn from(value: ParsedKeyPair) -> Self {
        Self {
            certificate: value.certificate().clone(),
            private_key: PrivateKey::Software {
                private_key: value.private_key().clone().into(),
            },
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
    NoCertificateType(#[source] CertificateTypeError, String),
}

pub trait ServerSettings: Sized {
    type ValidationError: std::error::Error + Send + Sync + 'static;

    fn new(config_file: &str, env_prefix: &str) -> Result<Self, ConfigError>;
    fn validate(&self) -> Result<(), Self::ValidationError>;
    fn server_settings(&self) -> &Settings;
}

pub fn verify_key_pairs(
    key_pairs: &[(&str, &KeyPair)],
    trust_anchors: &[TrustAnchor<'_>],
    usage: CertificateUsage,
    time: &impl Generator<DateTime<Utc>>,
) -> Result<(), CertificateVerificationError> {
    if trust_anchors.is_empty() {
        return Err(CertificateVerificationError::MissingTrustAnchors);
    }

    for (key_pair_id, key_pair) in key_pairs {
        tracing::debug!("verifying certificate of {key_pair_id}");

        key_pair
            .certificate
            .verify(usage, &[], time, trust_anchors)
            .map_err(|e| CertificateVerificationError::InvalidCertificate(e, key_pair_id.to_string()))?;

        if CertificateType::has_certificate_type(usage) {
            CertificateType::from_certificate(&key_pair.certificate)
                .map_err(|e| CertificateVerificationError::NoCertificateType(e, key_pair_id.to_string()))?;
        }
    }

    Ok(())
}
