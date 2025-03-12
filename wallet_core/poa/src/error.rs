use jsonwebtoken::jwk;

use wallet_common::jwt::JwkConversionError;
use wallet_common::jwt::JwtError;

use crate::POA_JWT_TYP;

#[derive(Debug, thiserror::Error)]
pub enum PoaError {
    #[error("error converting key from/to JWK: {0}")]
    Jwk(#[from] JwkConversionError),
    #[error("JWT bulk signing error: {0}")]
    Signing(#[from] JwtError),
    #[error("error obtaining verifying key from signing key: {0}")]
    VerifyingKey(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
}

#[derive(Debug, thiserror::Error)]
pub enum PoaVerificationError {
    #[error("JWT verification error: {0}")]
    Jwt(#[from] JwtError),
    #[error("unexpected amount of signatures in PoA: expected {expected}, found {found}")]
    UnexpectedSignatureCount { expected: usize, found: usize },
    #[error("unexpected amount of keys in PoA: expected {expected}, found {found}")]
    UnexpectedKeyCount { expected: usize, found: usize },
    #[error("incorrect nonce")]
    IncorrectNonce,
    #[error("error converting key from/to JWK: {0}")]
    Jwk(#[from] JwkConversionError),
    #[error("typ field of PoA header had unexpected value: expected 'Some({POA_JWT_TYP})', found '{0:?}'")]
    IncorrectTyp(Option<String>),
    #[error("key missing in PoA: {0:?}")]
    MissingKey(jwk::AlgorithmParameters),
}
