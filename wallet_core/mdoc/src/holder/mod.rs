//! Holder software, containing a [`Wallet`] that can store, receive, and disclose mdocs.
//! See [`Storage`], [`Wallet::start_issuance()`], and [`Wallet::disclose()`] respectively.

use crate::{iso::*, utils::x509::CertificateError};

pub mod disclosure;
pub use disclosure::*;

pub mod issuance;
pub use issuance::*;

pub mod mdocs;
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
    #[error("certificate error: {0}")]
    CertificateError(#[from] CertificateError),
    #[error("wrong private key type")]
    PrivateKeyTypeMismatch { expected: String, have: String },
    #[error("request error: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("malformed Service Engagement: url missing")]
    MalformedServiceEngagement,
    #[error("malformed attribute: random too short (was {0}; minimum {1}")]
    AttributeRandomLength(usize, usize),
    #[error("missing issuance session state")]
    MissingIssuanceSessionState,
    #[error("verifier URL not present in reader engagement")]
    VerifiedUrlMissing,
    #[error("verifier ephemeral key not present in reader engagement")]
    VerifierEphemeralKeyMissing,
    #[error("no document requests are present in device request")]
    NoDocumentRequests,
}
