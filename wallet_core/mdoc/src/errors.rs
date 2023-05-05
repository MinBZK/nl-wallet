use crate::{
    cose::CoseError, crypto::CryptoError, holder::HolderError, issuer_shared::IssuanceError,
    verifier::VerificationError,
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("cryptographic error")]
    CryptoError(#[from] CryptoError),
    #[error("COSE error")]
    CoseError(#[from] CoseError),
    #[error("holder error")]
    HolderError(#[from] HolderError),
    #[error("issuance error")]
    IssuanceError(#[from] IssuanceError),
    #[error("verification error")]
    VerificationError(#[from] VerificationError),
}
