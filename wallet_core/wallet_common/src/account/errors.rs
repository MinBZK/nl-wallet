use p256::pkcs8;

use error_category::ErrorCategory;

use crate::{account::signed::SignedType, jwt::JwtError};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(pd)]
pub enum Error {
    #[error("key deserialization error: {0}")]
    #[category(critical)]
    KeyDeserialization(#[from] pkcs8::Error),
    #[error("incorrect signing type (expected: {expected:?}, received: {received:?})")]
    #[category(critical)]
    TypeMismatch { expected: SignedType, received: SignedType },
    #[error("incorrect signing subject (expected: {expected}, received: {received})")]
    #[category(critical)]
    SubjectMismatch { expected: String, received: String },
    #[error("challenge does not match")]
    #[category(critical)]
    ChallengeMismatch,
    #[error("sequence number does not match")]
    #[category(critical)]
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
    #[category(defer)]
    Jwt(#[from] JwtError),
}
