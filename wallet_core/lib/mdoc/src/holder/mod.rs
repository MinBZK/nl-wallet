//! Holder software to store and disclose mdocs.

use crypto::x509::CertificateError;
use error_category::ErrorCategory;

pub mod disclosure;
pub use disclosure::*;

pub mod mdocs;
pub use mdocs::*;

#[derive(thiserror::Error, Debug, ErrorCategory)]
#[category(defer)]
pub enum HolderError {
    #[error("certificate error: {0}")]
    CertificateError(#[from] CertificateError),
    #[error("could not retrieve docs from source: {0}")]
    #[category(critical)]
    MdocDataSource(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("mdoc is missing type metadata integrity digest")]
    #[category(critical)]
    MissingMetadataIntegrity,
}
