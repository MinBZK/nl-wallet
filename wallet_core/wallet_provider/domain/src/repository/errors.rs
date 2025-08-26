use std::error::Error;

use strum::ParseError;

#[derive(Debug, thiserror::Error)]
pub enum PersistenceError {
    #[error("connection error: {0}")]
    Connection(#[source] Box<dyn Error + Send + Sync>),
    #[error("transaction error: {0}")]
    Transaction(#[source] Box<dyn Error + Send + Sync>),
    #[error("execution error: {0}")]
    Execution(#[source] Box<dyn Error + Send + Sync>),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("verifying key conversion error: {0}")]
    VerifyingKeyConversion(#[from] p256::pkcs8::spki::Error),
    #[error("signing key conversion error: {0}")]
    SigningKeyConversion(#[from] p256::ecdsa::Error),
    #[error("user state conversion error: {0}")]
    UserStateConversion(#[from] ParseError),
}
