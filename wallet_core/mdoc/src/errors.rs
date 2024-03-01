use crate::{
    holder::HolderError,
    issuer_shared::IssuanceError,
    server_keys::KeysError,
    utils::{cose::CoseError, crypto::CryptoError, serialization::CborError},
    verifier::VerificationError,
};

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("cryptographic error: {0}")]
    Crypto(#[from] CryptoError),
    #[error("COSE error: {0}")]
    Cose(#[from] CoseError),
    #[error("CBOR error: {0}")]
    Cbor(#[from] CborError),
    #[error("holder error: {0}")]
    Holder(#[from] HolderError),
    #[error("issuance error: {0}")]
    Issuance(#[from] IssuanceError),
    #[error("verification error: {0}")]
    Verification(#[from] VerificationError),
    #[error("keys error: {0}")]
    KeysError(#[from] KeysError),
}
