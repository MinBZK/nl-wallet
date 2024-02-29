use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

use crate::{
    credential::CredentialErrorCode,
    issuer::{CredentialRequestError, IssuanceError, TokenRequestError},
    token::TokenErrorCode,
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
                | TokenRequestError::AttributeService(_)
                | TokenRequestError::NoAttributes => TokenErrorCode::ServerError,
                TokenRequestError::IssuanceError(_) => TokenErrorCode::InvalidRequest,
                TokenRequestError::UnsupportedTokenRequestType => TokenErrorCode::UnsupportedGrantType,
            },
            error_description: Some(description),
            error_uri: None,
        }
    }
}
