use std::collections::HashMap;

use base64::prelude::*;
use chrono::{serde::ts_seconds, DateTime, Utc};
use futures::future::try_join_all;
use itertools::Itertools;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use url::Url;

use nl_wallet_mdoc::{
    utils::{
        keys::{KeyFactory, MdocEcdsaKey},
        serialization::CborBase64,
    },
    IssuerSigned,
};
use wallet_common::jwt::{Jwt, JwtError};

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
        let keys = key_factory
            .generate_new_multiple(number_of_keys)
            .await
            .map_err(|e| IssuerClientError::PrivateKeyGeneration(Box::new(e)))?;

        let keys_and_jwt_payloads = try_join_all(keys.into_iter().map(|privkey| async {
            let header = jwk_jwt_header(OPENID4VCI_VC_POP_JWT_TYPE, &privkey).await?;
            let payload = CredentialRequestProofJwtPayload {
                nonce: Some(nonce.clone()),
                iss: wallet_client_id.clone(),
                aud: credential_issuer_identifier.to_string(),
                iat: Utc::now(),
            };
            Ok::<_, IssuerClientError>((privkey, (payload, header)))
        }))
        .await?;

        let keys_and_proofs = sign_jwts(keys_and_jwt_payloads, &key_factory)
            .await?
            .into_iter()
            .map(|(key, jwt)| (key, CredentialRequestProof::Jwt { jwt }))
            .collect();

        Ok(keys_and_proofs)
    }
}

/// Bulk-sign the keys and JWT payloads into JWTs.
pub async fn sign_jwts<T: Serialize, K: MdocEcdsaKey>(
    keys_and_messages: Vec<(K, (T, jsonwebtoken::Header))>,
    key_factory: &impl KeyFactory<Key = K>,
) -> Result<Vec<(K, Jwt<T>)>, IssuerClientError> {
    let (keys, to_sign): (Vec<_>, Vec<_>) = keys_and_messages.into_iter().unzip();

    // Construct a Vec containing the strings to be signed with the private keys, i.e. schematically "header.body"
    let messages = to_sign
        .iter()
        .map(|(message, header)| {
            Ok([
                BASE64_URL_SAFE_NO_PAD.encode(serde_json::to_vec(header)?),
                BASE64_URL_SAFE_NO_PAD.encode(serde_json::to_vec(message)?),
            ]
            .join("."))
        })
        .collect::<Result<Vec<_>, JwtError>>()?;

    // Associate the messages to the keys with which they are to be signed, for below
    let keys_messages_map: HashMap<_, _> = keys
        .iter()
        .zip(&messages)
        .map(|(key, msg)| (key.identifier().to_string(), msg.clone()))
        .collect();

    // Have the WP sign our messages. It returns key-signature pairs in a random order.
    let keys_and_sigs = key_factory
        .sign_with_existing_keys(
            messages
                .into_iter()
                .map(|msg| msg.into_bytes())
                .zip(keys.into_iter().map(|key| vec![key]))
                .collect_vec(),
        )
        .await
        .map_err(|err| JwtError::Signing(Box::new(err)))?;

    // For each received key-signature pair, we use the key to lookup the appropriate message
    // from the map constructed above and create the JWT.
    let jwts = keys_and_sigs
        .into_iter()
        .map(|(key, sig)| {
            // The WP will respond only with the keys we fed it above, so we can unwrap
            let msg = keys_messages_map.get(&key.identifier().to_string()).unwrap().clone();
            let jwt = [msg, BASE64_URL_SAFE_NO_PAD.encode(sig.to_vec())].join(".").into();
            (key, jwt)
        })
        .collect();

    Ok(jwts)
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;
    use nl_wallet_mdoc::{
        mock::SoftwareKeyFactory,
        utils::keys::{KeyFactory, MdocEcdsaKey},
    };
    use serde::{Deserialize, Serialize};
    use wallet_common::jwt::{validations, EcdsaDecodingKey};

    #[derive(Serialize, Deserialize, Debug)]
    struct ToyMessage {
        count: usize,
    }

    #[tokio::test]
    async fn test_sign_jwts() {
        bulk_jwt_sign(&SoftwareKeyFactory::default()).await
    }

    fn json_header() -> jsonwebtoken::Header {
        jsonwebtoken::Header {
            alg: jsonwebtoken::Algorithm::ES256,
            ..Default::default()
        }
    }

    pub async fn bulk_jwt_sign<K: MdocEcdsaKey>(key_factory: &impl KeyFactory<Key = K>) {
        // Generate keys to sign with and messages to sign
        let keys = key_factory.generate_new_multiple(4).await.unwrap();
        let keys_and_messages = keys
            .into_iter()
            .enumerate()
            .map(|(count, key)| (key, (ToyMessage { count }, json_header())))
            .collect();

        let jwts = super::sign_jwts(keys_and_messages, key_factory).await.unwrap();

        // Verify JWTs
        futures::stream::iter(jwts) // convert to stream which supports async for_each closures
            .for_each(|(key, jwt)| async move {
                jwt.parse_and_verify(
                    &EcdsaDecodingKey::from(&key.verifying_key().await.unwrap()),
                    &validations(),
                )
                .unwrap();
            })
            .await;
    }
}
