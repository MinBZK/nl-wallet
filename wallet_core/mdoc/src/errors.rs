use crate::{
    cose::CoseError, crypto::KeyError, holder::HolderError, issuer_shared::IssuanceError, verifier::VerificationError,
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("CBOR serialization failed")]
    Serialization(#[from] ciborium::ser::Error<std::io::Error>),
    #[error("ECDSA error")]
    EcdsaError(#[from] ecdsa::Error),

    #[error("HKDF failed")]
    Hkdf,
    #[error("malformed key")]
    MalformedKey(#[from] KeyError),
    #[error("COSE error")]
    CoseError(#[from] CoseError),
    #[error("holder error")]
    HolderError(#[from] HolderError),
    #[error("issuance error")]
    IssuanceError(#[from] IssuanceError),
    #[error("verification error")]
    VerificationError(#[from] VerificationError),
}
