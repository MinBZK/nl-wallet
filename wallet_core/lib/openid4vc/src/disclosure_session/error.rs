use std::error::Error;

use derive_more::Constructor;

use attestation_data::auth::reader_auth::ValidationError;
use crypto::x509::CertificateError;
use error_category::ErrorCategory;

use crate::openid4vp::AuthRequestValidationError;
use crate::openid4vp::AuthResponseError;
use crate::verifier::SessionType;

use super::VpMessageClientError;
use super::uri_source::DisclosureUriSource;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum VpSessionError {
    #[error("{0}")]
    Client(#[from] VpClientError),

    #[error("{0}")]
    Verifier(#[from] VpVerifierError),
}

impl From<VpMessageClientError> for VpSessionError {
    fn from(source: VpMessageClientError) -> Self {
        match &source {
            VpMessageClientError::Json(_) => VpSessionError::Verifier(VpVerifierError::Request(source)),
            _ => VpSessionError::Client(VpClientError::Request(source)),
        }
    }
}

impl From<AuthRequestValidationError> for VpSessionError {
    fn from(source: AuthRequestValidationError) -> Self {
        VpSessionError::Verifier(VpVerifierError::AuthRequestValidation(source))
    }
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum VpClientError {
    #[error("error deserializing request_uri object: {0}")]
    // we cannot be sure that the URL is not included in the error.
    #[category(pd)]
    RequestUri(#[source] serde_urlencoded::de::Error),

    #[error("mismatch between session type and disclosure URI source: {0} not allowed from {1}")]
    #[category(critical)]
    DisclosureUriSourceMismatch(SessionType, DisclosureUriSource),

    #[error("error sending OpenID4VP message: {0}")]
    Request(#[source] VpMessageClientError),

    #[error("error creating mdoc device response: {0}")]
    DeviceResponse(#[source] mdoc::Error),

    #[error("error encrypting Authorization Response: {0}")]
    #[category(defer)]
    AuthResponseEncryption(#[source] AuthResponseError),
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum VpVerifierError {
    #[error("missing session_type query parameter in request URI")]
    #[category(critical)]
    MissingSessionType,

    #[error("malformed session_type query parameter in request URI: {0}")]
    // we cannot be sure that the URL is not included in the error
    #[category(pd)]
    MalformedSessionType(#[source] serde_urlencoded::de::Error),

    #[error("error sending OpenID4VP message: {0}")]
    Request(#[source] VpMessageClientError),

    #[error("error verifying Authorization Request: {0}")]
    AuthRequestValidation(#[source] AuthRequestValidationError),

    #[error("incorrect client_id: expected {expected}, found {found}")]
    #[category(critical)]
    IncorrectClientId { expected: String, found: String },

    #[error("error parsing RP certificate: {0}")]
    RpCertificate(#[source] CertificateError),

    #[error("no reader registration in RP certificate")]
    #[category(critical)]
    MissingReaderRegistration,

    #[error("error validating requested attributes: {0}")]
    RequestedAttributesValidation(#[source] ValidationError),
}

#[derive(Debug, Constructor, thiserror::Error)]
#[error("could not perform actual disclosure, attributes were shared: {data_shared}, error: {error}")]
pub struct DisclosureError<E: Error> {
    pub data_shared: bool,
    #[source]
    pub error: E,
}

impl<E: Error> DisclosureError<E> {
    pub fn before_sharing(error: E) -> Self {
        Self {
            data_shared: false,
            error,
        }
    }

    pub fn after_sharing(error: E) -> Self {
        Self {
            data_shared: true,
            error,
        }
    }
}

impl From<VpMessageClientError> for DisclosureError<VpSessionError> {
    fn from(value: VpMessageClientError) -> Self {
        let data_shared = match &value {
            VpMessageClientError::Http(reqwest_error) => !reqwest_error.is_connect(),
            _ => true,
        };

        Self::new(data_shared, value.into())
    }
}
