use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

use crate::{
    credential::CredentialErrorCode,
    issuer::{CredentialRequestError, IssuanceError, TokenRequestError},
    token::TokenErrorCode,
    verifier::{
        GetAuthRequestError, GetRequestErrorCode, PostAuthResponseError, PostAuthResponseErrorCode, SessionError,
        VerificationError, VerificationErrorCode,
    },
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
            _ => StatusCode::BAD_REQUEST,
        }
    }
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
                PostAuthResponseError::UserError(_) => panic!("UserError should never be sent as response to user"),
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
            _ => StatusCode::BAD_REQUEST,
        }
    }
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
