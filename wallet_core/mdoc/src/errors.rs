use crate::{
    cose::CoseError, crypto::CryptoError, holder::HolderError, issuer_shared::IssuanceError,
    verifier::VerificationError,
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("cryptographic error: {0}")]
    CryptoError(#[from] CryptoError),
    #[error("COSE error: {0}")]
    CoseError(#[from] CoseError),
    #[error("holder error: {0}")]
    HolderError(#[from] HolderError),
    #[error("issuance error: {0}")]
    IssuanceError(#[from] IssuanceError),
    #[error("verification error: {0}")]
    VerificationError(#[from] VerificationError),
}
