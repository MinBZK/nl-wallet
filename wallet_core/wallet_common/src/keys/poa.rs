use futures::future::try_join_all;
use jsonwebtoken::{jwk::Jwk, Algorithm, Header};
use serde::{Deserialize, Serialize};

use crate::{
    jwt::{jwk_from_p256, JsonJwt, JwkConversionError, Jwt, JwtError, JwtPopClaims},
    nonempty::NonEmpty,
};

use super::EcdsaKey;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoaPayload {
    #[serde(flatten)]
    pub payload: JwtPopClaims,
    pub jwks: Vec<Jwk>,
}

/// A Proof of Association, asserting that a set of credential public keys are managed by a single WSCD.
pub type Poa = JsonJwt<PoaPayload>;

pub static POA_JWT_TYP: &str = "poa+jwt";

#[derive(Debug, thiserror::Error)]
pub enum PoaError {
    #[error("error converting key to JWK: {0}")]
    Jwk(#[from] JwkConversionError),
    #[error("JWT bulk signing error: {0}")]
    Signing(#[from] JwtError),
    #[error("error obtaining verifying key from signing key: {0}")]
    VerifyingKey(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("cannot associate {0} keys, a minimum of 2 is required")]
    InsufficientKeys(usize),
}

impl Poa {
    pub async fn new<K: EcdsaKey>(keys: Vec<&K>, payload: JwtPopClaims) -> Result<Poa, PoaError> {
        if keys.len() < 2 {
            return Err(PoaError::InsufficientKeys(keys.len()));
        }

        let payload = PoaPayload {
            payload,
            jwks: try_join_all(keys.iter().map(|privkey| async {
                jwk_from_p256(
                    &privkey
                        .verifying_key()
                        .await
                        .map_err(|e| PoaError::VerifyingKey(Box::new(e)))?,
                )
                .map_err(PoaError::Jwk)
            }))
            .await?,
        };
        let header = Header {
            typ: Some(POA_JWT_TYP.to_string()),
            ..Header::new(Algorithm::ES256)
        };

        let jwts: NonEmpty<_> = try_join_all(keys.iter().map(|key| Jwt::sign(&payload, &header, *key)))
            .await?
            .try_into()
            .unwrap(); // This came from `keys` which is `NonEmpty`.

        // This unwrap() is safe because we correctly constructed the `jwts` above.
        Ok(jwts.try_into().unwrap())
    }
}

#[cfg(all(test, feature = "software_keys"))]
mod tests {
    use crate::{
        jwt::{validations, Jwt, JwtPopClaims},
        keys::{
            poa::{Poa, PoaPayload},
            software::SoftwareEcdsaKey,
            EcdsaKey,
        },
    };

    #[tokio::test]
    async fn test_poa() {
        let key1 = SoftwareEcdsaKey::new_random("key1".to_string());
        let key2 = SoftwareEcdsaKey::new_random("key2".to_string());
        let iss = "iss".to_string();
        let aud = "aud".to_string();

        let poa = Poa::new(vec![&key1, &key2], JwtPopClaims::new(None, iss.clone(), aud.clone()))
            .await
            .unwrap();

        let jwts: Vec<Jwt<PoaPayload>> = poa.into();

        let mut validations = validations();
        validations.set_audience(&[aud]);
        validations.set_issuer(&[iss]);

        for (jwt, key) in jwts.into_iter().zip([key1, key2]) {
            jwt.parse_and_verify(&(&key.verifying_key().await.unwrap()).into(), &validations)
                .unwrap();
        }
    }
}
