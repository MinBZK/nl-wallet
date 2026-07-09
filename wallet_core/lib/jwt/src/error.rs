use crypto::x509::CertificateError;
use error_category::ErrorCategory;
use jsonwebtoken::jwk::EllipticCurve;
use p256::ecdsa::VerifyingKey;
use p256::ecdsa::signature;

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum JwtParseError {
    #[error("JSON parsing error: {0}")]
    #[category(pd)]
    JsonParsing(#[source] serde_json::Error),

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
    Base64Error(#[source] base64::DecodeError),

    #[error("JWT conversion error: {0}")]
    #[category(defer)]
    Jwk(#[source] JwkConversionError),

    #[error("cannot construct JSON-serialized JWT: received differing payloads: {0}, {1}")]
    #[category(pd)]
    DifferentPayloads(String, String),

    #[error("header conversion failed: {0}")]
    #[category(critical)]
    HeaderConversion(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),

    #[error("missing jwk field in JWT header")]
    #[category(critical)]
    MissingJwk,

    #[error("missing x5c field in JWT header")]
    #[category(critical)]
    MissingX5c,

    #[error("missing typ field in JWT header")]
    #[category(critical)]
    MissingTyp,

    #[error("missing kid field in JWT header")]
    #[category(critical)]
    MissingKid,

    #[error("error converting iat: {0}")]
    #[category(critical)]
    InvalidIat(#[source] jsonwebtoken::errors::Error),

    #[error("iat out of range: {0}")]
    #[category(critical)]
    IatOutOfRange(i64),

    #[error("missing X.509 certificate(s) in JWT header to validate JWT against")]
    #[category(critical)]
    MissingCertificates,
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum JwtVerifyError {
    #[error("JWt parsing error: {0}")]
    #[category(pd)]
    ParseError(#[source] JwtParseError),

    #[error("error validating JWT: {0}")]
    #[category(critical)]
    Validation(#[source] jsonwebtoken::errors::Error),

    #[error("JWK in JWT header does not match expected public key: expected {0:?}, found {1:?}")]
    #[category(critical)]
    IncorrectJwkPublicKey(Box<VerifyingKey>, Box<VerifyingKey>),

    #[error("key not found in JWK set: {0}")]
    #[category(critical)]
    KeyNotFound(String),

    #[error("unexpected type: expected {0}, found {1:?}")]
    #[category(critical)]
    UnexpectedTyp(String, Option<String>),

    #[error("error converting JWK: {0}")]
    #[category(critical)]
    JwkConversion(#[source] jsonwebtoken::errors::Error),
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum JwtSignError {
    #[error("JSON serializing error: {0}")]
    #[category(pd)]
    JsonSerializing(#[source] serde_json::Error),

    #[error("JWT conversion error: {0}")]
    #[category(defer)]
    Jwk(#[source] JwkConversionError),

    #[error("error signing JWT: {0}")]
    #[category(critical)]
    Signing(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
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
    Base64Error(#[source] base64::DecodeError),

    #[error("failed to construct verifying key: {0}")]
    VerifyingKeyConstruction(#[source] signature::Error),

    #[error("missing coordinate in conversion to P256 public key")]
    #[category(critical)]
    MissingCoordinate,

    #[error("failed to get public key: {0}")]
    VerifyingKeyFromPrivateKey(#[source] Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum JwtX5cVerifyError {
    #[error("error validating JWT: {0}")]
    JwtVerify(#[from] JwtVerifyError),

    #[error("error base64-decoding certificate: {0}")]
    #[category(critical)]
    CertificateBase64(#[source] base64::DecodeError),

    #[error("error parsing certificate: {0}")]
    CertificateParsing(#[source] CertificateError),

    #[error("error verifying certificate: {0}")]
    CertificateValidation(#[source] CertificateError),
}
