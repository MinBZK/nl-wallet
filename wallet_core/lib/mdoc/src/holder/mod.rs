//! Holder software to store and disclose mdocs.

use attestation::auth::reader_auth;
use crypto::x509::BorrowingCertificate;
use crypto::x509::CertificateError;
use error_category::ErrorCategory;
use sd_jwt_vc_metadata::TypeMetadataChainError;

pub mod disclosure;
pub use disclosure::*;

pub mod mdocs;
pub use mdocs::*;

#[derive(thiserror::Error, Debug, ErrorCategory)]
#[category(defer)]
pub enum HolderError {
    #[error("readerAuth not present for all documents")]
    #[category(critical)]
    ReaderAuthMissing,
    #[error("document requests were signed by different readers")]
    #[category(critical)]
    ReaderAuthsInconsistent,
    #[error("certificate error: {0}")]
    CertificateError(#[from] CertificateError),
    #[error("no reader registration present in certificate")]
    #[category(critical)]
    NoReaderRegistration(Box<BorrowingCertificate>),
    #[error("reader registration attribute validation failed: {0}")]
    ReaderRegistrationValidation(#[from] reader_auth::ValidationError),
    #[error("could not retrieve docs from source: {0}")]
    #[category(critical)]
    MdocDataSource(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("mdoc is missing type metadata integrity digest")]
    #[category(critical)]
    MissingMetadataIntegrity,
    #[error("could not decode type metadata chain: {0}")]
    #[category(critical)]
    TypeMetadata(#[from] TypeMetadataChainError),
}
