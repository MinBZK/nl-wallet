use std::collections::HashSet;

use futures::future::try_join_all;
use jsonwebtoken::jwk;
use jsonwebtoken::jwk::Jwk;
use jsonwebtoken::Algorithm;
use jsonwebtoken::Header;
use p256::ecdsa::VerifyingKey;
use serde::Deserialize;
use serde::Serialize;

use crate::jwt::jwk_alg_from_p256;
use crate::jwt::jwk_from_p256;
use crate::jwt::jwk_to_p256;
use crate::jwt::validations;
use crate::jwt::JsonJwt;
use crate::jwt::JwkConversionError;
use crate::jwt::Jwt;
use crate::jwt::JwtError;
use crate::jwt::JwtPopClaims;
use crate::vec_at_least::VecAtLeastTwoUnique;
use crate::vec_at_least::VecNonEmpty;

use super::EcdsaKey;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoaPayload {
    #[serde(flatten)]
    pub payload: JwtPopClaims,
    pub jwks: VecAtLeastTwoUnique<Jwk>,
}

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
    MissingKey(jwk::AlgorithmParameters),
}

impl Poa {
    pub async fn new<K: EcdsaKey>(keys: VecAtLeastTwoUnique<&K>, payload: JwtPopClaims) -> Result<Poa, PoaError> {
        let payload = PoaPayload {
            payload,
            jwks: try_join_all(keys.as_slice().iter().map(|privkey| async {
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

        let jwts: VecNonEmpty<_> = try_join_all(keys.as_slice().iter().map(|key| Jwt::sign(&payload, &header, *key)))
            .await?
            .try_into()
            .unwrap(); // our iterable is a `VecAtLeastTwo`

        // This unwrap() is safe because we correctly constructed the `jwts` above.
        Ok(jwts.try_into().unwrap())
    }

    /// Verify the PoA, checking that:
    ///
    /// - all `expected_keys` are in the PoA (and no other keys). The keys may be passed in any order.
    /// - all signatures are valid against all keys in the PoA, and the order of the JWKs in the payload corresponds to
    ///   the order of the signatures.
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
        if jwts.len() != payload.jwks.as_slice().len() {
            return Err(PoaVerificationError::UnexpectedKeyCount {
                expected: jwts.len(),
                found: payload.jwks.as_slice().len(),
            });
        }
        if payload.payload.nonce.as_deref() != Some(expected_nonce) {
            return Err(PoaVerificationError::IncorrectNonce);
        }

        // Validate all the JWTs, against the keys in the payload of the JWTs.
        let mut validations = validations();
        validations.set_audience(&[expected_aud]);
        validations.set_issuer(&[expected_iss]);
        for (jwt, jwk) in jwts.into_iter().zip(payload.jwks.as_slice()) {
            let pubkey = jwk_to_p256(jwk)?;
            let (header, _) = jwt.parse_and_verify_with_header(&(&pubkey).into(), &validations)?;
            if header.typ.as_deref() != Some(POA_JWT_TYP) {
                return Err(PoaVerificationError::IncorrectTyp(header.typ));
            }
        }

        // Check that all keys that must be associated are in the PoA. We use `jwk::AlgorithmParameters` for this
        // since it implements Hash, unlike `VerifyingKey`. When comparing if two keys are equal, this type takes
        // exactly the right information into account (the EC curve identifier as well as the x and y coordinates),
        // while discarding irrelevant other keys from the JWK (e.g. `kid`, `x5c` and friends, `use`, `alg`).
        let associated_keys: HashSet<jwk::AlgorithmParameters> =
            payload.jwks.into_inner().into_iter().map(|key| key.algorithm).collect();
        for key in expected_keys {
            let expected_key = jwk_alg_from_p256(key)?;
            if !associated_keys.contains(&expected_key) {
                return Err(PoaVerificationError::MissingKey(expected_key));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use p256::ecdsa::SigningKey;
    use p256::ecdsa::VerifyingKey;
    use rand_core::OsRng;
    use rstest::rstest;

    use crate::jwt::validations;
    use crate::jwt::JsonJwt;
    use crate::jwt::Jwt;
    use crate::jwt::JwtPopClaims;
    use crate::keys::mock_remote::MockRemoteEcdsaKey;
    use crate::vec_at_least::VecNonEmpty;

    use super::Poa;
    use super::PoaPayload;
    use super::PoaVerificationError;

    async fn poa_setup() -> (Poa, VerifyingKey, VerifyingKey, String, String, String) {
        let key1 = MockRemoteEcdsaKey::new_random("key1".into());
        let key2 = MockRemoteEcdsaKey::new_random("key2".into());

        let iss = "iss".to_string();
        let aud = "aud".to_string();
        let nonce = "nonce".to_string();

        let poa = Poa::new(
            vec![&key1, &key2].try_into().unwrap(),
            JwtPopClaims::new(Some(nonce.clone()), iss.clone(), aud.clone()),
        )
        .await
        .unwrap();

        (poa, *key1.verifying_key(), *key2.verifying_key(), iss, aud, nonce)
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

        let key3 = *SigningKey::random(&mut OsRng).verifying_key();

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
        let jwts: VecNonEmpty<_> = jwts.try_into().unwrap(); // jwts always has at least one left after the pop();
        let poa: JsonJwt<PoaPayload> = jwts.try_into().unwrap();

        assert_matches!(
            &poa.verify(&[key1], &aud, &iss, &nonce).unwrap_err(),
            PoaVerificationError::UnexpectedKeyCount { .. }
        );
    }

    #[tokio::test]
    async fn missing_key() {
        let (poa, key1, _, iss, aud, nonce) = poa_setup().await;

        let other_key = *SigningKey::random(&mut OsRng).verifying_key();

        assert_matches!(
            &poa.verify(&[key1, other_key], &aud, &iss, &nonce).unwrap_err(),
            PoaVerificationError::MissingKey { .. }
        );
    }
}
