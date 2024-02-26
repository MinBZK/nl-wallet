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
                CredentialRequestError::IssuanceError(err) => match err {
                    IssuanceError::UnexpectedState => CredentialErrorType::InvalidRequest,
                    IssuanceError::UnknownSession(_) => CredentialErrorType::InvalidRequest,
                    IssuanceError::SessionStore(_) => CredentialErrorType::ServerError,
                    IssuanceError::DpopInvalid(_) => CredentialErrorType::InvalidRequest,
                },
                CredentialRequestError::Unauthorized => CredentialErrorType::InvalidToken,
                CredentialRequestError::MalformedToken => CredentialErrorType::InvalidToken,
                CredentialRequestError::UseBatchIssuance => CredentialErrorType::InvalidRequest,
                CredentialRequestError::UnsupportedCredentialFormat(_) => {
                    CredentialErrorType::UnsupportedCredentialFormat
                }
                CredentialRequestError::MissingJwk => CredentialErrorType::InvalidProof,
                CredentialRequestError::IncorrectNonce => CredentialErrorType::InvalidProof,
                CredentialRequestError::UnsupportedJwtAlgorithm { expected: _, found: _ } => {
                    CredentialErrorType::InvalidProof
                }
                CredentialRequestError::JwtDecodingFailed(_) => CredentialErrorType::InvalidProof,
                CredentialRequestError::JwkConversion(_) => CredentialErrorType::InvalidProof,
                CredentialRequestError::CoseKeyConversion(_) => CredentialErrorType::ServerError,
                CredentialRequestError::MissingPrivateKey(_) => CredentialErrorType::ServerError,
                CredentialRequestError::AttestationSigning(_) => CredentialErrorType::ServerError,
                CredentialRequestError::CborSerialization(_) => CredentialErrorType::ServerError,
                CredentialRequestError::JsonSerialization(_) => CredentialErrorType::ServerError,
                CredentialRequestError::DoctypeMismatch => CredentialErrorType::InvalidCredentialRequest,
                CredentialRequestError::MissingCredentialRequestPoP => CredentialErrorType::InvalidProof,
                CredentialRequestError::DoctypeNotOffered(_) => CredentialErrorType::InvalidCredentialRequest,
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
                TokenRequestError::IssuanceError(err) => match err {
                    IssuanceError::UnexpectedState => TokenErrorType::InvalidRequest,
                    IssuanceError::UnknownSession(_) => TokenErrorType::InvalidRequest,
                    IssuanceError::SessionStore(_) => TokenErrorType::ServerError,
                    IssuanceError::DpopInvalid(_) => TokenErrorType::InvalidRequest,
                },
                TokenRequestError::UnsupportedTokenRequestType => TokenErrorType::UnsupportedGrantType,
                TokenRequestError::AttributeService(_) => TokenErrorType::ServerError,
                TokenRequestError::NoAttributes => TokenErrorType::ServerError,
            },
            error_description: Some(description),
            error_uri: None,
        }
    }
}
