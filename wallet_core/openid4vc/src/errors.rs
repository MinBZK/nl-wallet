use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

use nl_wallet_mdoc::{identifiers::AttributeIdentifier, utils::serialization::CborError};
use wallet_common::jwt::JwtError;

use crate::{credential::CredentialErrorType, jwk::JwkConversionError, token::TokenErrorType};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unsupported JWT algorithm: expected {}, found {}", expected, found.as_ref().unwrap_or(&"<None>".to_string()))]
    UnsupportedJwtAlgorithm { expected: String, found: Option<String> },
    #[error("failed to get public key: {0}")]
    VerifyingKeyFromPrivateKey(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("JWT signing failed: {0}")]
    JwtSigningFailed(#[from] wallet_common::errors::Error),
    #[error("JWT decoding failed: {0}")]
    JwtDecodingFailed(#[from] jsonwebtoken::errors::Error),
    #[error("incorrect DPoP JWT HTTP method")]
    IncorrectDpopMethod,
    #[error("incorrect DPoP JWT url")]
    IncorrectDpopUrl,
    #[error("incorrect DPoP JWT nonce")]
    IncorrectDpopNonce,
    #[error("incorrect DPoP JWT access token hash")]
    IncorrectDpopAccessTokenHash,
    #[error("missing JWK")]
    MissingJwk,
    #[error("incorrect JWK public key")]
    IncorrectJwkPublicKey,
    #[error("failed to convert key from/to JWK format: {0}")]
    JwkConversion(#[from] JwkConversionError),
    #[error("JWT error: {0}")]
    Jwt(#[from] JwtError),
    #[error("URL encoding failed: {0}")]
    UrlEncoding(#[from] serde_urlencoded::ser::Error),
    #[error("http request failed: {0}")]
    Network(#[from] reqwest::Error),
    #[error("missing c_nonce")]
    MissingNonce,
    #[error("JSON (de)serialization failed: {0}")]
    Json(#[from] serde_json::Error),
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
