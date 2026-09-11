use std::fmt::Debug;
use std::net::IpAddr;
use std::num::NonZeroU64;
use std::time::Duration;

use attestation_data::registration_certificate::BoundRegistrationCertificate;
use attestation_data::registration_certificate::RegistrationCertificateAuthorizationError;
use attestation_data::registration_certificate::RegistrationCertificateEnvelope;
use attestation_data::registration_certificate::verify_registration_certificate_envelope;
use attestation_data::x509::CertificateType;
use attestation_data::x509::CertificateTypeError;
use attestation_data::x509::RelyingParty;
use chrono::DateTime;
use chrono::Utc;
use config::ConfigError;
use crypto::p256_der::DerSigningKey;
use crypto::server_keys::KeyPair as ParsedKeyPair;
use crypto::trust_anchor::TrustAnchors;
use crypto::x509::BorrowingCertificate;
use crypto::x509::CertificateError;
use crypto::x509::CertificateUsage;
use dcql::Query;
use hsm::service::Pkcs11Hsm;
use hsm::settings::Hsm;
use nutype::nutype;
use openid4vc::server_state::SessionStoreTimeouts;
use serde::Deserialize;
use serde_with::base64::Base64;
use serde_with::hex::Hex;
use serde_with::serde_as;
use url::Url;
use utils::generator::Generator;

use crate::keys::PrivateKeySettingsError;
use crate::keys::PrivateKeyVariant;

/// Used as the `iss` field by the wallet in various JWTs.
pub const NL_WALLET_CLIENT_ID: &str = "https://wallet.edi.rijksoverheid.nl";

/// Settings shared by all variants of issuer/verifier servers.
#[derive(Clone, Deserialize)]
pub struct Settings {
    // used by the wallet, MUST be reachable from the public internet.
    pub wallet_server: Server,

    // used by the application, SHOULD be reachable only by the application.
    // if not configured the wallet_server will be used, but an api_key is required in that case
    // if it conflicts with wallet_server, the application will crash on startup
    pub internal_server: ServerAuth,

    pub log_requests: bool,
    pub structured_logging: bool,

    pub storage: Storage,

    /// Issuer trust anchors are used to validate the keys and certificates in the issuer's private_keys configuration
    /// on application startup and the issuer of the disclosed attributes during disclosure sessions.
    pub issuer_trust_anchors: TrustAnchors,

    /// Trust anchors for Wallet Relying Party Access Certificates, used by both issuers and verifiers.
    pub wrpac_trust_anchors: TrustAnchors,

    /// Trust anchors for Wallet Relying Party Registration Certificates, used by both issuers and verifiers.
    pub wrprc_trust_anchors: TrustAnchors,

    /// Optional HSM settings in which private keys can be stored
    pub hsm: Option<Hsm>,
}

#[derive(Clone, Deserialize)]
pub struct Server {
    pub ip: IpAddr,
    pub port: u16,
}

#[derive(Clone, Deserialize)]
pub enum ServerAuth {
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

pub struct VerifierUseCase<'a> {
    pub id: &'a str,
    pub key_pair: &'a KeyPair,
    pub registration_certificate: &'a RegistrationCertificateEnvelope,
    pub dcql_query: Option<&'a Query>,
}

#[derive(Debug, thiserror::Error)]
pub enum VerifierUseCasesValidationError {
    #[error("{0}")]
    Certificate(#[source] CertificateVerificationError),
    #[error("invalid registration certificate for use case `{use_case_id}`: {source}")]
    InvalidRegistrationCertificate {
        use_case_id: String,
        #[source]
        source: anyhow::Error,
    },
    #[error("DCQL query for use case `{use_case_id}` is not authorized by its registration certificate: {source}")]
    UnauthorizedDcqlQuery {
        use_case_id: String,
        #[source]
        source: RegistrationCertificateAuthorizationError,
    },
}

pub trait ServerSettings: Sized {
    type ValidationError: std::error::Error + Send + Sync + 'static;

    fn new(config_file: &str, env_prefix: &str) -> Result<Self, ConfigError>;
    fn validate(&self) -> Result<(), Self::ValidationError>;
    fn server_settings(&self) -> &Settings;
}

pub fn verify_key_pairs(
    key_pairs: &[(&str, &KeyPair)],
    trust_anchors: &TrustAnchors,
    usage: Option<CertificateUsage>,
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

        if let Some(usage) = usage
            && CertificateType::has_certificate_type(usage)
        {
            CertificateType::from_certificate(&key_pair.certificate)
                .map_err(|e| CertificateVerificationError::NoCertificateType(e, key_pair_id.to_string()))?;
        }
    }

    Ok(())
}

pub fn validate_verifier_use_cases(
    use_cases: &[VerifierUseCase<'_>],
    wrpac_trust_anchors: &TrustAnchors,
    wrprc_trust_anchors: &TrustAnchors,
    time: &impl Generator<DateTime<Utc>>,
) -> Result<(), VerifierUseCasesValidationError> {
    let key_pairs = use_cases
        .iter()
        .map(|use_case| (use_case.id, use_case.key_pair))
        .collect::<Vec<_>>();

    verify_key_pairs(&key_pairs, wrpac_trust_anchors, None, time)
        .map_err(VerifierUseCasesValidationError::Certificate)?;

    for use_case in use_cases {
        let registration_certificate = validate_registration_certificate(
            use_case.registration_certificate,
            &use_case.key_pair.certificate,
            wrprc_trust_anchors,
            time,
        )
        .map_err(
            |source| VerifierUseCasesValidationError::InvalidRegistrationCertificate {
                use_case_id: use_case.id.to_string(),
                source,
            },
        )?;

        if let Some(dcql_query) = use_case.dcql_query {
            registration_certificate
                .validate_query_authorization(dcql_query)
                .map_err(|source| VerifierUseCasesValidationError::UnauthorizedDcqlQuery {
                    use_case_id: use_case.id.to_string(),
                    source,
                })?;
        }
    }

    Ok(())
}

fn validate_registration_certificate(
    registration_certificate: &RegistrationCertificateEnvelope,
    access_certificate: &BorrowingCertificate,
    trust_anchors: &TrustAnchors,
    time: &impl Generator<DateTime<Utc>>,
) -> Result<BoundRegistrationCertificate, anyhow::Error> {
    let access_subject = RelyingParty::try_from(access_certificate.to_distinguished_name()?)?;

    let payload =
        verify_registration_certificate_envelope(registration_certificate, trust_anchors, time)?.into_payload();

    payload
        .validate_binding_and_time(&access_subject, time.generate())
        .map_err(anyhow::Error::from)
}
