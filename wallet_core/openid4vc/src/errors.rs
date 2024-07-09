use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

use wallet_common::{
    config::wallet_config::BaseUrl,
    http_error::{HttpJsonError, HttpJsonErrorType},
};

use crate::{
    issuer::{CredentialRequestError, IssuanceError, TokenRequestError},
    verifier::{GetAuthRequestError, PostAuthResponseError, SessionError, VerificationError, WithRedirectUri},
};

/// Describes an error that occurred when processing an HTTP endpoint from the OAuth/OpenID protocol family.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse<T> {
    pub error: T,
    pub error_description: Option<String>,
    pub error_uri: Option<Url>,
}

/// Wrapper of [`ErrorResponse`] that has an optional redirect URI
/// and is as an error response for disclosure endpoints.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisclosureErrorResponse<T> {
    #[serde(flatten)]
    pub error_response: ErrorResponse<T>,
    pub redirect_uri: Option<BaseUrl>,
}

pub trait ErrorStatusCode {
    fn status_code(&self) -> StatusCode;
}

/// https://openid.github.io/OpenID4VCI/openid-4-verifiable-credential-issuance-wg-draft.html#name-credential-error-response
#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialErrorCode {
    InvalidCredentialRequest,
    UnsupportedCredentialType,
    UnsupportedCredentialFormat,
    InvalidProof,
    InvalidEncryptionParameters,

    // From https://www.rfc-editor.org/rfc/rfc6750.html#section-3.1
    InvalidRequest,
    InvalidToken,
    InsufficientScope,

    /// This can be returned in case of internal server errors, i.e. with HTTP status code 5xx.
    /// This error type is not defined in the spec, but then again the entire HTTP response in case
    /// 5xx status codes is not defined by the spec, so we have freedom to return what we want.
    ServerError,
}

impl From<CredentialRequestError> for ErrorResponse<CredentialErrorCode> {
    fn from(err: CredentialRequestError) -> ErrorResponse<CredentialErrorCode> {
        let description = err.to_string();
        ErrorResponse {
            error: match err {
                CredentialRequestError::IssuanceError(IssuanceError::SessionStore(_))
                | CredentialRequestError::CoseKeyConversion(_)
                | CredentialRequestError::MissingPrivateKey(_)
                | CredentialRequestError::AttestationSigning(_)
                | CredentialRequestError::CborSerialization(_)
                | CredentialRequestError::JsonSerialization(_) => CredentialErrorCode::ServerError,
                CredentialRequestError::IssuanceError(_) | CredentialRequestError::UseBatchIssuance => {
                    CredentialErrorCode::InvalidRequest
                }
                CredentialRequestError::Unauthorized | CredentialRequestError::MalformedToken => {
                    CredentialErrorCode::InvalidToken
                }
                CredentialRequestError::UnsupportedJwtAlgorithm { .. }
                | CredentialRequestError::MissingJwk
                | CredentialRequestError::IncorrectNonce
                | CredentialRequestError::JwtDecodingFailed(_)
                | CredentialRequestError::JwkConversion(_)
                | CredentialRequestError::MissingCredentialRequestPoP => CredentialErrorCode::InvalidProof,
                CredentialRequestError::DoctypeMismatch | CredentialRequestError::DoctypeNotOffered(_) => {
                    CredentialErrorCode::InvalidCredentialRequest
                }
                CredentialRequestError::UnsupportedCredentialFormat(_) => {
                    CredentialErrorCode::UnsupportedCredentialFormat
                }
            },
            error_description: Some(description),
            error_uri: None,
        }
    }
}

impl ErrorStatusCode for CredentialErrorCode {
    fn status_code(&self) -> reqwest::StatusCode {
        match self {
            CredentialErrorCode::InvalidCredentialRequest
            | CredentialErrorCode::UnsupportedCredentialType
            | CredentialErrorCode::UnsupportedCredentialFormat
            | CredentialErrorCode::InvalidProof
            | CredentialErrorCode::InvalidEncryptionParameters
            | CredentialErrorCode::InvalidRequest => StatusCode::BAD_REQUEST,
            CredentialErrorCode::ServerError => StatusCode::INTERNAL_SERVER_ERROR,
            CredentialErrorCode::InvalidToken => StatusCode::UNAUTHORIZED,
            CredentialErrorCode::InsufficientScope => StatusCode::FORBIDDEN,
        }
    }
}

/// https://openid.github.io/OpenID4VCI/openid-4-verifiable-credential-issuance-wg-draft.html#section-6.3
/// and https://www.rfc-editor.org/rfc/rfc6749.html#section-5.2.
#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenErrorCode {
    InvalidRequest,
    InvalidClient,
    InvalidGrant,
    UnauthorizedClient,
    UnsupportedGrantType,
    InvalidScope,
    AuthorizationPending, // OpenID4VCI-specific error type
    SlowDown,             // OpenID4VCI-specific error type

    /// This can be returned in case of internal server errors, i.e. with HTTP status code 5xx.
    /// This error type is not defined in the specs, but then again the entire HTTP response in case
    /// 5xx status codes is not defined by the specs, so we have freedom to return what we want.
    ServerError,
}

impl From<TokenRequestError> for ErrorResponse<TokenErrorCode> {
    fn from(err: TokenRequestError) -> Self {
        let description = err.to_string();
        ErrorResponse {
            error: match err {
                TokenRequestError::IssuanceError(IssuanceError::SessionStore(_))
                | TokenRequestError::AttributeService(_) => TokenErrorCode::ServerError,
                TokenRequestError::IssuanceError(_) => TokenErrorCode::InvalidRequest,
                TokenRequestError::UnsupportedTokenRequestType => TokenErrorCode::UnsupportedGrantType,
            },
            error_description: Some(description),
            error_uri: None,
        }
    }
}

impl ErrorStatusCode for TokenErrorCode {
    fn status_code(&self) -> reqwest::StatusCode {
        match self {
            TokenErrorCode::InvalidRequest
            | TokenErrorCode::InvalidGrant
            | TokenErrorCode::UnauthorizedClient
            | TokenErrorCode::UnsupportedGrantType
            | TokenErrorCode::InvalidScope
            | TokenErrorCode::AuthorizationPending
            | TokenErrorCode::SlowDown => StatusCode::BAD_REQUEST,
            TokenErrorCode::InvalidClient => StatusCode::UNAUTHORIZED,
            TokenErrorCode::ServerError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GetRequestErrorCode {
    InvalidRequest,
    ExpiredEphemeralId,
    ExpiredSession,
    UnknownSession,

    ServerError,
}

impl From<GetAuthRequestError> for ErrorResponse<GetRequestErrorCode> {
    fn from(err: GetAuthRequestError) -> Self {
        let description = err.to_string();
        ErrorResponse {
            error: match err {
                GetAuthRequestError::ExpiredEphemeralId(_) => GetRequestErrorCode::ExpiredEphemeralId,
                GetAuthRequestError::Session(SessionError::Expired) => GetRequestErrorCode::ExpiredSession,
                GetAuthRequestError::Session(SessionError::UnknownSession(_)) => GetRequestErrorCode::UnknownSession,
                GetAuthRequestError::EncryptionKey(_)
                | GetAuthRequestError::AuthRequest(_)
                | GetAuthRequestError::Jwt(_)
                | GetAuthRequestError::ReturnUrlConfigurationMismatch
                | GetAuthRequestError::UnknownUseCase(_)
                | GetAuthRequestError::Session(SessionError::SessionStore(_)) => GetRequestErrorCode::ServerError,
                GetAuthRequestError::QueryParametersMissing
                | GetAuthRequestError::QueryParametersDeserialization(_)
                | GetAuthRequestError::InvalidEphemeralId(_)
                | GetAuthRequestError::Session(SessionError::UnexpectedState) => GetRequestErrorCode::InvalidRequest,
            },
            error_description: Some(description),
            error_uri: None,
        }
    }
}

impl ErrorStatusCode for GetRequestErrorCode {
    fn status_code(&self) -> reqwest::StatusCode {
        match self {
            GetRequestErrorCode::ServerError => StatusCode::INTERNAL_SERVER_ERROR,
            GetRequestErrorCode::ExpiredSession | GetRequestErrorCode::UnknownSession => StatusCode::NOT_FOUND,
            GetRequestErrorCode::InvalidRequest => StatusCode::BAD_REQUEST,

            // Per RFC 7235 we MUST include a `WWW-Authenticate` HTTP header with this, but we can't do that
            // conveniently here. It seems this header is often skipped, and we use it internally here, we skip it too.
            GetRequestErrorCode::ExpiredEphemeralId => StatusCode::UNAUTHORIZED,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PostAuthResponseErrorCode {
    InvalidRequest,
    ExpiredSession,
    UnknownSession,

    ServerError,
}

impl From<PostAuthResponseError> for ErrorResponse<PostAuthResponseErrorCode> {
    fn from(err: PostAuthResponseError) -> Self {
        let description = err.to_string();
        ErrorResponse {
            error: match err {
                PostAuthResponseError::Session(SessionError::Expired) => PostAuthResponseErrorCode::ExpiredSession,
                PostAuthResponseError::Session(SessionError::SessionStore(_)) => PostAuthResponseErrorCode::ServerError,
                PostAuthResponseError::Session(SessionError::UnknownSession(_)) => {
                    PostAuthResponseErrorCode::UnknownSession
                }
                PostAuthResponseError::AuthResponse(_)
                | PostAuthResponseError::Session(SessionError::UnexpectedState) => {
                    PostAuthResponseErrorCode::InvalidRequest
                }
            },
            error_description: Some(description),
            error_uri: None,
        }
    }
}

impl ErrorStatusCode for PostAuthResponseErrorCode {
    fn status_code(&self) -> reqwest::StatusCode {
        match self {
            PostAuthResponseErrorCode::ExpiredSession | PostAuthResponseErrorCode::UnknownSession => {
                StatusCode::NOT_FOUND
            }
            PostAuthResponseErrorCode::ServerError => StatusCode::INTERNAL_SERVER_ERROR,
            PostAuthResponseErrorCode::InvalidRequest => StatusCode::BAD_REQUEST,
        }
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

// The `VerificationError` and `VerificationErrorCode` is handled differently from the errors above:
// instead of returning them as an `ErrorResponse`, they are returned as a `HttpJsonErrorBody`.
// This is because the endpoints that return these errors are not part of a protocol from the
// OAuth/OpenID family, which uses `ErrorResponse`, but instead they are specific to this implementation.

/// Error codes sent to the Relying Party when an error occurs when handling their request.
#[derive(Debug, Clone, Copy, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum VerificationErrorCode {
    ServerError,
    InvalidRequest,
    UnknownSession,
    Nonce,
    SessionState,
}

impl HttpJsonErrorType for VerificationErrorCode {
    fn title(&self) -> String {
        match self {
            VerificationErrorCode::ServerError => "Internal server error occurred".to_string(),
            VerificationErrorCode::InvalidRequest => "Invalid request".to_string(),
            VerificationErrorCode::UnknownSession => "Unknown session".to_string(),
            VerificationErrorCode::Nonce => "Redirect URI nonce incorrect or missing".to_string(),
            VerificationErrorCode::SessionState => "Session is not in the required state".to_string(),
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            VerificationErrorCode::ServerError => StatusCode::INTERNAL_SERVER_ERROR,
            VerificationErrorCode::UnknownSession => StatusCode::NOT_FOUND,
            VerificationErrorCode::InvalidRequest | VerificationErrorCode::SessionState => StatusCode::BAD_REQUEST,

            // See the other comment on `StatusCode::UNAUTHORIZED`
            VerificationErrorCode::Nonce => StatusCode::UNAUTHORIZED,
        }
    }
}

impl From<VerificationError> for VerificationErrorCode {
    fn from(err: VerificationError) -> Self {
        match err {
            VerificationError::Session(SessionError::Expired)
            | VerificationError::Session(SessionError::UnexpectedState)
            | VerificationError::SessionNotDone => VerificationErrorCode::SessionState,
            VerificationError::Session(SessionError::UnknownSession(_)) => VerificationErrorCode::UnknownSession,
            VerificationError::Session(SessionError::SessionStore(_)) | VerificationError::UrlEncoding(_) => {
                VerificationErrorCode::ServerError
            }
            VerificationError::UnknownUseCase(_)
            | VerificationError::ReturnUrlConfigurationMismatch
            | VerificationError::NoItemsRequests
            | VerificationError::MissingSAN
            | VerificationError::Certificate(_) => VerificationErrorCode::InvalidRequest,
            VerificationError::RedirectUriNonceMismatch(_) | VerificationError::RedirectUriNonceMissing => {
                VerificationErrorCode::Nonce
            }
        }
    }
}

impl From<VerificationError> for HttpJsonError<VerificationErrorCode> {
    fn from(value: VerificationError) -> Self {
        HttpJsonError::from_error(value)
    }
}

/// https://www.rfc-editor.org/rfc/rfc6750.html#section-3.1
#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthBearerErrorCode {
    InvalidRequest,
    InvalidToken,
    InsufficientScope,
}

/// Error codes that the wallet sends to the verifier when it encounters an error or rejects the session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VpAuthorizationErrorCode {
    VpFormatsNotSupported,
    InvalidPresentationDefinitionUri,
    InvalidPresentationDefinitionReference,
    InvalidRequestUriMethod,

    #[serde(untagged)]
    AuthorizationError(AuthorizationErrorCode),
}

/// https://www.rfc-editor.org/rfc/rfc6749.html#section-4.1.2.1
#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationErrorCode {
    InvalidRequest,
    UnauthorizedClient,
    AccessDenied,
    UnsupportedResponseType,
    InvalidScope,
    ServerError,
    TemporarilyUnavailable,
}

#[cfg(feature = "axum")]
mod axum {
    use std::fmt::Debug;

    use axum::{
        response::{IntoResponse, Response},
        Json,
    };
    use serde::Serialize;
    use tracing::warn;

    use super::{DisclosureErrorResponse, ErrorResponse, ErrorStatusCode};

    impl<T> IntoResponse for ErrorResponse<T>
    where
        T: ErrorStatusCode + Serialize + Debug,
    {
        fn into_response(self) -> Response {
            warn!("Responding with error body: {:?}", &self);

            (self.error.status_code(), Json(self)).into_response()
        }
    }

    impl<T> IntoResponse for DisclosureErrorResponse<T>
    where
        T: ErrorStatusCode + Serialize + Debug,
    {
        fn into_response(self) -> Response {
            warn!("Responding with error body: {:?}", &self);

            (self.error_response.error.status_code(), Json(self)).into_response()
        }
    }
}
