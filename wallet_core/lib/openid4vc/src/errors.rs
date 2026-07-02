use std::error::Error;
use std::fmt::Debug;
use std::fmt::Display;
use std::str::FromStr;

use derive_more::Constructor;
use http::StatusCode;
use http_utils::error::HttpJsonError;
use http_utils::error::HttpJsonErrorType;
use jwt::wia::WiaError;
use serde::Deserialize;
use serde::Serialize;
use serde_with::DisplayFromStr;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use strum::EnumString;
use url::Url;

use crate::authorizing_issuer::AuthorizationRequestError;
use crate::authorizing_issuer::AuthorizeError;
use crate::authorizing_issuer::CompleteAuthorizationError;
use crate::authorizing_issuer::ParError;
use crate::issuer::CredentialPreviewError;
use crate::issuer::CredentialRequestError;
use crate::issuer::IssuanceError;
use crate::issuer::TokenRequestError;
use crate::verifier::CancelSessionError;
use crate::verifier::DisclosedAttributesError;
use crate::verifier::GetAuthRequestError;
use crate::verifier::NewSessionError;
use crate::verifier::PostAuthResponseError;
use crate::verifier::SessionError;
use crate::verifier::SessionStatus;
use crate::verifier::SessionStatusError;
use crate::verifier::WithRedirectUri;

pub trait ErrorWithCode: Error {
    type ErrorCode: Display;

    fn error_code(&self) -> Self::ErrorCode;
}

/// A type that wraps a `Box<dyn>` error and implements both the `Error` and `ToErrorCode` traits. This allows it to be
/// used as an error source for `thiserror` error types.
#[derive(Debug, derive_more::Display)]
pub struct BoxedErrorWithCode<T>(Box<dyn ErrorWithCode<ErrorCode = T> + Send + Sync + 'static>);

impl<T> BoxedErrorWithCode<T> {
    pub fn new(error: impl ErrorWithCode<ErrorCode = T> + Send + Sync + 'static) -> Self {
        Self(Box::new(error))
    }
}

// Implement the `Error` trait manually, in order to  work around a limitation of `thiserror`'s `#[source]` annotation.
// Unfortunately this annotation does not appear to work with boxed errors that are not `Box<dyn Error>`, but instead a
// superset of traits.
impl<T> Error for BoxedErrorWithCode<T>
where
    T: Debug,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self(inner) = self;

        Some(inner.as_ref())
    }
}

impl<T> ErrorWithCode for BoxedErrorWithCode<T>
where
    T: Debug + Display,
{
    type ErrorCode = T;

    fn error_code(&self) -> Self::ErrorCode {
        let Self(inner) = self;

        inner.error_code()
    }
}

/// Describes an error that occurred when processing an HTTP endpoint from the OAuth/OpenID protocol family.
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorResponse<T> {
    #[serde_as(as = "DisplayFromStr")]
    #[serde(bound(serialize = "T: Display", deserialize = "T: FromStr, T::Err: Display"))]
    pub error: T,
    pub error_description: Option<String>,
    pub error_uri: Option<Url>,
}

impl<E, T> From<E> for ErrorResponse<T>
where
    E: Display + ErrorWithCode<ErrorCode = T>,
{
    fn from(value: E) -> Self {
        Self {
            error: value.error_code(),
            error_description: Some(value.to_string()),
            error_uri: None,
        }
    }
}

/// Wrapper of [`ErrorResponse`] that adds the optional `state` parameter used by authorization error responses.
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizationErrorResponse<T> {
    #[serde(
        flatten,
        bound(serialize = "T: Display", deserialize = "T: FromStr, T::Err: Display")
    )]
    pub error_response: ErrorResponse<T>,
    pub state: Option<String>,
}

impl<T> AuthorizationErrorResponse<T> {
    pub fn error(&self) -> &T {
        &self.error_response.error
    }
}

/// Describes an error that occured at a HTTP(S) endpoint that is meant to be returned in a 303 See Other redirect.
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedirectErrorResponse<T> {
    pub auth_error_response: AuthorizationErrorResponse<T>,
    pub redirect_uri: Url,
}

/// Wraps an [`Error`] type that can be returned in a HTTP(S) redirect.
#[derive(Debug, thiserror::Error, Constructor)]
#[error("{error}")]
pub struct RedirectError<E>
where
    E: Error,
{
    #[source]
    pub error: E,
    pub redirect_uri: Url,
    pub state: Option<String>,
}

impl<E, T> From<RedirectError<E>> for RedirectErrorResponse<T>
where
    E: Error + ErrorWithCode<ErrorCode = T>,
{
    fn from(value: RedirectError<E>) -> Self {
        let RedirectError {
            error,
            redirect_uri,
            state,
        } = value;

        Self {
            auth_error_response: AuthorizationErrorResponse {
                error_response: ErrorResponse::from(error),
                state,
            },
            redirect_uri,
        }
    }
}

/// Describes an error that occured at a HTTP(S) endpoint that is meant to be returned either as a status code and
/// plain-text body or in a 303 See Other redirect.
#[derive(Debug, Clone)]
pub enum BodyOrRedirectErrorResponse<T> {
    Body { status_code: StatusCode, body_text: String },
    Redirect(RedirectErrorResponse<T>),
}

impl<T> BodyOrRedirectErrorResponse<T> {
    pub fn new_body(status_code: StatusCode, body_text: String) -> Self {
        Self::Body { status_code, body_text }
    }

    pub fn new_redirect(redirect_response: RedirectErrorResponse<T>) -> Self {
        Self::Redirect(redirect_response)
    }
}

impl<E, T> From<RedirectError<E>> for BodyOrRedirectErrorResponse<T>
where
    E: Error + ErrorWithCode<ErrorCode = T>,
{
    fn from(value: RedirectError<E>) -> Self {
        Self::new_redirect(value.into())
    }
}

/// Wrapper of [`ErrorResponse`] that has an optional redirect URI
/// and is as an error response for disclosure endpoints.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisclosureErrorResponse<T> {
    #[serde(
        flatten,
        bound(serialize = "T: Display", deserialize = "T: FromStr, T::Err: Display")
    )]
    pub error_response: ErrorResponse<T>,
    pub redirect_uri: Option<Url>,
}

impl<T> DisclosureErrorResponse<T> {
    pub fn error(&self) -> &T {
        &self.error_response.error
    }
}

impl<E, T> From<WithRedirectUri<E>> for DisclosureErrorResponse<T>
where
    E: Into<ErrorResponse<T>> + Error,
{
    fn from(value: WithRedirectUri<E>) -> Self {
        DisclosureErrorResponse {
            error_response: value.error.into(),
            redirect_uri: value.redirect_uri,
        }
    }
}

pub trait ErrorStatusCode {
    fn status_code(&self) -> StatusCode;
}

// OpenID4VCI Error Codes

/// The list of error codes that can result from an Authorization Request. Note that this is also used by OpenID4VP.
///
/// See: <https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.2.1>
#[derive(Debug, Clone, PartialEq, Eq, strum::Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum AuthorizationErrorCode {
    InvalidRequest,
    UnauthorizedClient,
    AccessDenied,
    UnsupportedResponseType,
    InvalidScope,
    ServerError,
    TemporarilyUnavailable,

    // Catch-all variant, in case the server sends an error code that the holder is not aware of.
    // Note that this is never to be used by the verifier.
    #[strum(default)]
    Other(String),
}

impl From<AuthorizeError> for BodyOrRedirectErrorResponse<AuthorizationErrorCode> {
    fn from(value: AuthorizeError) -> Self {
        let status_code = match value {
            // The errors at the Authorization Endpoint that can occur before the PAR is retrieved and the
            // `redirect_uri` is known are represented as HTTP status code and plain-text bodies.
            AuthorizeError::UnknownClient(_) => StatusCode::UNAUTHORIZED,
            AuthorizeError::ParStore(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AuthorizeError::UnknownRequestUri(_) => StatusCode::NOT_FOUND,
            AuthorizeError::MismatchedClient { .. } => StatusCode::UNAUTHORIZED,

            // Once the `redirect_uri` is known, convert the error to a 303 redirect instead.
            AuthorizeError::AuthorizationRequest(redirect_error) => return Self::new_redirect(redirect_error.into()),
        };

        Self::new_body(status_code, value.to_string())
    }
}

impl ErrorWithCode for AuthorizationRequestError {
    type ErrorCode = AuthorizationErrorCode;

    fn error_code(&self) -> Self::ErrorCode {
        match self {
            Self::InvalidAuthorizationRequest(_) => AuthorizationErrorCode::InvalidRequest,

            Self::NoValidScope(_) => AuthorizationErrorCode::InvalidScope,

            Self::AuthorizationCodeFlow(error) => error.error_code(),

            Self::CompleteAuthorization(error) => error.error_code(),
        }
    }
}

impl ErrorWithCode for CompleteAuthorizationError {
    type ErrorCode = AuthorizationErrorCode;

    fn error_code(&self) -> Self::ErrorCode {
        match self {
            Self::IssuableDocument(_) | Self::SessionStore(_) => AuthorizationErrorCode::ServerError,
        }
    }
}

/// The list of error codes that can result from the PAR POST request.
///
/// According to <https://datatracker.ietf.org/doc/html/rfc9126#section-2.3>, these can be taken either from
/// <https://datatracker.ietf.org/doc/html/rfc6749#section-5.2> or
/// <https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.2.1>, i.e. the token endpoint error codes or the
/// authorization endpoint error codes.
///
/// This type represents a selection among these error codes, containing only those that the issuer returns. Any other
/// error code that a third-party issuer sends to the wallet will use the `Other` variant.
#[derive(Debug, Clone, PartialEq, Eq, strum::Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum ParErrorCode {
    // Token error code.
    InvalidClient,
    // Both token and authorization error code.
    InvalidRequest,
    // Authorization error code.
    ServerError,
    // Catch-all variant, in case the issuer sends an error code that the holder is not aware of.
    // Note that this is never to be used by the issuer.
    #[strum(default)]
    Other(String),
}

impl ErrorStatusCode for ParErrorCode {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidClient => StatusCode::UNAUTHORIZED,

            Self::InvalidRequest => StatusCode::BAD_REQUEST,

            Self::ServerError | Self::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl ErrorWithCode for ParError {
    type ErrorCode = ParErrorCode;

    fn error_code(&self) -> Self::ErrorCode {
        match self {
            Self::UnknownClient(_) => ParErrorCode::InvalidClient,

            Self::InvalidRedirectUri(_) => ParErrorCode::InvalidRequest,

            Self::Store(_) => ParErrorCode::ServerError,
        }
    }
}

/// See <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-6.3>
/// and <https://www.rfc-editor.org/rfc/rfc6749.html#section-5.2>.
#[derive(Debug, Clone, PartialEq, Eq, strum::Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum TokenErrorCode {
    InvalidRequest,
    InvalidClient,
    InvalidGrant,
    UnauthorizedClient,
    UnsupportedGrantType,
    InvalidScope,

    /// This can be returned in case of internal server errors, i.e. with HTTP status code 5xx.
    /// This error type is not defined in the specs, but then again the entire HTTP response in case
    /// 5xx status codes is not defined by the specs, so we have freedom to return what we want.
    ServerError,

    // Catch-all variant, in case the issuer sends an error code that the holder is not aware of.
    // Note that this is never to be used by the issuer.
    #[strum(default)]
    Other(String),
}

impl ErrorStatusCode for TokenErrorCode {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidRequest => StatusCode::BAD_REQUEST,

            Self::InvalidClient => StatusCode::UNAUTHORIZED,

            Self::InvalidGrant | Self::UnauthorizedClient | Self::UnsupportedGrantType | Self::InvalidScope => {
                StatusCode::BAD_REQUEST
            }

            Self::ServerError | Self::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl ErrorWithCode for TokenRequestError {
    type ErrorCode = TokenErrorCode;

    fn error_code(&self) -> Self::ErrorCode {
        match self {
            Self::IssuanceError(IssuanceError::SessionStore(_)) => TokenErrorCode::ServerError,

            // A missing session (cleaned up) or a session in a terminal/wrong state (already used or
            // expired) both mean the authorization grant presented at `/token` is no longer valid.
            // Per RFC 6749 section 5.2 that is exactly `invalid_grant` ("invalid, expired, revoked ...").
            //
            // In the pre-authorized-code flow `invalid_grant` can *only* result from these two cases
            // (there are no PKCE / client_id / scope / redirect_uri checks that also yield it), so the
            // wallet can unambiguously map a pre-authorized `invalid_grant` onto a specific error without
            // relying on a non-standard error code.
            Self::SessionNotFound | Self::IssuanceError(IssuanceError::UnexpectedState) => TokenErrorCode::InvalidGrant,

            Self::IssuanceError(_) => TokenErrorCode::InvalidRequest,

            Self::UnexpectedGrantType { .. } => TokenErrorCode::UnsupportedGrantType,

            Self::MissingCodeVerifier | Self::PkceVerificationFailed => TokenErrorCode::InvalidGrant,

            Self::MissingClientId | Self::UnknownClient(_) => TokenErrorCode::InvalidClient,

            Self::ClientIdMismatch { .. } => TokenErrorCode::InvalidGrant,

            Self::ScopeMismatch { .. } => TokenErrorCode::InvalidScope,

            Self::MissingRedirectUri | Self::RedirectUriMismatch { .. } => TokenErrorCode::InvalidRequest,

            Self::CredentialConfigNotOffered(_) => TokenErrorCode::ServerError,
        }
    }
}

/// Error codes for the credential preview endpoint.
#[derive(Debug, Clone, PartialEq, Eq, strum::Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum CredentialPreviewErrorCode {
    InvalidRequest,
    InvalidToken,
    ServerError,

    // Catch-all variant, in case the issuer sends an error code that the holder is not aware of.
    // Note that this is never to be used by the issuer.
    #[strum(default)]
    Other(String),
}

impl ErrorStatusCode for CredentialPreviewErrorCode {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidRequest => StatusCode::BAD_REQUEST,

            Self::InvalidToken => StatusCode::UNAUTHORIZED,

            Self::ServerError | Self::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl ErrorWithCode for CredentialPreviewError {
    type ErrorCode = CredentialPreviewErrorCode;

    fn error_code(&self) -> Self::ErrorCode {
        match self {
            Self::IssuanceError(IssuanceError::SessionStore(_)) => CredentialPreviewErrorCode::ServerError,

            Self::IssuanceError(_) => CredentialPreviewErrorCode::InvalidRequest,

            Self::MalformedToken | CredentialPreviewError::Unauthorized => CredentialPreviewErrorCode::InvalidToken,

            Self::MissingCredentialConfiguration(_) => CredentialPreviewErrorCode::ServerError,
        }
    }
}

/// See <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-8.3.1>.
#[derive(Debug, Clone, PartialEq, Eq, strum::Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum CredentialErrorCode {
    InvalidCredentialRequest,
    UnknownCredentialConfiguration,
    UnknownCredentialIdentifier,
    InvalidProof,
    InvalidNonce,
    InvalidEncryptionParameters,
    CredentialRequestDenied,

    // From https://www.rfc-editor.org/rfc/rfc6750.html#section-3.1
    InvalidRequest,
    InvalidToken,
    InsufficientScope,

    /// This can be returned in case of internal server errors, i.e. with HTTP status code 5xx.
    /// This error type is not defined in the spec, but then again the entire HTTP response in case
    /// 5xx status codes is not defined by the spec, so we have freedom to return what we want.
    ServerError,

    // Catch-all variant, in case the issuer sends an error code that the holder is not aware of.
    // Note that this is never to be used by the issuer.
    #[strum(default)]
    Other(String),
}

impl ErrorStatusCode for CredentialErrorCode {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidCredentialRequest
            | Self::UnknownCredentialConfiguration
            | Self::UnknownCredentialIdentifier
            | Self::InvalidProof
            | Self::InvalidNonce
            | Self::InvalidEncryptionParameters
            | Self::CredentialRequestDenied
            | Self::InvalidRequest => StatusCode::BAD_REQUEST,

            Self::InvalidToken => StatusCode::UNAUTHORIZED,

            Self::InsufficientScope => StatusCode::FORBIDDEN,

            Self::ServerError | Self::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl ErrorWithCode for CredentialRequestError {
    type ErrorCode = CredentialErrorCode;

    fn error_code(&self) -> Self::ErrorCode {
        // TODO (PVW-5541): Return `CredentialErrorCode::UnknownCredentialIdentifier` when appropriate.
        // TODO (PVW-5538): Return `CredentialErrorCode::InvalidEncryptionParameters` when appropriate.
        match self {
            Self::IssuanceError(IssuanceError::UnexpectedState)
            | Self::IssuanceError(IssuanceError::UnknownSession(_))
            | Self::IssuanceError(IssuanceError::DpopInvalid(_)) => CredentialErrorCode::InvalidCredentialRequest,

            Self::IssuanceError(IssuanceError::SessionStore(_)) => CredentialErrorCode::ServerError,

            Self::Unauthorized | Self::MalformedToken => CredentialErrorCode::InvalidToken,

            Self::CredentialTypeNotOffered(_) => CredentialErrorCode::UnknownCredentialConfiguration,

            Self::UseBatchIssuance => CredentialErrorCode::InvalidCredentialRequest,

            Self::InvalidProofJwt(_) | Self::InvalidProofPublicKey(_) => CredentialErrorCode::InvalidProof,

            Self::MissingProofNonce => CredentialErrorCode::InvalidNonce,

            Self::ProofNonceStore(_) => CredentialErrorCode::ServerError,

            Self::InvalidNonce => CredentialErrorCode::InvalidNonce,

            Self::Jwt(_) => CredentialErrorCode::InvalidProof,

            Self::MissingCredentialConfiguration(_) => CredentialErrorCode::ServerError,

            Self::CredentialTypeMismatch { .. } | Self::WrongNumberOfCredentialRequests => {
                CredentialErrorCode::InvalidCredentialRequest
            }

            Self::MissingCredentialRequestPoP => CredentialErrorCode::InvalidProof,

            Self::MissingWia => CredentialErrorCode::InvalidCredentialRequest,

            Self::JwkConversion(_) | Self::MdocConversion(_) | Self::SdJwtConversion(_) => {
                CredentialErrorCode::ServerError
            }

            Self::Wia(WiaError::MissingNonce) => CredentialErrorCode::InvalidNonce,

            Self::Wia(_) => CredentialErrorCode::InvalidProof,

            Self::ObtainStatusClaim(_) | Self::IncorrectNumberOfStatusClaims(_) => CredentialErrorCode::ServerError,
        }
    }
}

// OpenID4VP Error Codes

#[derive(Debug, Clone, PartialEq, Eq, strum::Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum GetAuthRequestErrorCode {
    InvalidRequest,
    ExpiredEphemeralId,
    ExpiredSession,
    CancelledSession,
    UnknownSession,

    ServerError,

    // Catch-all variant, in case the verifier sends an error code that the holder is not aware of.
    // Note that this is never to be used by the verifier.
    #[strum(default)]
    Other(String),
}

impl ErrorStatusCode for GetAuthRequestErrorCode {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidRequest => StatusCode::BAD_REQUEST,

            // Per RFC 7235 we MUST include a `WWW-Authenticate` HTTP header with this, but we can't do that
            // conveniently here. It seems this header is often skipped, and we use it internally here, we skip it too.
            Self::ExpiredEphemeralId => StatusCode::UNAUTHORIZED,

            Self::ExpiredSession | Self::CancelledSession | Self::UnknownSession => StatusCode::NOT_FOUND,

            Self::ServerError | Self::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl ErrorWithCode for GetAuthRequestError {
    type ErrorCode = GetAuthRequestErrorCode;

    fn error_code(&self) -> Self::ErrorCode {
        match self {
            Self::Session(SessionError::SessionStore(_)) => GetAuthRequestErrorCode::ServerError,

            Self::Session(SessionError::UnknownSession(_)) => GetAuthRequestErrorCode::UnknownSession,

            Self::Session(SessionError::UnexpectedState(SessionStatus::Cancelled)) => {
                GetAuthRequestErrorCode::CancelledSession
            }

            Self::Session(SessionError::UnexpectedState(SessionStatus::Expired)) => {
                GetAuthRequestErrorCode::ExpiredSession
            }

            Self::Session(SessionError::UnexpectedState(_)) => GetAuthRequestErrorCode::InvalidRequest,

            Self::InvalidEphemeralId(_) => GetAuthRequestErrorCode::InvalidRequest,

            Self::ExpiredEphemeralId(_) => GetAuthRequestErrorCode::ExpiredEphemeralId,

            Self::Jwt(_) | Self::ReturnUrlConfigurationMismatch | Self::UnknownUseCase(_) => {
                GetAuthRequestErrorCode::ServerError
            }

            Self::QueryParametersMissing | Self::QueryParametersDeserialization(_) => {
                GetAuthRequestErrorCode::InvalidRequest
            }
        }
    }
}

/// <https://openid.net/specs/openid-4-verifiable-presentations-1_0-20.html#name-error-response>
#[derive(Debug, Clone, PartialEq, Eq, strum::Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum PostAuthResponseErrorCode {
    InvalidRequest,
    ExpiredSession,
    CancelledSession,
    UnknownSession,

    ServerError,

    /// An NL Wallet specific error code, meaning the following: in a disclosure based issuance session,
    /// the issuer found no attestations to issue.
    NoIssuableAttestations,

    // Catch-all variant, in case the verifier sends an error code that the holder is not aware of.
    // Note that this is never to be used by the verifier.
    #[strum(default)]
    Other(String),
}

impl ErrorStatusCode for PostAuthResponseErrorCode {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidRequest => StatusCode::BAD_REQUEST,

            Self::ExpiredSession | Self::CancelledSession | Self::UnknownSession => StatusCode::NOT_FOUND,

            Self::ServerError => StatusCode::INTERNAL_SERVER_ERROR,

            Self::NoIssuableAttestations => StatusCode::NOT_FOUND,

            Self::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl ErrorWithCode for PostAuthResponseError {
    type ErrorCode = PostAuthResponseErrorCode;

    fn error_code(&self) -> Self::ErrorCode {
        match self {
            Self::Session(SessionError::SessionStore(_)) => PostAuthResponseErrorCode::ServerError,

            Self::Session(SessionError::UnknownSession(_)) => PostAuthResponseErrorCode::UnknownSession,

            Self::Session(SessionError::UnexpectedState(SessionStatus::Cancelled)) => {
                PostAuthResponseErrorCode::CancelledSession
            }

            Self::Session(SessionError::UnexpectedState(SessionStatus::Expired)) => {
                PostAuthResponseErrorCode::ExpiredSession
            }

            Self::Session(SessionError::UnexpectedState(_)) | Self::AuthResponse(_) => {
                PostAuthResponseErrorCode::InvalidRequest
            }

            Self::HandlingDisclosureResult(err) => err.error_code(),

            Self::ResponseEncoding(_) => PostAuthResponseErrorCode::ServerError,
        }
    }
}

/// Error codes that the wallet sends to the verifier when it encounters an error or rejects the session.
/// See: https://openid.net/specs/openid-4-verifiable-presentations-1_0.html#section-8.5
#[derive(Debug, Clone, PartialEq, Eq, strum::Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum VpAuthorizationErrorCode {
    InvalidClient,
    VpFormatsNotSupported,
    InvalidRequestUriMethod,
    InvalidTransactionData,
    WalletUnavailable,

    #[strum(default)]
    AuthorizationError(AuthorizationErrorCode),
}

// The RP error types and `VerificationErrorCode` are handled differently from the errors above:
// instead of returning them as an `ErrorResponse`, they are returned as a `HttpJsonErrorBody`.
// This is because the endpoints that return these errors are not part of a protocol from the
// OAuth/OpenID family, which uses `ErrorResponse`, but instead they are specific to this implementation.

/// Error codes sent to the Relying Party when an error occurs when handling their request.
#[derive(Debug, Clone, strum::Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum VerificationErrorCode {
    ServerError,
    InvalidRequest,
    UnknownSession,
    Nonce,
    SessionState,
}

impl HttpJsonErrorType for VerificationErrorCode {
    fn title(&self) -> &'static str {
        match self {
            Self::ServerError => "Internal server error occurred",
            Self::InvalidRequest => "Invalid request",
            Self::UnknownSession => "Unknown session",
            Self::Nonce => "Redirect URI nonce incorrect or missing",
            Self::SessionState => "Session is not in the required state",
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::ServerError => StatusCode::INTERNAL_SERVER_ERROR,

            Self::InvalidRequest => StatusCode::BAD_REQUEST,

            Self::UnknownSession => StatusCode::NOT_FOUND,

            // See the other comment on `StatusCode::UNAUTHORIZED`
            Self::Nonce => StatusCode::UNAUTHORIZED,

            Self::SessionState => StatusCode::BAD_REQUEST,
        }
    }
}

impl From<&SessionError> for VerificationErrorCode {
    fn from(error: &SessionError) -> Self {
        match error {
            SessionError::SessionStore(_) => VerificationErrorCode::ServerError,
            SessionError::UnknownSession(_) => VerificationErrorCode::UnknownSession,
            SessionError::UnexpectedState(_) => VerificationErrorCode::SessionState,
        }
    }
}

impl From<&NewSessionError> for VerificationErrorCode {
    fn from(error: &NewSessionError) -> Self {
        match error {
            NewSessionError::Session(session_error) => session_error.into(),
            NewSessionError::NoCredentialRequests
            | NewSessionError::UnknownUseCase(_)
            | NewSessionError::UnsupportedDcqlFeatures(_)
            | NewSessionError::ReturnUrlConfigurationMismatch => VerificationErrorCode::InvalidRequest,
        }
    }
}

impl From<&SessionStatusError> for VerificationErrorCode {
    fn from(error: &SessionStatusError) -> Self {
        match error {
            SessionStatusError::Session(session_error) => session_error.into(),
            SessionStatusError::UrlEncoding(_) => VerificationErrorCode::ServerError,
        }
    }
}

impl From<&CancelSessionError> for VerificationErrorCode {
    fn from(error: &CancelSessionError) -> Self {
        match error {
            CancelSessionError::Session(session_error) => session_error.into(),
        }
    }
}

impl From<&DisclosedAttributesError> for VerificationErrorCode {
    fn from(error: &DisclosedAttributesError) -> Self {
        match error {
            DisclosedAttributesError::Session(session_error) => session_error.into(),
            DisclosedAttributesError::RedirectUriNonceMissing
            | DisclosedAttributesError::RedirectUriNonceMismatch(_) => Self::Nonce,
        }
    }
}

impl From<NewSessionError> for HttpJsonError<VerificationErrorCode> {
    fn from(error: NewSessionError) -> Self {
        HttpJsonError::from_error(&error)
    }
}

impl From<SessionStatusError> for HttpJsonError<VerificationErrorCode> {
    fn from(error: SessionStatusError) -> Self {
        HttpJsonError::from_error(&error)
    }
}

impl From<CancelSessionError> for HttpJsonError<VerificationErrorCode> {
    fn from(error: CancelSessionError) -> Self {
        HttpJsonError::from_error(&error)
    }
}

#[skip_serializing_none]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DisclosedAttributesErrorData {
    pub session_status: Option<String>,
    pub session_error: Option<String>,
}

impl From<DisclosedAttributesError> for HttpJsonError<VerificationErrorCode> {
    fn from(error: DisclosedAttributesError) -> Self {
        let r#type = (&error).into();
        let detail = error.to_string();

        // The `session_status` field is only included if the session was in an unexpected state,
        // while the `session_error` field is further only included if that status is "FAILED".
        let data = match error {
            DisclosedAttributesError::Session(SessionError::UnexpectedState(session_status)) => {
                let status = Some(session_status.to_string());
                let error = match session_status {
                    SessionStatus::Failed { error } => Some(error),
                    _ => None,
                };

                DisclosedAttributesErrorData {
                    session_status: status,
                    session_error: error,
                }
            }
            _ => Default::default(),
        };

        // As `DisclosedAttributesErrorData` is a struct that only contains two simple strings,
        // we can assume that this will serialize to a `serde_json::Map` without fault.
        let Ok(serde_json::Value::Object(data)) = serde_json::to_value(data) else {
            panic!("serialized DisclosedAttributesErrorData should be an object");
        };

        Self::new(r#type, detail, data)
    }
}

// Other OAuth error codes

/// https://www.rfc-editor.org/rfc/rfc6750.html#section-3.1
#[derive(Debug, Clone, PartialEq, Eq, strum::Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum AuthBearerErrorCode {
    InvalidRequest,
    InvalidToken,
    InsufficientScope,

    // Catch-all variant, in case the server sends an error code that the holder is not aware of.
    // Note that this is never to be used by a server.
    #[strum(default)]
    Other(String),
}

#[cfg(feature = "axum")]
mod axum {
    use std::fmt::Debug;
    use std::fmt::Display;

    use axum::Json;
    use axum::response::IntoResponse;
    use axum::response::Redirect;
    use axum::response::Response;
    use tracing::warn;

    use super::BodyOrRedirectErrorResponse;
    use super::DisclosureErrorResponse;
    use super::ErrorResponse;
    use super::ErrorStatusCode;
    use super::RedirectErrorResponse;

    impl<T> IntoResponse for ErrorResponse<T>
    where
        T: ErrorStatusCode + Display + Debug,
    {
        fn into_response(self) -> Response {
            warn!("Responding with error body: {:?}", &self);

            (self.error.status_code(), Json(self)).into_response()
        }
    }

    impl<T> IntoResponse for RedirectErrorResponse<T>
    where
        T: Display + Debug,
    {
        fn into_response(self) -> Response {
            warn!("Responding with error redirect: {:?}", &self);

            let mut redirect_uri = self.redirect_uri;

            {
                let mut query_pairs = redirect_uri.query_pairs_mut();

                query_pairs.append_pair("error", &self.auth_error_response.error_response.error.to_string());

                if let Some(error_description) = self.auth_error_response.error_response.error_description.as_deref() {
                    query_pairs.append_pair("error_description", error_description);
                }

                if let Some(error_uri) = self.auth_error_response.error_response.error_uri.as_ref() {
                    query_pairs.append_pair("error_description", error_uri.as_str());
                }

                if let Some(state) = self.auth_error_response.state.as_deref() {
                    query_pairs.append_pair("state", state);
                }
            }

            Redirect::to(redirect_uri.as_str()).into_response()
        }
    }

    impl<T> IntoResponse for BodyOrRedirectErrorResponse<T>
    where
        T: Display + Debug,
    {
        fn into_response(self) -> Response {
            match self {
                Self::Body { status_code, body_text } => {
                    warn!("Responding with error body ({status_code}): {body_text}");

                    (status_code, body_text).into_response()
                }
                Self::Redirect(redirect_response) => redirect_response.into_response(),
            }
        }
    }

    impl<T> IntoResponse for DisclosureErrorResponse<T>
    where
        T: ErrorStatusCode + Display + Debug,
    {
        fn into_response(self) -> Response {
            warn!("Responding with error body: {:?}", &self);

            (self.error_response.error.status_code(), Json(self)).into_response()
        }
    }

    #[cfg(test)]
    mod tests {
        use axum::response::IntoResponse;
        use derive_more::Display;
        use http::header;
        use url::Url;

        use super::super::ErrorWithCode;
        use super::super::RedirectError;
        use super::super::RedirectErrorResponse;

        #[test]
        fn test_redirect_error_into_response() {
            #[derive(Debug, thiserror::Error)]
            #[error("{0}")]
            struct ExampleError(String);

            #[derive(Debug, Display)]
            #[display("something_happened")]
            struct ExampleErrorCode;

            impl ErrorWithCode for ExampleError {
                type ErrorCode = ExampleErrorCode;

                fn error_code(&self) -> Self::ErrorCode {
                    ExampleErrorCode
                }
            }

            let example_error = ExampleError("Something happened 猫".to_string());
            let redirect_uri = "http://example.com/redirect?foo=bar".parse().unwrap();
            let state = "wallet_state".to_string();

            let redirect_error = RedirectError::new(example_error, redirect_uri, Some(state));
            let error_response = RedirectErrorResponse::<ExampleErrorCode>::from(redirect_error);

            let response = error_response.into_response();

            let location_header = response
                .headers()
                .get(header::LOCATION)
                .expect("response should have Location header");

            let url = location_header
                .to_str()
                .unwrap()
                .parse::<Url>()
                .expect("Location header should contain a URL");

            assert_eq!(
                url.query(),
                Some(
                    "foo=bar&error=something_happened&error_description=Something+happened+%E7%8C%AB&\
                     state=wallet_state"
                )
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ErrorWithCode;
    use super::TokenErrorCode;
    use crate::issuer::IssuanceError;
    use crate::issuer::TokenRequestError;

    #[test]
    fn expired_or_used_session_maps_to_invalid_grant() {
        // A missing session (cleaned up) and a session in a terminal/wrong state (already used or
        // expired) both mean the authorization grant is no longer valid, which per RFC 6749 section
        // 5.2 is `invalid_grant`. In the pre-authorized-code flow this is the only source of
        // `invalid_grant`, which lets the wallet render the "QR code no longer valid" screen.
        assert_eq!(
            TokenRequestError::SessionNotFound.error_code(),
            TokenErrorCode::InvalidGrant
        );
        assert_eq!(
            TokenRequestError::IssuanceError(IssuanceError::UnexpectedState).error_code(),
            TokenErrorCode::InvalidGrant
        );
    }
}
