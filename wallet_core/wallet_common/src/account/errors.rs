use super::{signed::SignedType, signing_key::EcdsaKeyError};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
#[error("Message validation failed: {0}")]
pub enum ValidationError {
    Jwt(#[from] jsonwebtoken::errors::Error),
    P256Ecdsa(#[from] p256::ecdsa::Error),
    Ecdsa(#[from] EcdsaKeyError),
}

#[derive(Debug, thiserror::Error)]
#[error("Message signing failed")] // Do not format original error to prevent potentially leaking key material
pub enum SigningError {
    Jwt(#[from] jsonwebtoken::errors::Error),
    P256Ecdsa(#[from] p256::ecdsa::Error),
    Ecdsa(#[from] EcdsaKeyError), // not actually used
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incorrect signing type (expected: {expected:?}, received: {received:?})")]
    TypeMismatch { expected: SignedType, received: SignedType },
    #[error("Challenge does not match")]
    ChallengeMismatch,
    #[error("JSON parsing error: {0}")]
    JsonParsing(#[from] serde_json::Error),
    #[error(transparent)]
    Validation(#[from] ValidationError),
    #[error(transparent)]
    Signing(#[from] SigningError),
}
