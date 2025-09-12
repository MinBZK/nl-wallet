use base64::DecodeError;
use jsonwebtoken::jwk::EllipticCurve;
use p256::ecdsa::signature;

use crypto::x509::CertificateError;
use error_category::ErrorCategory;

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum JwtError {
    #[error("JSON parsing error: {0}")]
    #[category(pd)]
    JsonParsing(#[from] serde_json::Error),

    #[error("error validating JWT: {0}")]
    #[category(critical)]
    Validation(#[source] jsonwebtoken::errors::Error),

    #[error("error signing JWT: {0}")]
    #[category(critical)]
    Signing(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),

    #[error("unexpected amount of parts in JWT credential: expected 3, found {0}")]
    #[category(critical)]
    UnexpectedNumberOfParts(usize),

    #[error("failed to decode Base64: {0}")]
    #[category(pd)]
    Base64Error(#[from] base64::DecodeError),

    #[error("JWT conversion error: {0}")]
    #[category(defer)]
    Jwk(#[from] JwkConversionError),

    #[error("cannot construct JSON-serialized JWT: received differing payloads: {0}, {1}")]
    #[category(pd)]
    DifferentPayloads(String, String),

    #[error("Header conversion failed: {0}")]
    #[category(critical)]
    HeaderConversion(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),

    #[error("unexpected type: expected {0}, found {1}")]
    #[category(critical)]
    UnexpectedTyp(String, String),

    #[error("missing jwk field in JWT header")]
    #[category(critical)]
    MissingJwk,

    #[error("missing x5c field in JWT header")]
    #[category(critical)]
    MissingX5c,

    #[error("missing typ field in JWT header")]
    #[category(critical)]
    MissingTyp,
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(pd)]
pub enum JwkConversionError {
    #[error("unsupported JWK EC curve: expected P256, found {found:?}")]
    #[category(critical)]
    UnsupportedJwkEcCurve { found: EllipticCurve },
    #[error("unsupported JWK algorithm")]
    #[category(critical)]
    UnsupportedJwkAlgorithm,
    #[error("base64 decoding failed: {0}")]
    Base64Error(#[from] base64::DecodeError),
    #[error("failed to construct verifying key: {0}")]
    VerifyingKeyConstruction(#[from] signature::Error),
    #[error("missing coordinate in conversion to P256 public key")]
    #[category(critical)]
    MissingCoordinate,
    #[error("failed to get public key: {0}")]
    VerifyingKeyFromPrivateKey(#[source] Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum JwtX5cError {
    #[error("error validating JWT: {0}")]
    Jwt(#[from] JwtError),

    #[error("missing X.509 certificate(s) in JWT header to validate JWT against")]
    #[category(critical)]
    MissingCertificates,

    #[error("error base64-decoding certificate: {0}")]
    #[category(critical)]
    CertificateBase64(#[source] DecodeError),

    #[error("error parsing certificate: {0}")]
    CertificateParsing(#[source] CertificateError),

    #[error("error verifying certificate: {0}")]
    CertificateValidation(#[source] CertificateError),
}
