//! Holder software to store and disclose mdocs.

use url::Url;

pub use webpki::TrustAnchor;

use crate::{
    errors::Error,
    iso::*,
    utils::{
        reader_auth,
        serialization::CborError,
        x509::{Certificate, CertificateError},
    },
};

pub mod disclosure;
pub use disclosure::*;

pub mod http_client;
pub use http_client::*;

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
