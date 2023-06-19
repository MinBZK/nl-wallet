//! Holder software, containing a [`Wallet`] that can store, receive, and disclose mdocs.
//! See [`Storage`], [`Wallet::do_issuance()`], and [`Wallet::disclose()`] respectively.

use x509_parser::prelude::X509Error;

use crate::iso::*;

pub mod disclosure;
pub mod issuance;
pub mod mdocs;

pub use disclosure::*;
pub use issuance::*;
pub use mdocs::*;

#[derive(thiserror::Error, Debug)]
pub enum HolderError {
    #[error("unsatisfiable request: DocType {0} not in wallet")]
    UnsatisfiableRequest(DocType),
    #[error("readerAuth not present for all documents")]
    ReaderAuthMissing,
    #[error("document requests were signed by different readers")]
    ReaderAuthsInconsistent,
    #[error("issuer not trusted for doctype {0}")]
    UntrustedIssuer(DocType),
    #[error("failed to parse certificate: {0}")]
    CertificateParsingFailed(#[from] x509_parser::nom::Err<X509Error>),
    #[error("wrong private key type")]
    PrivateKeyTypeMismatch { expected: String, have: String },
}
