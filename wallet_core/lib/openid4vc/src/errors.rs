use std::fmt::Display;
use std::str::FromStr;

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

use crate::authorization_code_flow::InvalidAuthorizationRequest;
use crate::authorizing_issuer::AuthorizeError;
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
    E: Display + Into<T>,
{
    fn from(value: E) -> Self {
        let error_description = Some(value.to_string());

        Self {
            error: value.into(),
            error_description,
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
    E: Into<ErrorResponse<T>> + std::error::Error,
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

/// Wire-format error codes for the authorization endpoint.
#[derive(Debug, Clone, PartialEq, Eq, strum::Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum AuthorizeErrorCode {
    InvalidClient,
    InvalidRequest,
    ServerError,

    // Catch-all variant, in case the issuer sends an error code that the holder is not aware of.
    // Note that this is never to be used by the issuer.
    #[strum(default)]
    Other(String),
}

impl ErrorStatusCode for AuthorizeErrorCode {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidClient => StatusCode::UNAUTHORIZED,

            Self::InvalidRequest => StatusCode::BAD_REQUEST,

            Self::ServerError | Self::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<AuthorizeError> for AuthorizeErrorCode {
    fn from(value: AuthorizeError) -> Self {
        match value {
            AuthorizeError::UnknownClient(_) | AuthorizeError::MismatchedClient { .. } => Self::InvalidClient,

            AuthorizeError::UnknownRequestUri(_)
            | AuthorizeError::InvalidAuthorizationRequest(InvalidAuthorizationRequest::UnsupportedCodeChallenge)
            | AuthorizeError::NoValidScope(_) => Self::InvalidRequest,

            AuthorizeError::ParStore(_)
            | AuthorizeError::AuthorizationCodeFlow(_)
            | AuthorizeError::CompleteAuthorization(_) => Self::ServerError,
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

impl From<ParError> for ParErrorCode {
    fn from(value: ParError) -> Self {
        match value {
            ParError::UnknownClient(_) => Self::InvalidClient,

            ParError::InvalidRedirectUri(_) => Self::InvalidRequest,

            ParError::Store(_) => Self::ServerError,
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

impl From<TokenRequestError> for TokenErrorCode {
    fn from(err: TokenRequestError) -> Self {
        match err {
            TokenRequestError::IssuanceError(IssuanceError::SessionStore(_)) => Self::ServerError,

            TokenRequestError::SessionNotFound => Self::InvalidGrant,

            TokenRequestError::IssuanceError(_) => Self::InvalidRequest,

            TokenRequestError::UnexpectedGrantType { .. } => Self::UnsupportedGrantType,

            TokenRequestError::MissingCodeVerifier | TokenRequestError::PkceVerificationFailed => Self::InvalidGrant,

            TokenRequestError::MissingClientId | TokenRequestError::UnknownClient(_) => Self::InvalidClient,

            TokenRequestError::ClientIdMismatch { .. } => Self::InvalidGrant,

            TokenRequestError::ScopeMismatch { .. } => Self::InvalidScope,

            TokenRequestError::MissingRedirectUri | TokenRequestError::RedirectUriMismatch { .. } => {
                Self::InvalidRequest
            }

            TokenRequestError::CredentialConfigNotOffered(_) => Self::ServerError,
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

impl From<CredentialPreviewError> for CredentialPreviewErrorCode {
    fn from(value: CredentialPreviewError) -> Self {
        match value {
            CredentialPreviewError::IssuanceError(IssuanceError::SessionStore(_)) => Self::ServerError,

            CredentialPreviewError::IssuanceError(_) => Self::InvalidRequest,

            CredentialPreviewError::MalformedToken | CredentialPreviewError::Unauthorized => Self::InvalidToken,

            CredentialPreviewError::MissingCredentialConfiguration(_) => Self::ServerError,

            CredentialPreviewError::CredentialPreviewsNotFound => Self::InvalidRequest,
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

impl From<CredentialRequestError> for CredentialErrorCode {
    fn from(value: CredentialRequestError) -> Self {
        // TODO (PVW-5541): Return `CredentialErrorCode::UnknownCredentialIdentifier` when appropriate.
        // TODO (PVW-5538): Return `CredentialErrorCode::InvalidEncryptionParameters` when appropriate.
        match value {
            CredentialRequestError::IssuanceError(IssuanceError::UnexpectedState)
            | CredentialRequestError::IssuanceError(IssuanceError::UnknownSession(_))
            | CredentialRequestError::IssuanceError(IssuanceError::DpopInvalid(_)) => Self::InvalidCredentialRequest,

            CredentialRequestError::IssuanceError(IssuanceError::SessionStore(_)) => Self::ServerError,

            CredentialRequestError::Unauthorized | CredentialRequestError::MalformedToken => Self::InvalidToken,

            CredentialRequestError::CredentialTypeNotOffered(_) => Self::UnknownCredentialConfiguration,

            CredentialRequestError::UseBatchIssuance => Self::InvalidCredentialRequest,

            CredentialRequestError::InvalidProofJwt(_) | CredentialRequestError::InvalidProofPublicKey(_) => {
                Self::InvalidProof
            }

            CredentialRequestError::MissingProofNonce => Self::InvalidNonce,

            CredentialRequestError::ProofNonceStore(_) => Self::ServerError,

            CredentialRequestError::InvalidNonce => Self::InvalidNonce,

            CredentialRequestError::Jwt(_) => Self::InvalidProof,

            CredentialRequestError::MissingCredentialConfiguration(_) => Self::ServerError,

            CredentialRequestError::CredentialTypeMismatch { .. }
            | CredentialRequestError::WrongNumberOfCredentialRequests => Self::InvalidCredentialRequest,

            CredentialRequestError::MissingCredentialRequestPoP => Self::InvalidProof,

            CredentialRequestError::MissingWia => Self::InvalidCredentialRequest,

            CredentialRequestError::JwkConversion(_)
            | CredentialRequestError::MdocConversion(_)
            | CredentialRequestError::SdJwtConversion(_) => Self::ServerError,

            CredentialRequestError::Wia(WiaError::MissingNonce) => Self::InvalidNonce,

            CredentialRequestError::Wia(_) => Self::InvalidProof,

            CredentialRequestError::ObtainStatusClaim(_) | CredentialRequestError::IncorrectNumberOfStatusClaims(_) => {
                Self::ServerError
            }
        }
    }
}

// OpenID4VP Error Codes

#[derive(Debug, Clone, PartialEq, Eq, strum::Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum GetRequestErrorCode {
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

impl ErrorStatusCode for GetRequestErrorCode {
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

impl From<GetAuthRequestError> for GetRequestErrorCode {
    fn from(value: GetAuthRequestError) -> Self {
        match value {
            GetAuthRequestError::Session(SessionError::SessionStore(_)) => Self::ServerError,

            GetAuthRequestError::Session(SessionError::UnknownSession(_)) => Self::UnknownSession,

            GetAuthRequestError::Session(SessionError::UnexpectedState(SessionStatus::Cancelled)) => {
                Self::CancelledSession
            }

            GetAuthRequestError::Session(SessionError::UnexpectedState(SessionStatus::Expired)) => Self::ExpiredSession,

            GetAuthRequestError::Session(SessionError::UnexpectedState(_)) => Self::InvalidRequest,

            GetAuthRequestError::InvalidEphemeralId(_) => Self::InvalidRequest,

            GetAuthRequestError::ExpiredEphemeralId(_) => Self::ExpiredEphemeralId,

            GetAuthRequestError::Jwt(_)
            | GetAuthRequestError::ReturnUrlConfigurationMismatch
            | GetAuthRequestError::UnknownUseCase(_) => Self::ServerError,

            GetAuthRequestError::QueryParametersMissing | GetAuthRequestError::QueryParametersDeserialization(_) => {
                Self::InvalidRequest
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

impl From<PostAuthResponseError> for PostAuthResponseErrorCode {
    fn from(value: PostAuthResponseError) -> Self {
        match value {
            PostAuthResponseError::Session(SessionError::SessionStore(_)) => Self::ServerError,

            PostAuthResponseError::Session(SessionError::UnknownSession(_)) => Self::UnknownSession,

            PostAuthResponseError::Session(SessionError::UnexpectedState(SessionStatus::Cancelled)) => {
                Self::CancelledSession
            }

            PostAuthResponseError::Session(SessionError::UnexpectedState(SessionStatus::Expired)) => {
                Self::ExpiredSession
            }

            PostAuthResponseError::Session(SessionError::UnexpectedState(_))
            | PostAuthResponseError::AuthResponse(_) => Self::InvalidRequest,

            PostAuthResponseError::HandlingDisclosureResult(err) => err.as_ref().to_error_code(),

            PostAuthResponseError::ResponseEncoding(_) => Self::ServerError,
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

/// https://www.rfc-editor.org/rfc/rfc6749.html#section-4.1.2.1
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

    // Catch-all variant, in case the verifier sends an error code that the holder is not aware of.
    // Note that this is never to be used by the verifier.
    #[strum(default)]
    Other(String),
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

    // Catch-all variant, in case the verifier sends an error code that the holder is not aware of.
    // Note that this is never to be used by a verifier.
    #[strum(default)]
    Other(String),
}

impl HttpJsonErrorType for VerificationErrorCode {
    fn title(&self) -> Option<&'static str> {
        match self {
            Self::ServerError => Some("Internal server error occurred"),
            Self::InvalidRequest => Some("Invalid request"),
            Self::UnknownSession => Some("Unknown session"),
            Self::Nonce => Some("Redirect URI nonce incorrect or missing"),
            Self::SessionState => Some("Session is not in the required state"),
            Self::Other(_) => None,
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

            Self::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
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
    use axum::response::Response;
    use tracing::warn;

    use super::DisclosureErrorResponse;
    use super::ErrorResponse;
    use super::ErrorStatusCode;

    impl<T> IntoResponse for ErrorResponse<T>
    where
        T: ErrorStatusCode + Display + Debug,
    {
        fn into_response(self) -> Response {
            warn!("Responding with error body: {:?}", &self);

            (self.error.status_code(), Json(self)).into_response()
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
}
