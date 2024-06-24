use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

use crate::{
    issuer::{CredentialRequestError, IssuanceError, TokenRequestError},
    verifier::{GetAuthRequestError, PostAuthResponseError, SessionError, VerificationError},
};

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ErrorResponse<T> {
    pub error: T,
    pub error_description: Option<String>,
    pub error_uri: Option<Url>,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

                GetAuthRequestError::InvalidEphemeralId(_)
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
            GetRequestErrorCode::ExpiredEphemeralId => StatusCode::FORBIDDEN,
            _ => StatusCode::BAD_REQUEST,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationErrorCode {
    InvalidRequest,
    ExpiredSession,
    SessionUnknown,
    ServerError,
}

impl From<VerificationError> for ErrorResponse<VerificationErrorCode> {
    fn from(err: VerificationError) -> Self {
        let description = err.to_string();
        ErrorResponse {
            error: match err {
                VerificationError::Session(SessionError::Expired) => VerificationErrorCode::ExpiredSession,
                VerificationError::Session(SessionError::UnknownSession(_)) => VerificationErrorCode::SessionUnknown,
                VerificationError::Session(SessionError::SessionStore(_)) | VerificationError::UrlEncoding(_) => {
                    VerificationErrorCode::ServerError
                }
                VerificationError::Session(SessionError::UnexpectedState) => VerificationErrorCode::InvalidRequest,
                VerificationError::UnknownUseCase(_)
                | VerificationError::ReturnUrlConfigurationMismatch
                | VerificationError::NoItemsRequests
                | VerificationError::SessionNotDone
                | VerificationError::RedirectUriNonceMismatch(_)
                | VerificationError::RedirectUriNonceMissing
                | VerificationError::MissingSAN
                | VerificationError::Certificate(_) => VerificationErrorCode::InvalidRequest,
            },
            error_description: Some(description),
            error_uri: None,
        }
    }
}

impl ErrorStatusCode for VerificationErrorCode {
    fn status_code(&self) -> StatusCode {
        match self {
            VerificationErrorCode::ServerError => StatusCode::INTERNAL_SERVER_ERROR,
            VerificationErrorCode::SessionUnknown | VerificationErrorCode::ExpiredSession => StatusCode::NOT_FOUND,
            VerificationErrorCode::InvalidRequest => StatusCode::BAD_REQUEST,
        }
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
