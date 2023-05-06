use crate::{
    cose::CoseError, crypto::CryptoError, holder::HolderError, issuer_shared::IssuanceError,
    verifier::VerificationError,
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("cryptographic error: {0}")]
    Crypto(#[from] CryptoError),
    #[error("COSE error: {0}")]
    Cose(#[from] CoseError),
    #[error("holder error: {0}")]
    Holder(#[from] HolderError),
    #[error("issuance error: {0}")]
    Issuance(#[from] IssuanceError),
    #[error("verification error: {0}")]
    Verification(#[from] VerificationError),
}
