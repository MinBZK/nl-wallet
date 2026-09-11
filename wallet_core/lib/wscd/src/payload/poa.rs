use std::collections::HashSet;

use crypto::PublicKey;
use crypto::keys::EcdsaKey;
use crypto::wscd::WscdPoa;
use derive_more::Constructor;
use futures::future::try_join_all;
use jwt::DEFAULT_VALIDATION;
use jwt::JsonJwt;
use jwt::JwtDecodingKey;
use jwt::JwtTyp;
use jwt::SignedJwt;
use jwt::UnverifiedJwt;
use jwt::error::JwkConversionError;
use jwt::error::JwtParseError;
use jwt::error::JwtSignError;
use jwt::error::JwtVerifyError;
use jwt::jwk::AlgorithmParameters;
use jwt::jwk::Jwk;
use jwt::jwk::jwk_alg_from_public_key;
use jwt::jwk::jwk_from_public_key;
use jwt::jwk::jwk_to_public_key;
use jwt::nonce::Nonce;
use serde::Deserialize;
use serde::Serialize;
use utils::vec_at_least::VecAtLeastTwoUnique;
use utils::vec_at_least::VecNonEmpty;

use crate::payload::pop::JwtPopClaims;

pub const POA_JWT_TYP: &str = "poa+jwt";

#[derive(Debug, thiserror::Error)]
pub enum PoaSigning {
    #[error("error converting key from/to JWK: {0}")]
    Jwk(#[source] JwkConversionError),

    #[error("JWT bulk signing error: {0}")]
    Sign(#[source] JwtSignError),

    #[error("error obtaining verifying key from signing key: {0}")]
    VerifyingKey(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
}

#[derive(Debug, thiserror::Error)]
pub enum PoaVerificationError {
    #[error("JWT verification error: {0}")]
    JwtParse(#[source] JwtParseError),

    #[error("unexpected amount of signatures in PoA: expected {expected}, found {found}")]
    UnexpectedSignatureCount { expected: usize, found: usize },

    #[error("unexpected amount of keys in PoA: expected {expected}, found {found}")]
    UnexpectedKeyCount { expected: usize, found: usize },

    #[error("nonce is missing from PoA payload")]
    MissingNonce,

    #[error("incorrect nonce")]
    IncorrectNonce,

    #[error("error converting key from/to JWK: {0}")]
    Jwk(#[source] JwkConversionError),

    #[error("any of the JWTs in the PoA are invalid: {0}")]
    InvalidJwt(#[source] JwtVerifyError),

    #[error("key missing in PoA: {0:?}")]
    MissingKey(AlgorithmParameters),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoaPayload {
    #[serde(flatten)]
    pub payload: JwtPopClaims,
    pub jwks: VecAtLeastTwoUnique<Jwk>,
}

impl JwtTyp for PoaPayload {
    const TYP: &'static str = POA_JWT_TYP;
}

/// A Proof of Association, asserting that a set of credential public keys are managed by a single WSCD.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Poa(JsonJwt<PoaPayload>);

#[derive(Debug, Constructor)]
pub struct JwtPoaInput {
    pub nonce: Option<Nonce>,
    pub aud: String,
}

impl TryFrom<VecNonEmpty<UnverifiedJwt<PoaPayload>>> for Poa {
    type Error = JwtParseError;

    fn try_from(source: VecNonEmpty<UnverifiedJwt<PoaPayload>>) -> Result<Self, Self::Error> {
        let json_jwt: JsonJwt<_, _> = source.try_into()?;

        Ok(Poa(json_jwt))
    }
}

impl From<Poa> for Vec<UnverifiedJwt<PoaPayload>> {
    fn from(source: Poa) -> Self {
        source.0.into()
    }
}

impl WscdPoa for Poa {
    type Input = JwtPoaInput;
}

impl Poa {
    pub async fn new<K: EcdsaKey>(keys: VecAtLeastTwoUnique<&K>, payload: JwtPopClaims) -> Result<Poa, PoaSigning> {
        let payload = PoaPayload {
            payload,
            jwks: try_join_all(keys.as_slice().iter().map(|privkey| async {
                jwk_from_public_key(&PublicKey::from(
                    privkey
                        .verifying_key()
                        .await
                        .map_err(|error| PoaSigning::VerifyingKey(Box::new(error)))?,
                ))
                .map_err(PoaSigning::Jwk)
            }))
            .await?
            .try_into()
            .unwrap(), // our iterable is a VecAtLeastTwo
        };

        let jwts: VecNonEmpty<_> =
            try_join_all(keys.as_slice().iter().map(async |key| {
                Result::<_, JwtSignError>::Ok(SignedJwt::sign(&payload, *key).await?.into_unverified())
            }))
            .await
            .map_err(PoaSigning::Sign)?
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
        expected_keys: &[PublicKey],
        expected_aud: &str,
        accepted_issuers: &[String],
        expected_nonce: &Nonce,
    ) -> Result<(), PoaVerificationError> {
        let nonce = self.verify_returning_nonce(expected_keys, expected_aud, accepted_issuers)?;

        if nonce != *expected_nonce {
            return Err(PoaVerificationError::IncorrectNonce);
        }

        Ok(())
    }

    /// Verify the PoA and return the nonce used, checking that:
    ///
    /// - all `expected_keys` are in the PoA (and no other keys). The keys may be passed in any order.
    /// - all signatures are valid against all keys in the PoA, and the order of the JWKs in the payload corresponds to
    ///   the order of the signatures.
    /// - the `aud` and `iss` fields in the payload have the expected values.
    /// - a nonce is present in the payload
    fn verify_returning_nonce(
        self,
        expected_keys: &[PublicKey],
        expected_aud: &str,
        accepted_issuers: &[String],
    ) -> Result<Nonce, PoaVerificationError> {
        let jwts: Vec<UnverifiedJwt<_, _>> = self.into();

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
        let (_, payload) = jwts
            .first()
            .unwrap()
            .dangerous_parse_unverified()
            .map_err(PoaVerificationError::JwtParse)?;
        if jwts.len() != payload.jwks.as_slice().len() {
            return Err(PoaVerificationError::UnexpectedKeyCount {
                expected: jwts.len(),
                found: payload.jwks.as_slice().len(),
            });
        }

        let nonce = payload.payload.nonce.ok_or(PoaVerificationError::MissingNonce)?;

        // Validate all the JWTs, against the keys in the payload of the JWTs.
        let mut base_validation = DEFAULT_VALIDATION.to_owned();
        base_validation.require_aud(expected_aud);
        base_validation.require_iss(accepted_issuers);
        for (jwt, jwk) in jwts.into_iter().zip(payload.jwks.as_slice()) {
            let pubkey = jwk_to_public_key(jwk).map_err(PoaVerificationError::Jwk)?;
            let validation = base_validation.clone().into_validation(&pubkey);
            jwt.parse_and_verify(JwtDecodingKey::from(&pubkey), validation)
                .map_err(PoaVerificationError::InvalidJwt)?;
        }

        // Check that all keys that must be associated are in the PoA. We use `jwk::AlgorithmParameters` for this
        // since it implements Hash, unlike `VerifyingKey`. When comparing if two keys are equal, this type takes
        // exactly the right information into account (the EC curve identifier as well as the x and y coordinates),
        // while discarding irrelevant other keys from the JWK (e.g. `kid`, `x5c` and friends, `use`, `alg`).
        let associated_keys: HashSet<AlgorithmParameters> =
            payload.jwks.into_inner().into_iter().map(|key| key.algorithm).collect();
        for key in expected_keys {
            let expected_key = jwk_alg_from_public_key(key).map_err(PoaVerificationError::Jwk)?;
            if !associated_keys.contains(&expected_key) {
                return Err(PoaVerificationError::MissingKey(expected_key));
            }
        }

        Ok(nonce)
    }
}

#[cfg(feature = "mock")]
mod mock {
    use super::Poa;

    impl Poa {
        pub fn set_payload(&mut self, payload: String) {
            self.0.payload = payload;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use crypto::PublicKey;
    use crypto::mock_remote::MockRemoteEcdsaKey;
    use futures::FutureExt;
    use jwt::DEFAULT_VALIDATION;
    use jwt::JwtDecodingKey;
    use jwt::UnverifiedJwt;
    use jwt::nonce::Nonce;
    use p256::ecdsa::SigningKey;
    use p256::elliptic_curve::Generate;
    use rstest::rstest;
    use utils::generator::mock::MockTimeGenerator;
    use utils::vec_at_least::VecNonEmpty;

    use super::Poa;
    use super::PoaPayload;
    use super::PoaVerificationError;
    use crate::payload::pop::JwtPopClaims;

    fn poa_setup() -> (Poa, PublicKey, PublicKey, String, String, Nonce) {
        let key1 = MockRemoteEcdsaKey::new_random("key1".into());
        let key2 = MockRemoteEcdsaKey::new_random("key2".into());

        let iss = "iss".to_string();
        let aud = "aud".to_string();
        let nonce = Nonce::from("nonce".to_string());

        let poa = Poa::new(
            vec![&key1, &key2].try_into().unwrap(),
            JwtPopClaims::new(
                Some(nonce.clone()),
                iss.clone(),
                aud.clone(),
                &MockTimeGenerator::default(),
            ),
        )
        .now_or_never()
        .unwrap()
        .unwrap();

        (
            poa,
            PublicKey::from(*key1.verifying_key()),
            PublicKey::from(*key2.verifying_key()),
            iss,
            aud,
            nonce,
        )
    }

    #[test]
    fn it_works() {
        let (poa, key1, key2, iss, aud, nonce) = poa_setup();

        let jwts: Vec<UnverifiedJwt<PoaPayload>> = poa.clone().into();

        let mut base_validation = DEFAULT_VALIDATION.to_owned();
        base_validation.require_aud(&aud);
        base_validation.require_iss([&iss]);

        // Manually verify the JWTs
        for (jwt, key) in jwts.into_iter().zip([&key1, &key2]) {
            let validation = base_validation.clone().into_validation(key);
            jwt.parse_and_verify(JwtDecodingKey::from(key), &validation).unwrap();
        }

        poa.verify(
            &[key2, key1], // verify() is insensitive to the order of the keys
            &aud,
            &[iss],
            &nonce,
        )
        .unwrap();
    }

    #[rstest]
    #[case(Some("other_issuer"), None, None)]
    #[case(None, Some("other_aud"), None)]
    #[case(None, None, Some("other_nonce"))]
    #[test]
    fn incorrect_values(
        #[case] verification_iss: Option<&str>,
        #[case] verification_aud: Option<&str>,
        #[case] verification_nonce: Option<&str>,
    ) {
        let (poa, key1, key2, iss, aud, nonce) = poa_setup();

        poa.verify(
            &[key1, key2],
            verification_aud.unwrap_or(&aud),
            &[verification_iss.unwrap_or(&iss).to_string()],
            &verification_nonce
                .map(|nonce| Nonce::from(nonce.to_string()))
                .unwrap_or(nonce),
        )
        .unwrap_err();
    }

    #[test]
    fn insufficient_keys() {
        let (poa, key1, _, iss, aud, nonce) = poa_setup();

        assert_matches!(
            &poa.verify(&[key1], &aud, &[iss], &nonce).unwrap_err(),
            PoaVerificationError::UnexpectedSignatureCount { .. }
        );
    }

    #[test]
    fn too_many_keys() {
        let (poa, key1, key2, iss, aud, nonce) = poa_setup();

        let key3 = PublicKey::from(*SigningKey::generate().verifying_key());

        assert_matches!(
            &poa.verify(&[key1, key2, key3], &aud, &[iss], &nonce).unwrap_err(),
            PoaVerificationError::UnexpectedSignatureCount { .. }
        );
    }

    #[test]
    fn missing_signature() {
        let (poa, key1, _, iss, aud, nonce) = poa_setup();

        let mut jwts: Vec<UnverifiedJwt<PoaPayload>> = poa.into(); // a poa always involves at least two keys
        jwts.pop();
        let jwts: VecNonEmpty<_> = jwts.try_into().unwrap(); // jwts always has at least one left after the pop();
        let poa: Poa = jwts.try_into().unwrap();

        assert_matches!(
            &poa.verify(&[key1], &aud, &[iss], &nonce).unwrap_err(),
            PoaVerificationError::UnexpectedKeyCount { .. }
        );
    }

    #[test]
    fn missing_key() {
        let (poa, key1, _, iss, aud, nonce) = poa_setup();

        let other_key = PublicKey::from(*SigningKey::generate().verifying_key());

        assert_matches!(
            &poa.verify(&[key1, other_key], &aud, &[iss], &nonce).unwrap_err(),
            PoaVerificationError::MissingKey { .. }
        );
    }
}
