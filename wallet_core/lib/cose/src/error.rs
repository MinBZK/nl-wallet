use coset::Label;
use crypto::x509::CertificateError;
use error_category::ErrorCategory;

use crate::serialization::CborError;

#[derive(thiserror::Error, Debug, ErrorCategory)]
#[category(defer)]
pub enum CoseError {
    #[error("missing payload")]
    #[category(critical)]
    MissingPayload,
    #[error("missing label {0:?}")]
    #[category(critical)]
    MissingLabel(Label),
    #[error("missing protected COSE algorithm")]
    #[category(critical)]
    MissingAlgorithm,
    #[error("unsupported protected COSE algorithm: {0:?}")]
    #[category(critical)]
    UnsupportedAlgorithm(coset::Algorithm),
    #[error("ECDSA signature parsing failed: {0}")]
    #[category(pd)]
    EcdsaSignatureParsingFailed(#[source] p256::ecdsa::Error),
    #[error("ECDSA signature verification failed: {0}")]
    #[category(pd)]
    EcdsaSignatureVerificationFailed(#[source] p256::ecdsa::Error),
    #[error("MAC verification failed")]
    #[category(critical)]
    MacVerificationFailed,
    #[error("CBOR error: {0}")]
    Cbor(#[source] CborError),
    #[error("signing certificate header did not contain bytes")]
    #[category(critical)]
    CertificateUnexpectedHeaderType,
    #[error("x5chain certificate chain is empty")]
    #[category(critical)]
    EmptyCertificateChain,
    #[error("x5chain certificate array must contain at least two certificates, found {0}")]
    #[category(critical)]
    CertificateChainTooShort(usize),
    #[error("certificate error: {0}")]
    Certificate(#[source] CertificateError),
    #[error("signing failed: {0}")]
    #[category(pd)]
    Signing(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("no signature received")]
    #[category(critical)]
    SignatureMissing,
}
