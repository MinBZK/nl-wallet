use std::collections::HashSet;

use futures::future::try_join_all;
use jsonwebtoken::{Algorithm, Header, Validation};
use nl_wallet_mdoc::utils::keys::{KeyFactory, MdocEcdsaKey};
use p256::ecdsa::VerifyingKey;
use serde::{Deserialize, Serialize};
use wallet_common::keys::SecureEcdsaKey;

use crate::{
    jwk_from_p256, jwk_to_p256,
    jwt::{EcdsaDecodingKey, Jwt, StandardJwtClaims},
    Error, Format, Result,
};

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
    pub proof: CredentialRequestProof, // this is OPTIONAL per the spec, but we require it
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
pub struct CredentialResponse {
    pub format: Format,
    pub credential: serde_json::Value, // TODO maybe make this typed
}

// https://openid.github.io/OpenID4VCI/openid-4-verifiable-credential-issuance-wg-draft.html#section-7.2.1.1
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CredentialRequestProofJwtPayload {
    /// Required: `iss`, `aud`, `iat`
    #[serde(flatten)]
    pub jwt_claims: StandardJwtClaims,
    pub nonce: String,
}

const OPENID4VCI_VC_POP_JWT_TYPE: &str = "openid4vci-proof+jwt";

impl CredentialRequestProof {
    pub async fn new_multiple<'a, K: MdocEcdsaKey + Sync>(
        nonce: String,
        wallet_name: String,
        audience: String,
        number_of_keys: u64,
        key_factory: &'a impl KeyFactory<'a, Key = K>,
    ) -> Result<Vec<(K, CredentialRequestProof)>> {
        // TODO: extend key factory so that it can do this in a single instruction
        let keys = key_factory.generate_new_multiple(number_of_keys).await.unwrap(); // TODO
        try_join_all(keys.into_iter().map(|privkey| async {
            CredentialRequestProof::new(&privkey, nonce.clone(), wallet_name.clone(), audience.clone())
                .await
                .map(|jwt| (privkey, jwt))
        }))
        .await
    }

    pub async fn new(
        private_key: &impl SecureEcdsaKey,
        nonce: String,
        wallet_name: String,
        audience: String,
    ) -> Result<Self> {
        let header = Header {
            typ: Some(OPENID4VCI_VC_POP_JWT_TYPE.to_string()),
            alg: Algorithm::ES256,
            jwk: Some(jwk_from_p256(
                &private_key
                    .verifying_key()
                    .await
                    .map_err(|e| Error::VerifyingKeyFromPrivateKey(e.into()))?,
            )?),
            ..Default::default()
        };

        let payload = CredentialRequestProofJwtPayload {
            jwt_claims: StandardJwtClaims {
                issuer: Some(wallet_name),
                audience: Some(audience),
                issued_at: Some(jsonwebtoken::get_current_timestamp() as i64),
                ..Default::default()
            },
            nonce,
        };

        let jwt = Jwt::sign(&payload, &header, private_key).await?;

        Ok(CredentialRequestProof::Jwt { jwt })
    }

    pub fn verify(&self, nonce: String, wallet_name: String, audience: String) -> Result<VerifyingKey> {
        let jwt = match self {
            CredentialRequestProof::Jwt { jwt } => jwt,
        };
        let header = jsonwebtoken::decode_header(&jwt.0)?;
        let verifying_key = jwk_to_p256(&header.jwk.ok_or(Error::MissingJwk)?)?;

        let mut validation_options = Validation::new(Algorithm::ES256);
        validation_options.required_spec_claims = HashSet::from(["iss".to_string(), "aud".to_string()]);
        validation_options.set_issuer(&[wallet_name]);
        validation_options.set_audience(&[audience]);
        let token_data = jsonwebtoken::decode::<CredentialRequestProofJwtPayload>(
            &jwt.0,
            &EcdsaDecodingKey::from(verifying_key).0,
            &validation_options,
        )?;

        if token_data.header.typ != Some(OPENID4VCI_VC_POP_JWT_TYPE.to_string()) {
            return Err(Error::UnsupportedJwtAlgorithm {
                expected: OPENID4VCI_VC_POP_JWT_TYPE.to_string(),
                found: token_data.header.typ.unwrap_or_default(),
            });
        }
        if token_data.claims.nonce != nonce {
            return Err(Error::IncorrectNonce);
        }

        Ok(verifying_key)
    }
}
