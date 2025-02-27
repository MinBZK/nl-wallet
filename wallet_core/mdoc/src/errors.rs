use error_category::ErrorCategory;
use sd_jwt::metadata::TypeMetadataError;

use crate::holder::HolderError;
use crate::utils::cose::CoseError;
use crate::utils::cose::KeysError;
use crate::utils::crypto::CryptoError;
use crate::utils::serialization::CborError;
use crate::utils::x509::CertificateError;
use crate::verifier::VerificationError;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug, ErrorCategory)]
#[category(defer)]
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
    #[category(unexpected)]
    Verification(#[from] VerificationError),
    #[error("keys error: {0}")]
    KeysError(#[from] KeysError),
    #[error("certificate error: {0}")]
    CertificateError(#[from] CertificateError),
    #[error("type metadata error: {0}")]
    #[category(critical)]
    TypeMetadata(#[from] TypeMetadataError),
    #[error("missing issuer URI")]
    #[category(critical)]
    MissingIssuerUri,
}
