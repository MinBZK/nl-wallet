use crate::{account::signed::SignedType, jwt::JwtError};
use p256::pkcs8;

pub type Result<T> = std::result::Result<T, Error>;

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
    #[error("signing error: {0}")]
    Ecdsa(#[from] p256::ecdsa::Error),
    #[error("verifying key error: {0}")]
    VerifyingKey(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("message signing failed")] // Do not format original error to prevent potentially leaking key material
    Signing(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error(transparent)]
    Jwt(#[from] JwtError),
}
