use crate::{
    holder::HolderError,
    server_keys::KeysError,
    utils::{cose::CoseError, crypto::CryptoError, serialization::CborError, x509::CertificateError},
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
    #[error("verification error: {0}")]
    Verification(#[from] VerificationError),
    #[error("keys error: {0}")]
    KeysError(#[from] KeysError),
    #[error("certificate error: {0}")]
    CertificateError(#[from] CertificateError),
}
