use error_category::{Category, ErrorCategory};

use p256::pkcs8;

use crate::{account::signed::SignedType, jwt::JwtError};

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

impl ErrorCategory for Error {
    fn category(&self) -> Category {
        match self {
            Error::KeyDeserialization(_) => Category::Critical,
            Error::TypeMismatch { .. } => Category::Critical,
            Error::ChallengeMismatch => Category::Critical,
            Error::SequenceNumberMismatch => Category::Critical,
            Error::JsonParsing(_) => Category::PersonalData,
            Error::Ecdsa(_) => Category::Critical,
            Error::VerifyingKey(_) => Category::PersonalData,
            Error::Signing(_) => Category::PersonalData,
            Error::Jwt(error) => error.category(),
        }
    }
}
