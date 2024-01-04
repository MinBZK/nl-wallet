use crate::{
    holder::HolderError,
    issuer_shared::IssuanceError,
    server_keys::KeysError,
    utils::{cose::CoseError, crypto::CryptoError, serialization::CborError},
    verifier::VerificationError,
};

pub type Result<T> = std::result::Result<T, Error>;

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

pub type DisclosureResult<T> = std::result::Result<T, DisclosureError>;

#[derive(thiserror::Error, Debug)]
#[error("error during disclosure, data_shared: {data_shared}, error: {error}")]
pub struct DisclosureError {
    pub data_shared: bool,
    #[source]
    pub error: Error,
}

impl DisclosureError {
    pub fn new(data_shared: bool, error: Error) -> Self {
        Self { data_shared, error }
    }

    pub fn before_sharing(error: Error) -> Self {
        Self {
            data_shared: false,
            error,
        }
    }

    pub fn after_sharing(error: Error) -> Self {
        Self {
            data_shared: true,
            error,
        }
    }
}
