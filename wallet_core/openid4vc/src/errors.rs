use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

use nl_wallet_mdoc::{identifiers::AttributeIdentifier, utils::serialization::CborError};
use wallet_common::jwt::JwtError;

use crate::{
    credential::CredentialErrorType,
    dpop::DpopError,
    issuer::{self, CredentialRequestError, TokenRequestError},
    jwk::JwkConversionError,
    token::TokenErrorType,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to get public key: {0}")]
    VerifyingKeyFromPrivateKey(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("DPoP error: {0}")]
    Dpop(#[from] DpopError),
    #[error("failed to convert key from/to JWK format: {0}")]
    JwkConversion(#[from] JwkConversionError),
    #[error("JWT error: {0}")]
    Jwt(#[from] JwtError),
    #[error("http request failed: {0}")]
    Network(#[from] reqwest::Error),
    #[error("missing c_nonce")]
    MissingNonce,
    #[error("CBOR (de)serialization error: {0}")]
    Cbor(#[from] CborError),
    #[error("base64 decoding failed: {0}")]
    Base64Error(#[from] base64::DecodeError),
    #[error("mismatch between issued and expected attributes")]
    IssuedAttributesMismatch(Vec<AttributeIdentifier>),
    #[error("mdoc verification failed: {0}")]
    MdocVerification(#[source] nl_wallet_mdoc::Error),
    #[error("error requesting access token: {0:?}")]
    TokenRequest(Box<ErrorResponse<TokenErrorType>>),
    #[error("error requesting credentials: {0:?}")]
    CredentialRequest(Box<ErrorResponse<CredentialErrorType>>),
    #[error("generating attestation private keys failed: {0}")]
    PrivateKeyGeneration(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("missing issuance session state")]
    MissingIssuanceSessionState,
    #[error("public key contained in mdoc not equal to expected value")]
    PublicKeyMismatch,
    #[error("failed to get mdoc public key: {0}")]
    PublicKeyFromMdoc(#[source] nl_wallet_mdoc::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

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
                    issuer::Error::UnexpectedState => CredentialErrorType::InvalidRequest,
                    issuer::Error::UnknownSession(_) => CredentialErrorType::InvalidRequest,
                    issuer::Error::SessionStore(_) => CredentialErrorType::ServerError,
                    issuer::Error::DpopInvalid(_) => CredentialErrorType::InvalidRequest,
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
                    issuer::Error::UnexpectedState => TokenErrorType::InvalidRequest,
                    issuer::Error::UnknownSession(_) => TokenErrorType::InvalidRequest,
                    issuer::Error::SessionStore(_) => TokenErrorType::ServerError,
                    issuer::Error::DpopInvalid(_) => TokenErrorType::InvalidRequest,
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
