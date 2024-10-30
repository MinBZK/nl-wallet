use std::collections::HashSet;

use futures::future::try_join_all;
use jsonwebtoken::{jwk::Jwk, Algorithm, Header};
use nutype::nutype;
use p256::ecdsa::VerifyingKey;
use serde::{Deserialize, Serialize};

use crate::{
    jwt::{jwk_from_p256, jwk_to_p256, validations, JsonJwt, JwkConversionError, Jwt, JwtError, JwtPopClaims},
    nonempty::NonEmpty,
};

use super::EcdsaKey;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoaPayload {
    #[serde(flatten)]
    pub payload: JwtPopClaims,
    pub jwks: VecAtLeastTwo<Jwk>,
}

#[nutype(derive(Debug, Clone, Serialize, Deserialize, TryFrom, AsRef), validate(predicate = |vec| vec.len() >= 2))]
pub struct VecAtLeastTwo<T>(Vec<T>);

/// A Proof of Association, asserting that a set of credential public keys are managed by a single WSCD.
pub type Poa = JsonJwt<PoaPayload>;

pub static POA_JWT_TYP: &str = "poa+jwt";

#[derive(Debug, thiserror::Error)]
pub enum PoaError {
    #[error("error converting key from/to JWK: {0}")]
    Jwk(#[from] JwkConversionError),
    #[error("JWT bulk signing error: {0}")]
    Signing(#[from] JwtError),
    #[error("error obtaining verifying key from signing key: {0}")]
    VerifyingKey(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
}

#[derive(Debug, thiserror::Error)]
pub enum PoaVerificationError {
    #[error("JWT verification error: {0}")]
    Jwt(#[from] JwtError),
    #[error("unexpected amount of signatures in PoA: expected {expected}, found {found}")]
    UnexpectedSignatureCount { expected: usize, found: usize },
    #[error("unexpected amount of keys in PoA: expected {expected}, found {found}")]
    UnexpectedKeyCount { expected: usize, found: usize },
    #[error("incorrect nonce")]
    IncorrectNonce,
    #[error("error converting key from/to JWK: {0}")]
    Jwk(#[from] JwkConversionError),
    #[error("typ field of PoA header had unexpected value: expected 'Some({POA_JWT_TYP})', found '{0:?}'")]
    IncorrectTyp(Option<String>),
    #[error("key missing in PoA: {0:?}")]
    MissingKey(Box<Jwk>),
}

impl Poa {
    pub async fn new<K: EcdsaKey>(keys: VecAtLeastTwo<&K>, payload: JwtPopClaims) -> Result<Poa, PoaError> {
        let payload = PoaPayload {
            payload,
            jwks: try_join_all(keys.as_ref().iter().map(|privkey| async {
                jwk_from_p256(
                    &privkey
                        .verifying_key()
                        .await
                        .map_err(|e| PoaError::VerifyingKey(Box::new(e)))?,
                )
                .map_err(PoaError::Jwk)
            }))
            .await?
            .try_into()
            .unwrap(), // our iterable is a VecAtLeastTwo
        };
        let header = Header {
            typ: Some(POA_JWT_TYP.to_string()),
            ..Header::new(Algorithm::ES256)
        };

        let jwts: NonEmpty<_> = try_join_all(keys.as_ref().iter().map(|key| Jwt::sign(&payload, &header, *key)))
            .await?
            .try_into()
            .unwrap(); // our iterable is a `VecAtLeastTwo`

        // This unwrap() is safe because we correctly constructed the `jwts` above.
        Ok(jwts.try_into().unwrap())
    }

    /// Verify the PoA, checking that:
    ///
    /// - all `expected_keys` are in the PoA (and no other keys). The keys may be passed in any order.
    /// - all signatures are valid against all keys in the PoA, and the order of the JWKs in the payload
    ///   corresponds to the order of the signatures.
    /// - the `aud`, `nonce` and `iss` fields in the payload have the expected values.
    pub fn verify(
        self,
        expected_keys: &[VerifyingKey],
        expected_aud: &str,
        expected_iss: &str,
        expected_nonce: &str,
    ) -> Result<(), PoaVerificationError> {
        let jwts: Vec<Jwt<_>> = self.into();

        if jwts.len() != expected_keys.len() {
            return Err(PoaVerificationError::UnexpectedSignatureCount {
                expected: expected_keys.len(),
                found: jwts.len(),
            });
        }

        // Some checks on the payload of the JWTs. Since the JWTs came from a `JsonJwt`, we know that the
        // payloads of all of them are equal to one another, so we can suffice with checking the first one.
        // We may use `unwrap()` because of the use of `NonEmpty` in `JsonJwtSignatures`, and we may use
        // `dangerous_parse_unverified()` because we actually validate all JWTs below.
        let (_, payload) = jwts.first().unwrap().dangerous_parse_unverified()?;
        if jwts.len() != payload.jwks.as_ref().len() {
            return Err(PoaVerificationError::UnexpectedKeyCount {
                expected: jwts.len(),
                found: payload.jwks.as_ref().len(),
            });
        }
        if payload.payload.nonce.as_deref() != Some(expected_nonce) {
            return Err(PoaVerificationError::IncorrectNonce);
        }

        // Validate all the JWTs, against the keys in the payload of the JWTs.
        let mut validations = validations();
        validations.set_audience(&[expected_aud]);
        validations.set_issuer(&[expected_iss]);
        for (jwt, jwk) in jwts.into_iter().zip(payload.jwks.as_ref()) {
            let pubkey = jwk_to_p256(jwk)?;
            let (header, _) = jwt.parse_and_verify_with_header(&(&pubkey).into(), &validations)?;
            if header.typ.as_deref() != Some(POA_JWT_TYP) {
                return Err(PoaVerificationError::IncorrectTyp(header.typ));
            }
        }

        // Check that all keys that must be associated are in the PoA. We use the JWK format for this
        // since it implements Hash, unlike `VerifyingKey`.
        let associated_keys: HashSet<Jwk> = payload.jwks.into_inner().into_iter().collect();
        for key in expected_keys {
            let expected_key = jwk_from_p256(key)?;
            if !associated_keys.contains(&expected_key) {
                return Err(PoaVerificationError::MissingKey(expected_key.into()));
            }
        }

        Ok(())
    }
}

#[cfg(all(test, feature = "software_keys"))]
mod tests {
    use assert_matches::assert_matches;
    use p256::ecdsa::VerifyingKey;
    use rstest::rstest;

    use crate::{
        jwt::{validations, JsonJwt, Jwt, JwtPopClaims},
        keys::{
            poa::{Poa, PoaPayload},
            software::SoftwareEcdsaKey,
            EcdsaKey,
        },
        nonempty::NonEmpty,
    };

    use super::PoaVerificationError;

    async fn poa_setup() -> (Poa, VerifyingKey, VerifyingKey, String, String, String) {
        let key1 = SoftwareEcdsaKey::new_random("key1".to_string());
        let key2 = SoftwareEcdsaKey::new_random("key2".to_string());

        let iss = "iss".to_string();
        let aud = "aud".to_string();
        let nonce = "nonce".to_string();

        let poa = Poa::new(
            vec![&key1, &key2].try_into().unwrap(),
            JwtPopClaims::new(Some(nonce.clone()), iss.clone(), aud.clone()),
        )
        .await
        .unwrap();

        (
            poa,
            key1.verifying_key().await.unwrap(),
            key2.verifying_key().await.unwrap(),
            iss,
            aud,
            nonce,
        )
    }

    #[tokio::test]
    async fn it_works() {
        let (poa, key1, key2, iss, aud, nonce) = poa_setup().await;

        let jwts: Vec<Jwt<PoaPayload>> = poa.clone().into();

        let mut validations = validations();
        validations.set_audience(&[&aud]);
        validations.set_issuer(&[&iss]);

        // Manually verify the JWTs
        for (jwt, key) in jwts.into_iter().zip([key1, key2]) {
            jwt.parse_and_verify(&(&key).into(), &validations).unwrap();
        }

        poa.verify(
            &[key2, key1], // verify() is insensitive to the order of the keys
            &aud,
            &iss,
            &nonce,
        )
        .unwrap();
    }

    #[rstest]
    #[case(Some("other_issuer"), None, None)]
    #[case(None, Some("other_aud"), None)]
    #[case(None, None, Some("other_nonce"))]
    #[tokio::test]
    async fn incorrect_values(
        #[case] verification_iss: Option<&str>,
        #[case] verification_aud: Option<&str>,
        #[case] verification_nonce: Option<&str>,
    ) {
        let (poa, key1, key2, iss, aud, nonce) = poa_setup().await;

        poa.verify(
            &[key1, key2],
            verification_aud.unwrap_or(&aud),
            verification_iss.unwrap_or(&iss),
            verification_nonce.unwrap_or(&nonce),
        )
        .unwrap_err();
    }

    #[tokio::test]
    async fn insufficient_keys() {
        let (poa, key1, _, iss, aud, nonce) = poa_setup().await;

        assert_matches!(
            &poa.verify(&[key1], &aud, &iss, &nonce).unwrap_err(),
            PoaVerificationError::UnexpectedSignatureCount { .. }
        );
    }

    #[tokio::test]
    async fn too_many_keys() {
        let (poa, key1, key2, iss, aud, nonce) = poa_setup().await;

        let key3 = SoftwareEcdsaKey::new_random("key3".to_string())
            .verifying_key()
            .await
            .unwrap();

        assert_matches!(
            &poa.verify(&[key1, key2, key3], &aud, &iss, &nonce).unwrap_err(),
            PoaVerificationError::UnexpectedSignatureCount { .. }
        );
    }

    #[tokio::test]
    async fn missing_signature() {
        let (poa, key1, _, iss, aud, nonce) = poa_setup().await;

        let mut jwts: Vec<Jwt<PoaPayload>> = poa.into(); // a poa always involves at least two keys
        jwts.pop();
        let jwts: NonEmpty<_> = jwts.try_into().unwrap(); // jwts always has at least one left after the pop();
        let poa: JsonJwt<PoaPayload> = jwts.try_into().unwrap();

        assert_matches!(
            &poa.verify(&[key1], &aud, &iss, &nonce).unwrap_err(),
            PoaVerificationError::UnexpectedKeyCount { .. }
        );
    }

    #[tokio::test]
    async fn missing_key() {
        let (poa, key1, _, iss, aud, nonce) = poa_setup().await;

        let other_key = SoftwareEcdsaKey::new_random("other_key".to_string())
            .verifying_key()
            .await
            .unwrap();

        assert_matches!(
            &poa.verify(&[key1, other_key], &aud, &iss, &nonce).unwrap_err(),
            PoaVerificationError::MissingKey { .. }
        );
    }
}
