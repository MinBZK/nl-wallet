//! Holder software, containing a [`Wallet`] that can store, receive, and disclose mdocs.
//! See [`MdocRetriever`], [`Wallet::start_issuance()`], and [`Wallet::disclose()`] respectively.

use std::error::Error;

use url::Url;

use crate::{
    iso::*,
    utils::{
        reader_auth,
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
    #[error("readerAuth not present for all documents")]
    ReaderAuthMissing,
    #[error("document requests were signed by different readers")]
    ReaderAuthsInconsistent,
    #[error("no unsigned mdocs received from issuer")]
    NoUnsignedMdocs,
    #[error("certificate error: {0}")]
    CertificateError(#[from] CertificateError),
    #[error("request error: {0}")]
    RequestError(#[from] HttpClientError),
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
    #[error("return URL prefix in reader registration ({}) does not match return URL provided: {}", (.0).0, (.0).1)]
    ReturnUrlPrefix(Box<(Url, Url)>), // Box these URLs, otherwise the error type becomes too big
    #[error("could not retrieve docs from source: {0}")]
    MdocDataSource(#[source] Box<dyn Error + Send + Sync>),
    #[error("multiple candidates for disclosure is unsupported, found for doc types: {}", .0.join(", "))]
    MultipleCandidates(Vec<DocType>),
    #[error("verifier returned error in response to disclosure: {0:?}")]
    DisclosureResponse(SessionStatus),
}
