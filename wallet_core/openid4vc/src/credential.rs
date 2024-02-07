use chrono::{serde::ts_seconds, DateTime, Utc};
use futures::future::try_join_all;
use nl_wallet_mdoc::{
    utils::{
        keys::{KeyFactory, MdocEcdsaKey},
        serialization::CborBase64,
    },
    IssuerSigned,
};

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use url::Url;
use wallet_common::{jwt::Jwt, keys::SecureEcdsaKey};

use crate::{jwk::jwk_jwt_header, ErrorStatusCode, Format, IssuerClientError};

/// https://openid.github.io/OpenID4VCI/openid-4-verifiable-credential-issuance-wg-draft.html#section-8.1.
/// Sent JSON-encoded to `POST /batch_credential`.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CredentialRequests {
    pub credential_requests: Vec<CredentialRequest>,
}

/// https://openid.github.io/OpenID4VCI/openid-4-verifiable-credential-issuance-wg-draft.html#section-7.2.
/// Sent JSON-encoded to `POST /credential`.
// TODO: add `wallet_attestation`, `wallet_attestation_pop`, and `proof_of_secure_combination`
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CredentialRequest {
    pub format: Format,
    pub doctype: Option<String>,
    pub proof: Option<CredentialRequestProof>,
}

/// https://openid.github.io/OpenID4VCI/openid-4-verifiable-credential-issuance-wg-draft.html#name-credential-endpoint
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "proof_type", rename_all = "snake_case")]
pub enum CredentialRequestProof {
    Jwt { jwt: Jwt<CredentialRequestProofJwtPayload> },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CredentialResponses {
    pub credential_responses: Vec<CredentialResponse>,
}

/// https://openid.github.io/OpenID4VCI/openid-4-verifiable-credential-issuance-wg-draft.html#name-credential-response.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "format", rename_all = "snake_case")]
pub enum CredentialResponse {
    MsoMdoc { credential: CborBase64<IssuerSigned> },
}

// https://openid.github.io/OpenID4VCI/openid-4-verifiable-credential-issuance-wg-draft.html#section-7.2.1.1
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CredentialRequestProofJwtPayload {
    pub iss: String,
    pub aud: String,
    pub nonce: Option<String>,
    #[serde(with = "ts_seconds")]
    pub iat: DateTime<Utc>,
}

pub const OPENID4VCI_VC_POP_JWT_TYPE: &str = "openid4vci-proof+jwt";

/// https://openid.github.io/OpenID4VCI/openid-4-verifiable-credential-issuance-wg-draft.html#name-credential-error-response
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum CredentialErrorType {
    InvalidCredentialRequest,
    UnsupportedCredentialType,
    UnsupportedCredentialFormat,
    InvalidProof,
    InvalidEncryptionParameters,
    ServerError,

    // From https://www.rfc-editor.org/rfc/rfc6750.html#section-3.1
    InvalidRequest,
    InvalidToken,
    InsufficientScope,
}

impl ErrorStatusCode for CredentialErrorType {
    fn status_code(&self) -> reqwest::StatusCode {
        match self {
            CredentialErrorType::InvalidCredentialRequest => StatusCode::BAD_REQUEST,
            CredentialErrorType::UnsupportedCredentialType => StatusCode::BAD_REQUEST,
            CredentialErrorType::UnsupportedCredentialFormat => StatusCode::BAD_REQUEST,
            CredentialErrorType::InvalidProof => StatusCode::BAD_REQUEST,
            CredentialErrorType::InvalidEncryptionParameters => StatusCode::BAD_REQUEST,
            CredentialErrorType::ServerError => StatusCode::INTERNAL_SERVER_ERROR,
            CredentialErrorType::InvalidRequest => StatusCode::BAD_REQUEST,
            CredentialErrorType::InvalidToken => StatusCode::UNAUTHORIZED,
            CredentialErrorType::InsufficientScope => StatusCode::FORBIDDEN,
        }
    }
}

impl CredentialRequestProof {
    pub async fn new_multiple<K: MdocEcdsaKey>(
        nonce: String,
        wallet_client_id: String,
        credential_issuer_identifier: &Url,
        number_of_keys: u64,
        key_factory: impl KeyFactory<Key = K>,
    ) -> Result<Vec<(K, CredentialRequestProof)>, IssuerClientError> {
        // TODO: extend key factory so that it can do this in a single instruction
        let keys = key_factory
            .generate_new_multiple(number_of_keys)
            .await
            .map_err(|e| IssuerClientError::PrivateKeyGeneration(Box::new(e)))?;
        try_join_all(keys.into_iter().map(|privkey| async {
            CredentialRequestProof::new(
                &privkey,
                nonce.clone(),
                wallet_client_id.clone(),
                credential_issuer_identifier,
            )
            .await
            .map(|jwt| (privkey, jwt))
        }))
        .await
    }

    pub async fn new(
        private_key: &impl SecureEcdsaKey,
        nonce: String,
        wallet_client_id: String,
        credential_issuer_identifier: &Url,
    ) -> Result<Self, IssuerClientError> {
        let header = jwk_jwt_header(OPENID4VCI_VC_POP_JWT_TYPE, private_key).await?;

        let payload = CredentialRequestProofJwtPayload {
            nonce: Some(nonce),
            iss: wallet_client_id,
            aud: credential_issuer_identifier.to_string(),
            iat: Utc::now(),
        };

        let jwt = Jwt::sign(&payload, &header, private_key).await?;

        Ok(CredentialRequestProof::Jwt { jwt })
    }
}
