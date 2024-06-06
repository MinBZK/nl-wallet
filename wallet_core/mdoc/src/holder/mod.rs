//! Holder software to store and disclose mdocs.

pub use webpki::TrustAnchor;

use crate::{
    errors::Error,
    iso::*,
    utils::{
        reader_auth,
        serialization::CborError,
        x509::{Certificate, CertificateError},
    },
    verifier::SessionType,
};

pub mod disclosure;
pub use disclosure::*;

pub mod http_client;
pub use http_client::*;

pub mod mdocs;
pub use mdocs::*;

#[derive(thiserror::Error, Debug)]
pub enum HolderError {
    #[error("missing session_type query parameter in verifier URL")]
    MissingSessionType,
    #[error("malformed session_type query parameter in verifier URL: {0}")]
    MalformedSessionType(serde_urlencoded::de::Error),
    #[error("mismatch between session type and reader engagement source: {0} not allowed from {1}")]
    ReaderEnagementSourceMismatch(SessionType, DisclosureUriSource),
    #[error("readerAuth not present for all documents")]
    ReaderAuthMissing,
    #[error("document requests were signed by different readers")]
    ReaderAuthsInconsistent,
    #[error("certificate error: {0}")]
    CertificateError(#[from] CertificateError),
    #[error("request error: {0}")]
    RequestError(#[from] HttpClientError),
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
    MdocDataSource(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("multiple candidates for disclosure is unsupported, found for doc types: {}", .0.join(", "))]
    MultipleCandidates(Vec<DocType>),
    #[error("verifier returned error in response to disclosure: {0:?}")]
    DisclosureResponse(SessionStatus),
}

pub type DisclosureResult<T> = std::result::Result<T, DisclosureError>;

#[derive(thiserror::Error, Debug)]
#[error("could not perform actual disclosure, attributes were shared: {data_shared}, error: {error}")]
pub struct DisclosureError {
    pub data_shared: bool,
    #[source]
    pub error: Error,
}

impl DisclosureError {
    pub fn new(data_shared: bool, error: Error) -> Self {
        Self { data_shared, error }
    }

    pub fn before_sharing(error: Error) -> Self {
        Self {
            data_shared: false,
            error,
        }
    }

    pub fn after_sharing(error: Error) -> Self {
        Self {
            data_shared: true,
            error,
        }
    }
}

impl From<HttpClientError> for DisclosureError {
    fn from(source: HttpClientError) -> Self {
        let data_shared = match source {
            // Cbor serialization happens before sharing
            HttpClientError::Cbor(CborError::Serialization(_)) => false,
            // Cbor deserialization happens after sharing
            HttpClientError::Cbor(CborError::Deserialization(_)) => true,
            // When connection cannot be established, no data is shared
            HttpClientError::Request(ref reqwest_error) => !reqwest_error.is_connect(),
        };
        Self::new(data_shared, Error::Holder(HolderError::RequestError(source)))
    }
}
