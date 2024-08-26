use chrono::{serde::ts_seconds, DateTime, Utc};
use futures::future::try_join_all;
use serde::{Deserialize, Serialize};

use nl_wallet_mdoc::{
    utils::{
        keys::{KeyFactory, MdocEcdsaKey},
        serialization::CborBase64,
    },
    IssuerSigned,
};
use wallet_common::{config::wallet_config::BaseUrl, jwt::Jwt, nonempty::NonEmpty};

use crate::{
    issuance_session::IssuanceSessionError,
    jwt::{self, jwk_jwt_header},
    Format,
};

/// https://openid.github.io/OpenID4VCI/openid-4-verifiable-credential-issuance-wg-draft.html#section-8.1.
/// Sent JSON-encoded to `POST /batch_credential`.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CredentialRequests {
    pub credential_requests: NonEmpty<Vec<CredentialRequest>>,
}

/// https://openid.github.io/OpenID4VCI/openid-4-verifiable-credential-issuance-wg-draft.html#section-7.2.
/// Sent JSON-encoded to `POST /credential`.
// TODO: add `wallet_attestation`, `wallet_attestation_pop`, and `proof_of_secure_combination` (PVW-2361, PVW-2362)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CredentialRequest {
    #[serde(flatten)]
    pub format: CredentialRequestFormat,
    pub proof: Option<CredentialRequestProof>,
}

impl CredentialRequest {
    pub fn attestation_type(&self) -> Option<&String> {
        match &self.format {
            CredentialRequestFormat::MsoMdoc { doctype } => doctype.as_ref(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "format", rename_all = "snake_case")]
pub enum CredentialRequestFormat {
    MsoMdoc { doctype: Option<String> },
}

impl From<&CredentialRequestFormat> for Format {
    fn from(value: &CredentialRequestFormat) -> Self {
        match value {
            CredentialRequestFormat::MsoMdoc { doctype: _ } => Format::MsoMdoc,
        }
    }
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

impl CredentialRequestProof {
    pub async fn new_multiple<K: MdocEcdsaKey>(
        nonce: String,
        wallet_client_id: String,
        credential_issuer_identifier: BaseUrl,
        number_of_keys: u64,
        key_factory: impl KeyFactory<Key = K>,
    ) -> Result<Vec<(K, CredentialRequestProof)>, IssuanceSessionError> {
        let keys = key_factory
            .generate_new_multiple(number_of_keys)
            .await
            .map_err(|e| IssuanceSessionError::PrivateKeyGeneration(Box::new(e)))?;

        let payload = CredentialRequestProofJwtPayload {
            nonce: Some(nonce),
            iss: wallet_client_id,
            aud: credential_issuer_identifier.as_ref().to_string(),
            iat: Utc::now(),
        };
        let keys_and_jwt_payloads = try_join_all(keys.into_iter().map(|privkey| async {
            let header = jwk_jwt_header(OPENID4VCI_VC_POP_JWT_TYPE, &privkey).await?;
            let payload = payload.clone();
            Ok::<_, IssuanceSessionError>((privkey, (payload, header)))
        }))
        .await?;

        let keys_and_proofs = jwt::sign_jwts(keys_and_jwt_payloads, &key_factory)
            .await?
            .into_iter()
            .map(|(key, jwt)| (key, CredentialRequestProof::Jwt { jwt }))
            .collect();

        Ok(keys_and_proofs)
    }
}
