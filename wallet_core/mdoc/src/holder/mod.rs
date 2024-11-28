//! Holder software to store and disclose mdocs.

use error_category::ErrorCategory;
pub use webpki::types::TrustAnchor;

use crate::utils::reader_auth;
use crate::utils::x509::BorrowingCertificate;
use crate::utils::x509::CertificateError;

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
}
