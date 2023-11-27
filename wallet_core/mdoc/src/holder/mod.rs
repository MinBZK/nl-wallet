//! Holder software, containing a [`Wallet`] that can store, receive, and disclose mdocs.
//! See [`MdocRetriever`], [`Wallet::start_issuance()`], and [`Wallet::disclose()`] respectively.

use std::error::Error;

use crate::{
    identifiers::AttributeIdentifier,
    iso::*,
    utils::{
        reader_auth::{self, ReaderRegistration},
        x509::{Certificate, CertificateError},
    },
};

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
    VerifierUrlMissing,
    #[error("verifier ephemeral key not present in reader engagement")]
    VerifierEphemeralKeyMissing,
    #[error("no document requests are present in device request")]
    NoAttributesRequested,
    #[error("no reader registration present in certificate")]
    NoReaderRegistration(Certificate),
    #[error("reader registration attribute validation failed: {0}")]
    ReaderRegistrationValidation(#[from] reader_auth::ValidationError),
    #[error("could not retrieve docs from source: {0}")]
    MdocDataSource(#[source] Box<dyn Error + Send + Sync>),
    #[error("multiple candidates for disclosure is unsupported, found for doc types: {}", .0.join(", "))]
    MultipleCandidates(Vec<DocType>),
    #[error("not all requested attributes are available, missing: {missing_attributes:?}")]
    AttributesNotAvailable {
        reader_registration: Box<ReaderRegistration>,
        missing_attributes: Vec<AttributeIdentifier>,
    },
}
