use reqwest::StatusCode;
use serde::Deserialize;
use serde::Serialize;
use serde_with::skip_serializing_none;
use url::Url;

use http_utils::error::HttpJsonError;
use http_utils::error::HttpJsonErrorType;
use http_utils::urls::BaseUrl;

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

/// See
/// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0-13.html#name-credential-error-response>
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
                | CredentialRequestError::CredentialSigning(_)
                | CredentialRequestError::CborSerialization(_)
                | CredentialRequestError::Jwt(_)
                | CredentialRequestError::JsonSerialization(_)
                | CredentialRequestError::WteTracking(_) => CredentialErrorCode::ServerError,

                CredentialRequestError::IssuanceError(_)
                | CredentialRequestError::UseBatchIssuance
                | CredentialRequestError::MissingWte
                | CredentialRequestError::WteAlreadyUsed
                | CredentialRequestError::MissingPoa
                | CredentialRequestError::CredentialTypeMismatch
                | CredentialRequestError::CredentialTypeNotOffered(_) => CredentialErrorCode::InvalidCredentialRequest,

                CredentialRequestError::Unauthorized | CredentialRequestError::MalformedToken => {
                    CredentialErrorCode::InvalidToken
                }

                CredentialRequestError::UnsupportedJwtAlgorithm { .. }
                | CredentialRequestError::MissingJwk
                | CredentialRequestError::IncorrectNonce
                | CredentialRequestError::JwtDecodingFailed(_)
                | CredentialRequestError::JwkConversion(_)
                | CredentialRequestError::MissingCredentialRequestPoP
                | CredentialRequestError::PoaVerification(_) => CredentialErrorCode::InvalidProof,

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

/// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0-13.html#section-6.3>
/// and <https://www.rfc-editor.org/rfc/rfc6749.html#section-5.2>.
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
                | TokenRequestError::AttributeService(_)
                | TokenRequestError::CredentialTypeNotOffered(_)
                | TokenRequestError::AttributeConversion(_)
                | TokenRequestError::CredentialPayload(_)
                | TokenRequestError::Certificate(_) => TokenErrorCode::ServerError,
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
    CancelledSession,
    UnknownSession,

    ServerError,
}

impl From<GetAuthRequestError> for ErrorResponse<GetRequestErrorCode> {
    fn from(err: GetAuthRequestError) -> Self {
        let description = err.to_string();
        ErrorResponse {
            error: match err {
                GetAuthRequestError::ExpiredEphemeralId(_) => GetRequestErrorCode::ExpiredEphemeralId,
                GetAuthRequestError::Session(SessionError::UnexpectedState(SessionStatus::Expired)) => {
                    GetRequestErrorCode::ExpiredSession
                }
                GetAuthRequestError::Session(SessionError::UnexpectedState(SessionStatus::Cancelled)) => {
                    GetRequestErrorCode::CancelledSession
                }
                GetAuthRequestError::Session(SessionError::UnknownSession(_)) => GetRequestErrorCode::UnknownSession,
                GetAuthRequestError::EncryptionKey(_)
                | GetAuthRequestError::AuthRequest(_)
                | GetAuthRequestError::Jwt(_)
                | GetAuthRequestError::ReturnUrlConfigurationMismatch
                | GetAuthRequestError::UnknownUseCase(_)
                | GetAuthRequestError::NoAttributesToRequest(_)
                | GetAuthRequestError::Session(SessionError::SessionStore(_)) => GetRequestErrorCode::ServerError,
                GetAuthRequestError::QueryParametersMissing
                | GetAuthRequestError::QueryParametersDeserialization(_)
                | GetAuthRequestError::InvalidEphemeralId(_)
                | GetAuthRequestError::Session(SessionError::UnexpectedState(_)) => GetRequestErrorCode::InvalidRequest,
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
            GetRequestErrorCode::ExpiredSession
            | GetRequestErrorCode::CancelledSession
            | GetRequestErrorCode::UnknownSession => StatusCode::NOT_FOUND,
            GetRequestErrorCode::InvalidRequest => StatusCode::BAD_REQUEST,

            // Per RFC 7235 we MUST include a `WWW-Authenticate` HTTP header with this, but we can't do that
            // conveniently here. It seems this header is often skipped, and we use it internally here, we skip it too.
            GetRequestErrorCode::ExpiredEphemeralId => StatusCode::UNAUTHORIZED,
        }
    }
}

/// <https://openid.net/specs/openid-4-verifiable-presentations-1_0-20.html#name-error-response>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PostAuthResponseErrorCode {
    InvalidRequest,
    ExpiredSession,
    CancelledSession,
    UnknownSession,

    ServerError,

    /// An NL Wallet specific error code, meaning the following: in a disclosure based issuance session,
    /// the issuer found no attestations to issue.
    NoIssuableAttestations,
}

impl From<PostAuthResponseError> for ErrorResponse<PostAuthResponseErrorCode> {
    fn from(err: PostAuthResponseError) -> Self {
        let description = err.to_string();
        ErrorResponse {
            error: match err {
                PostAuthResponseError::Session(SessionError::UnexpectedState(SessionStatus::Expired)) => {
                    PostAuthResponseErrorCode::ExpiredSession
                }
                PostAuthResponseError::Session(SessionError::UnexpectedState(SessionStatus::Cancelled)) => {
                    PostAuthResponseErrorCode::CancelledSession
                }
                PostAuthResponseError::Session(SessionError::SessionStore(_))
                | PostAuthResponseError::ResponseEncoding(_) => PostAuthResponseErrorCode::ServerError,
                PostAuthResponseError::Session(SessionError::UnknownSession(_)) => {
                    PostAuthResponseErrorCode::UnknownSession
                }
                PostAuthResponseError::AuthResponse(_)
                | PostAuthResponseError::Session(SessionError::UnexpectedState(_)) => {
                    PostAuthResponseErrorCode::InvalidRequest
                }
                PostAuthResponseError::HandlingDisclosureResult(err) => err.as_ref().to_error_code(),
            },
            error_description: Some(description),
            error_uri: None,
        }
    }
}

impl ErrorStatusCode for PostAuthResponseErrorCode {
    fn status_code(&self) -> reqwest::StatusCode {
        match self {
            PostAuthResponseErrorCode::ExpiredSession
            | PostAuthResponseErrorCode::CancelledSession
            | PostAuthResponseErrorCode::UnknownSession
            | PostAuthResponseErrorCode::NoIssuableAttestations => StatusCode::NOT_FOUND,
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

// The RP error types and `VerificationErrorCode` are handled differently from the errors above:
// instead of returning them as an `ErrorResponse`, they are returned as a `HttpJsonErrorBody`.
// This is because the endpoints that return these errors are not part of a protocol from the
// OAuth/OpenID family, which uses `ErrorResponse`, but instead they are specific to this implementation.

/// Error codes sent to the Relying Party when an error occurs when handling their request.
#[derive(Debug, Clone, Copy, strum::EnumString, strum::Display)]
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
            NewSessionError::NoItemsRequests
            | NewSessionError::UnknownUseCase(_)
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

/// https://www.rfc-editor.org/rfc/rfc6750.html#section-3.1
#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthBearerErrorCode {
    InvalidRequest,
    InvalidToken,
    InsufficientScope,
}

/// Error codes that the wallet sends to the verifier when it encounters an error or rejects the session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationErrorCode {
    InvalidRequest,
    UnauthorizedClient,
    AccessDenied,
    UnsupportedResponseType,
    InvalidScope,
    ServerError,
    TemporarilyUnavailable,
    #[serde(untagged)]
    Other(String),
}

#[cfg(feature = "axum")]
mod axum {
    use std::fmt::Debug;

    use axum::response::IntoResponse;
    use axum::response::Response;
    use axum::Json;
    use serde::Serialize;
    use tracing::warn;

    use super::DisclosureErrorResponse;
    use super::ErrorResponse;
    use super::ErrorStatusCode;

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
