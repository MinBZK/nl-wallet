use super::signed::SignedType;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incorrect signing type (expected: {expected:?}, received: {received:?})")]
    TypeMismatch { expected: SignedType, received: SignedType },
    #[error("Challenge does not match")]
    ChallengeMismatch,
    #[error("JSON parsing error: {0}")]
    JsonParsing(#[from] serde_json::Error),
    #[error("Message validation failed: {0}")]
    Validation(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("Message signing failed")] // Do not format original error to prevent potentially leaking key material
    Signing(#[source] Box<dyn std::error::Error + Send + Sync>),
}
