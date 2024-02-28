use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

use crate::{
    credential::CredentialErrorType,
    issuer::{CredentialRequestError, IssuanceError, TokenRequestError},
    token::TokenErrorType,
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

impl From<CredentialRequestError> for ErrorResponse<CredentialErrorType> {
    fn from(err: CredentialRequestError) -> ErrorResponse<CredentialErrorType> {
        let description = err.to_string();
        ErrorResponse {
            error: match err {
                CredentialRequestError::IssuanceError(IssuanceError::SessionStore(_))
                | CredentialRequestError::CoseKeyConversion(_)
                | CredentialRequestError::MissingPrivateKey(_)
                | CredentialRequestError::AttestationSigning(_)
                | CredentialRequestError::CborSerialization(_)
                | CredentialRequestError::JsonSerialization(_) => CredentialErrorType::ServerError,
                CredentialRequestError::IssuanceError(_) | CredentialRequestError::UseBatchIssuance => {
                    CredentialErrorType::InvalidRequest
                }
                CredentialRequestError::Unauthorized | CredentialRequestError::MalformedToken => {
                    CredentialErrorType::InvalidToken
                }
                CredentialRequestError::UnsupportedJwtAlgorithm { .. }
                | CredentialRequestError::MissingJwk
                | CredentialRequestError::IncorrectNonce
                | CredentialRequestError::JwtDecodingFailed(_)
                | CredentialRequestError::JwkConversion(_)
                | CredentialRequestError::MissingCredentialRequestPoP => CredentialErrorType::InvalidProof,
                CredentialRequestError::DoctypeMismatch | CredentialRequestError::DoctypeNotOffered(_) => {
                    CredentialErrorType::InvalidCredentialRequest
                }
                CredentialRequestError::UnsupportedCredentialFormat(_) => {
                    CredentialErrorType::UnsupportedCredentialFormat
                }
            },
            error_description: Some(description),
            error_uri: None,
        }
    }
}

impl From<TokenRequestError> for ErrorResponse<TokenErrorType> {
    fn from(err: TokenRequestError) -> Self {
        let description = err.to_string();
        ErrorResponse {
            error: match err {
                TokenRequestError::IssuanceError(IssuanceError::SessionStore(_))
                | TokenRequestError::AttributeService(_)
                | TokenRequestError::NoAttributes => TokenErrorType::ServerError,
                TokenRequestError::IssuanceError(_) => TokenErrorType::InvalidRequest,
                TokenRequestError::UnsupportedTokenRequestType => TokenErrorType::UnsupportedGrantType,
            },
            error_description: Some(description),
            error_uri: None,
        }
    }
}
