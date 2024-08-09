//! Holder software to store and disclose mdocs.

use error_category::ErrorCategory;
pub use webpki::TrustAnchor;

use crate::{
    iso::*,
    utils::{
        reader_auth,
        x509::{Certificate, CertificateError},
    },
    verifier::SessionType,
};

pub mod disclosure;
pub use disclosure::*;

pub mod mdocs;
pub use mdocs::*;

#[derive(thiserror::Error, Debug, ErrorCategory)]
#[category(defer)]
pub enum HolderError {
    #[error("missing session_type query parameter in verifier URL")]
    #[category(critical)]
    MissingSessionType,
    #[error("malformed session_type query parameter in verifier URL: {0}")]
    #[category(critical)]
    MalformedSessionType(serde_urlencoded::de::Error),
    #[error("mismatch between session type and disclosure URI source: {0} not allowed from {1}")]
    #[category(critical)]
    DisclosureUriSourceMismatch(SessionType, DisclosureUriSource),
    #[error("readerAuth not present for all documents")]
    #[category(critical)]
    ReaderAuthMissing,
    #[error("document requests were signed by different readers")]
    #[category(critical)]
    ReaderAuthsInconsistent,
    #[error("certificate error: {0}")]
    CertificateError(#[from] CertificateError),
    #[error("verifier URL not present in reader engagement")]
    #[category(critical)]
    VerifierUrlMissing,
    #[error("verifier ephemeral key not present in reader engagement")]
    #[category(critical)]
    VerifierEphemeralKeyMissing,
    #[error("no document requests are present in device request")]
    #[category(critical)]
    NoAttributesRequested,
    #[error("no reader registration present in certificate")]
    #[category(critical)]
    NoReaderRegistration(Certificate),
    #[error("reader registration attribute validation failed: {0}")]
    ReaderRegistrationValidation(#[from] reader_auth::ValidationError),
    #[error("could not retrieve docs from source: {0}")]
    #[category(critical)]
    MdocDataSource(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("multiple candidates for disclosure is unsupported, found for doc types: {}", .0.join(", "))]
    #[category(critical)]
    MultipleCandidates(Vec<DocType>),
    #[error("verifier returned error in response to disclosure: {0:?}")]
    #[category(critical)]
    DisclosureResponse(SessionStatus),
}
