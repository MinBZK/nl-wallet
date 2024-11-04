use error_category::ErrorCategory;

use crate::account::signed::SignedType;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(pd)]
pub enum Error {
    #[error("signed message did not verify: {0}")]
    Verification(#[source] p256::ecdsa::Error),
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
    JsonParsing(#[source] serde_json::Error),
    #[error("message signing failed")] // Do not format original error to prevent potentially leaking key material
    Signing(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("verifying key error: {0}")]
    VerifyingKey(#[source] Box<dyn std::error::Error + Send + Sync>),
}
