#[cfg(feature = "server")]
use crate::signed::SignatureType;

#[cfg(feature = "client")]
#[derive(Debug, thiserror::Error, error_category::ErrorCategory)]
#[category(pd)]
pub enum EncodeError {
    #[error("could not encode JSON: {0}")]
    Json(#[source] serde_json::Error),
    #[error("could not sign message: {0}")]
    Signing(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("could not get verifying key: {0}")]
    VerifyingKey(#[source] Box<dyn std::error::Error + Send + Sync>),
}

#[cfg(feature = "server")]
#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("could not decode JSON: {0}")]
    Json(#[source] serde_json::Error),
    #[error("incorrect signing type (expected: {expected:?}, received: {received:?})")]
    SignatureTypeMismatch {
        expected: SignatureType,
        received: SignatureType,
    },
    #[error("message signature did not verify: {0}")]
    Signature(#[source] p256::ecdsa::Error),
    #[error("message assertion did not verify: {0}")]
    Assertion(#[source] apple_app_attest::AssertionError),
    #[error("incorrect signing subject (expected: {expected}, received: {received})")]
    SubjectMismatch { expected: String, received: String },
    #[error("incorrect wallet id")]
    WalletIdMismatch,
    #[error("incorrect sequence number")]
    SequenceNumberMismatch,
    #[error("incorrect challenge")]
    ChallengeMismatch,
}
