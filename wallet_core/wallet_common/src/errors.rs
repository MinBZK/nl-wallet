use crate::account::signed::SignedType;
use p256::pkcs8;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
#[error("message validation failed: {0}")]
pub enum ValidationError {
    Jwt(#[from] jsonwebtoken::errors::Error),
    P256Ecdsa(#[from] p256::ecdsa::Error),
    Ecdsa(#[source] Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, thiserror::Error)]
#[error("message signing failed")] // Do not format original error to prevent potentially leaking key material
pub enum SigningError {
    Jwt(#[from] jsonwebtoken::errors::Error),
    P256Ecdsa(#[from] p256::ecdsa::Error),
    Ecdsa(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("key deserialization error: {0}")]
    KeyDeserialization(#[from] pkcs8::Error),
    #[error("incorrect signing type (expected: {expected:?}, received: {received:?})")]
    TypeMismatch { expected: SignedType, received: SignedType },
    #[error("challenge does not match")]
    ChallengeMismatch,
    #[error("sequence number does not match")]
    SequenceNumberMismatch,
    #[error("JSON parsing error: {0}")]
    JsonParsing(#[from] serde_json::Error),
    #[error(transparent)]
    Validation(#[from] ValidationError),
    #[error(transparent)]
    Signing(#[from] SigningError),
}
