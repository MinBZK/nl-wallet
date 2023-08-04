use anyhow::Result;
use async_trait::async_trait;
use josekit::{jwe, jwk::Jwk};
use openid::{
    biscuit::{jwa::SignatureAlgorithm, ClaimsSet, CompactJson, CompactPart, ValidationOptions},
    error::ClientError,
    Client, Empty, Jws, OAuth2Error,
};
use reqwest::header::ACCEPT;
use serde_json::Value;
use std::collections::HashMap;
use tracing::debug;

// TODO replace with deserializable struct once we're certain about the claim(s)
pub type AttributeMap = HashMap<String, Value>;

#[async_trait]
pub trait UserinfoExtensions {
    fn jwe_decrypt_claims<C, H>(&self, jwe: &str, private_key: &Jwk) -> Result<Jws<ClaimsSet<C>, H>>
    where
        ClaimsSet<C>: CompactPart,
        H: CompactJson;
    fn extract_bsn(&self, userinfo_token: &Jws<ClaimsSet<AttributeMap>, Empty>) -> Result<Option<String>>;
    async fn invoke_userinfo_endpoint(&self, access_token: &str) -> Result<Value, ClientError>;
}

#[async_trait]
impl UserinfoExtensions for Client {
    fn jwe_decrypt_claims<C, H>(&self, jwe: &str, private_key: &Jwk) -> Result<Jws<ClaimsSet<C>, H>>
    where
        ClaimsSet<C>: CompactPart,
        H: CompactJson,
    {
        let alg = jwe::RSA_OAEP;
        // TODO An optimalization might be to inject the decrypter instead of the private key, to avoid creating the
        // decrypter on every invocation.
        let decrypter = alg.decrypter_from_jwk(private_key)?;
        let (payload, _header) = jwe::deserialize_compact(jwe, &decrypter)?;
        let decrypted = String::from_utf8(payload)?;

        let decrypted_claims = openid::biscuit::JWT::<C, H>::new_encoded(&decrypted).decode_with_jwks(
            self.jwks.as_ref().expect("NO JWKS found for client - 2"),
            Some(SignatureAlgorithm::RS256),
        )?;
        Ok(decrypted_claims)
    }

    fn extract_bsn(&self, userinfo_token: &Jws<ClaimsSet<AttributeMap>, Empty>) -> Result<Option<String>> {
        debug!("userinfo_token: {:?}", userinfo_token);

        let userinfo_payload = userinfo_token.payload()?;

        debug!("Registered Claims: {:?}", userinfo_payload.registered);
        userinfo_payload.registered.validate(ValidationOptions::default())?;

        for (key, value) in userinfo_payload.private.iter() {
            debug!("Private Claim {}: {}", key, value);
        }

        let bsn = userinfo_payload.private.get("uzi_id").and_then(|value| match value {
            Value::String(claim) => Some(claim.to_string()),
            _ => None,
        });
        Ok(bsn)
    }

    async fn invoke_userinfo_endpoint(&self, access_token: &str) -> Result<Value, ClientError> {
        let userinfo_endpoint = self.config().userinfo_endpoint.as_ref().unwrap().clone();
        let userinfo_response = self
            .http_client
            .post(userinfo_endpoint)
            .bearer_auth(access_token)
            .header(ACCEPT, "application/jwt")
            .send()
            .await?;

        let userinfo = Value::String(userinfo_response.text().await.unwrap());

        let error: Result<OAuth2Error, _> = serde_json::from_value(userinfo.clone());

        // TODO check for response state instead, and decode based on that
        if let Ok(error) = error {
            Err(ClientError::from(error))
        } else {
            Ok(userinfo)
        }
    }
}
