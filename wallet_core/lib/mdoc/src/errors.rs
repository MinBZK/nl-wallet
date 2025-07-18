use crypto::x509::CertificateError;
use error_category::ErrorCategory;

use crate::IssuerNameSpacesPreConditionError;
use crate::holder::HolderError;
use crate::identifiers::AttributeIdentifierError;
use crate::utils::cose::CoseError;
use crate::utils::cose::KeysError;
use crate::utils::crypto::CryptoError;
use crate::utils::serialization::CborError;
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

    #[error("Missing validity information: {0}")]
    #[category(critical)]
    MissingValidityInformation(String),

    #[error("Missing or empty NameSpace detected: {0}")]
    #[category(critical)]
    MissingOrEmptyNamespace(#[from] IssuerNameSpacesPreConditionError),

    #[error("Unable to extract attribute identifiers from items request: {0}")]
    #[category(critical)]
    AttributeIdentifier(#[from] AttributeIdentifierError),
}
