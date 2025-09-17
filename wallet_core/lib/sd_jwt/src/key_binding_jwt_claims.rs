// Copyright 2020-2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0
use chrono::Duration;
use chrono::serde::ts_seconds;
use derive_more::Display;
use jsonwebtoken::Algorithm;
use jsonwebtoken::Header;
use jsonwebtoken::Validation;
use jsonwebtoken::jwk::Jwk;
use p256::ecdsa::VerifyingKey;
use serde::Deserialize;
use serde::Serialize;
use serde_with::chrono::DateTime;
use serde_with::chrono::Utc;

use crypto::CredentialEcdsaKey;
use crypto::EcdsaKey;
use crypto::wscd::DisclosureWscd;
use crypto::wscd::WscdPoa;
use jwt::EcdsaDecodingKey;
use jwt::UnverifiedJwt;
use jwt::VerifiedJwt;
use jwt::jwk::jwk_to_p256;
use utils::generator::Generator;
use utils::vec_at_least::IntoNonEmptyIterator;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;

use crate::error::Error;
use crate::error::Result;
use crate::hasher::Hasher;
use crate::sd_jwt::SdJwt;

pub const KB_JWT_HEADER_TYP: &str = "kb+jwt";

/// Representation of a [KB-JWT](https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-12.html#name-key-binding-jwt).
#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub struct KeyBindingJwt(VerifiedJwt<KeyBindingJwtClaims>);

impl KeyBindingJwt {
    pub fn parse_and_verify(
        s: &str,
        pubkey: &EcdsaDecodingKey,
        expected_aud: &str,
        expected_nonce: &str,
        iat_acceptance_window: Duration,
        time: &impl Generator<DateTime<Utc>>,
    ) -> Result<Self> {
        let jwt: UnverifiedJwt<KeyBindingJwtClaims> = s.parse()?;

        let verified_jwt = jwt.into_verified(pubkey, &kb_jwt_validation(expected_aud))?;
        if verified_jwt
            .header()
            .typ
            .as_ref()
            .is_none_or(|typ| typ != KB_JWT_HEADER_TYP)
        {
            return Err(Error::Deserialization(format!(
                "invalid KB-JWT: typ must be \"{KB_JWT_HEADER_TYP}\""
            )));
        }

        if verified_jwt.payload().nonce != expected_nonce {
            return Err(Error::Deserialization(String::from("invalid KB-JWT: unexpected nonce")));
        }

        if (verified_jwt.payload().iat + iat_acceptance_window) < time.generate() {
            return Err(Error::Deserialization(String::from(
                "invalid KB-JWT: iat not in acceptable window",
            )));
        }

        Ok(Self(verified_jwt))
    }

    /// Returns a reference to this [`KeyBindingJwt`] claim set.
    pub fn claims(&self) -> &KeyBindingJwtClaims {
        self.0.payload()
    }
}

fn kb_jwt_validation(expected_aud: &str) -> Validation {
    let mut validation = Validation::new(Algorithm::ES256);
    validation.validate_nbf = true;
    validation.leeway = 0;
    validation.set_audience(&[expected_aud]);
    validation.set_required_spec_claims(&["aud"]);
    validation
}

/// Builder-style struct to ease the creation of an [`KeyBindingJwt`].
#[derive(Debug, Clone)]
pub struct KeyBindingJwtBuilder {
    header: Header,
    iat: DateTime<Utc>,
    aud: String,
    nonce: String,
}

impl KeyBindingJwtBuilder {
    fn sd_hash_for_sd_jwt(sd_jwt: &SdJwt) -> Result<String> {
        let hasher = sd_jwt.claims()._sd_alg.unwrap_or_default().hasher()?;

        let sd_hash = hasher.encoded_digest(&sd_jwt.to_string());

        Ok(sd_hash)
    }

    pub fn new(iat: DateTime<Utc>, aud: String, nonce: String, alg: Algorithm) -> Self {
        let header = Header {
            typ: Some(String::from(KB_JWT_HEADER_TYP)),
            alg,
            ..Default::default()
        };

        Self {
            header,
            iat,
            aud,
            nonce,
        }
    }

    /// Builds an [`KeyBindingJwt`] from the data provided to builder.
    pub(crate) async fn finish(self, sd_jwt: &SdJwt, signing_key: &impl EcdsaKey) -> Result<KeyBindingJwt> {
        let sd_hash = Self::sd_hash_for_sd_jwt(sd_jwt)?;

        let claims = KeyBindingJwtClaims {
            iat: self.iat,
            aud: self.aud,
            nonce: self.nonce,
            sd_hash,
        };

        let verified_jwt = VerifiedJwt::sign(claims, self.header, signing_key).await?;

        Ok(KeyBindingJwt(verified_jwt))
    }

    /// Builds several [`KeyBindingJwt`]s from the data provided by one builder, using the WSCD.
    pub(crate) async fn multi_finish<W, K, P>(
        self,
        sd_jwts_and_keys: &VecNonEmpty<(SdJwt, K)>,
        wscd: &W,
        poa_input: P::Input,
    ) -> Result<(VecNonEmpty<KeyBindingJwt>, Option<P>)>
    where
        W: DisclosureWscd<Key = K, Poa = P>,
        K: CredentialEcdsaKey,
        P: WscdPoa,
    {
        // Create a `KeyBindingJwtClaims` for each `SdJwt`, based on the contents of the builder,
        // a header copied from the builder, and combine it with the provided key.
        let sd_jwt_count = sd_jwts_and_keys.len().get();
        let contents_and_keys = sd_jwts_and_keys
            .into_iter()
            .zip(itertools::repeat_n(self, sd_jwt_count))
            .map(
                |(
                    (sd_jwt, key),
                    KeyBindingJwtBuilder {
                        header,
                        iat,
                        aud,
                        nonce,
                    },
                )| {
                    let sd_hash = Self::sd_hash_for_sd_jwt(sd_jwt)?;

                    let claims = KeyBindingJwtClaims {
                        iat,
                        aud,
                        nonce,
                        sd_hash,
                    };

                    Ok((claims, header, key))
                },
            )
            .collect::<Result<Vec<_>>>()?
            .try_into()
            // This unwrap is safe, as the source iterator is non-empty.
            .unwrap();

        // Create JWTs from all of these by having the WSCD sign the
        // `KeyBindingJwtClaims` values and their accompanying headers.
        let (verified_jwts, poa) = VerifiedJwt::multi_sign(contents_and_keys, wscd, poa_input).await?;

        let key_binding_jwts = verified_jwts.into_nonempty_iter().map(KeyBindingJwt).collect();

        Ok((key_binding_jwts, poa))
    }
}

/// Claims set for key binding JWT.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct KeyBindingJwtClaims {
    #[serde(with = "ts_seconds")]
    pub iat: DateTime<Utc>,
    pub aud: String,
    pub nonce: String,
    pub sd_hash: String,
}

/// Proof of possession of a given key. See [RFC7800](https://www.rfc-editor.org/rfc/rfc7800.html#section-3) for more details.
/// Currently, only Jwk is supported.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RequiredKeyBinding {
    /// Json Web Key (JWK).
    Jwk(Jwk),
}

impl RequiredKeyBinding {
    pub fn verifying_key(&self) -> Result<VerifyingKey> {
        let verifying_key = match self {
            Self::Jwk(jwk) => jwk_to_p256(jwk)?,
        };

        Ok(verifying_key)
    }
}

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;
    use chrono::Duration;
    use chrono::Utc;
    use futures::FutureExt;
    use itertools::Itertools;
    use jsonwebtoken::Algorithm;
    use jsonwebtoken::Header;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;
    use serde_json::json;
    use ssri::Integrity;

    use crypto::mock_remote::MockRemoteEcdsaKey;
    use crypto::mock_remote::MockRemoteWscd;
    use jwt::EcdsaDecodingKey;
    use jwt::UnverifiedJwt;
    use utils::generator::mock::MockTimeGenerator;
    use utils::vec_at_least::IntoNonEmptyIterator;
    use utils::vec_at_least::NonEmptyIterator;
    use utils::vec_nonempty;

    use crate::builder::SdJwtBuilder;
    use crate::error::Error;
    use crate::examples::SIMPLE_STRUCTURED_SD_JWT;
    use crate::examples::examples_sd_jwt_decoding_key;
    use crate::hasher::Hasher;
    use crate::hasher::Sha256Hasher;
    use crate::key_binding_jwt_claims::KeyBindingJwt;
    use crate::key_binding_jwt_claims::KeyBindingJwtBuilder;
    use crate::key_binding_jwt_claims::KeyBindingJwtClaims;
    use crate::sd_jwt::SdJwt;

    async fn example_kb_jwt(signing_key: &SigningKey, header: Header) -> UnverifiedJwt<KeyBindingJwtClaims> {
        UnverifiedJwt::sign(
            &KeyBindingJwtClaims {
                iat: Utc::now() - Duration::days(2),
                aud: String::from("aud"),
                nonce: String::from("abc123"),
                sd_hash: String::from("sd_hash"),
            },
            &header,
            signing_key,
        )
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_key_binding_jwt_builder() {
        let sd_jwt = SdJwt::parse_and_verify(SIMPLE_STRUCTURED_SD_JWT, &examples_sd_jwt_decoding_key()).unwrap();

        let signing_key = SigningKey::random(&mut OsRng);
        let hasher = Sha256Hasher;

        let iat = Utc::now();

        let kb_jwt = KeyBindingJwtBuilder::new(iat, String::from("receiver"), String::from("abc123"), Algorithm::ES256)
            .finish(&sd_jwt, &signing_key)
            .await
            .unwrap();

        let sd_hash = hasher.encoded_digest(sd_jwt.presentation().as_str());

        assert_eq!(iat.timestamp(), kb_jwt.claims().iat.timestamp());
        assert_eq!(String::from("receiver"), kb_jwt.claims().aud);
        assert_eq!(String::from("abc123"), kb_jwt.claims().nonce);
        assert_eq!(sd_hash, kb_jwt.claims().sd_hash);
        assert_eq!(Some(String::from("kb+jwt")), kb_jwt.0.header().typ);
        assert_eq!(Algorithm::ES256, kb_jwt.0.header().alg);
    }

    #[test]
    fn test_key_binding_jwt_builder_multi() {
        let issuer_signing_key = SigningKey::random(&mut OsRng);
        let key1 = MockRemoteEcdsaKey::new_random("key1".to_string());
        let key2 = MockRemoteEcdsaKey::new_random("key2".to_string());
        let wscd = MockRemoteWscd::new(vec![key1.clone(), key2.clone()]);

        let iat = Utc::now();

        let sd_jwts_and_keys = vec_nonempty![("Doe", key1), ("Deer", key2)]
            .into_nonempty_iter()
            .map(|(family_name, key)| {
                // Create a minimal SD-JWT that contains the holder verifying key as JWK.
                let sd_jwt = SdJwtBuilder::new(json!({
                    "iss": "https://iss.example.com",
                    "iat": iat.timestamp(),
                    "family_name": family_name
                }))
                .unwrap()
                .finish(
                    Algorithm::ES256,
                    Integrity::from(""),
                    &issuer_signing_key,
                    vec![],
                    key.verifying_key(),
                )
                .now_or_never()
                .unwrap()
                .unwrap();

                (sd_jwt, key)
            })
            .collect();

        let (kb_jwts, poa) =
            KeyBindingJwtBuilder::new(iat, String::from("receiver"), String::from("abc123"), Algorithm::ES256)
                .multi_finish(&sd_jwts_and_keys, &wscd, ())
                .now_or_never()
                .unwrap()
                .expect("signing multiple KeyBindingJwt values using WSCD should succeed");

        assert!(poa.is_some());

        let hasher = Sha256Hasher;

        for (sd_jwt, key_binding_jwt) in sd_jwts_and_keys
            .iter()
            .zip_eq(kb_jwts.iter())
            .map(|((sd_jwt, _), kb_jwt)| (sd_jwt, kb_jwt))
        {
            let sd_hash = hasher.encoded_digest(sd_jwt.presentation().as_str());

            assert_eq!(iat.timestamp(), key_binding_jwt.claims().iat.timestamp());
            assert_eq!(String::from("receiver"), key_binding_jwt.claims().aud);
            assert_eq!(String::from("abc123"), key_binding_jwt.claims().nonce);
            assert_eq!(sd_hash, key_binding_jwt.claims().sd_hash);
            assert_eq!(Some(String::from("kb+jwt")), key_binding_jwt.0.header().typ);
            assert_eq!(Algorithm::ES256, key_binding_jwt.0.header().alg);
        }
    }

    #[tokio::test]
    async fn test_parse_should_validate() {
        let signing_key = SigningKey::random(&mut OsRng);

        let mut header = Header::new(Algorithm::ES256);
        header.typ = Some(String::from("kb+jwt"));

        KeyBindingJwt::parse_and_verify(
            example_kb_jwt(&signing_key, header).await.serialization(),
            &EcdsaDecodingKey::from(signing_key.verifying_key()),
            "aud",
            "abc123",
            Duration::days(3),
            &MockTimeGenerator::default(),
        )
        .unwrap();
    }

    #[tokio::test]
    async fn test_parse_should_error_for_wrong_typ() {
        let signing_key = SigningKey::random(&mut OsRng);

        let err = KeyBindingJwt::parse_and_verify(
            example_kb_jwt(&signing_key, Header::new(Algorithm::ES256))
                .await
                .serialization(),
            &EcdsaDecodingKey::from(signing_key.verifying_key()),
            "aud",
            "abc123",
            Duration::days(3),
            &MockTimeGenerator::default(),
        )
        .expect_err("should fail validation");
        assert_matches!(err, Error::Deserialization(msg) if msg == "invalid KB-JWT: typ must be \"kb+jwt\"");
    }

    #[tokio::test]
    async fn test_parse_should_error_for_wrong_iat() {
        let signing_key = SigningKey::random(&mut OsRng);

        let mut header = Header::new(Algorithm::ES256);
        header.typ = Some(String::from("kb+jwt"));

        let err = KeyBindingJwt::parse_and_verify(
            example_kb_jwt(&signing_key, header).await.serialization(),
            &EcdsaDecodingKey::from(signing_key.verifying_key()),
            "aud",
            "abc123",
            Duration::days(1),
            &MockTimeGenerator::default(),
        )
        .expect_err("should fail validation");
        assert_matches!(err, Error::Deserialization(msg) if msg == "invalid KB-JWT: iat not in acceptable window");
    }

    #[tokio::test]
    async fn test_parse_should_error_for_wrong_nonce() {
        let signing_key = SigningKey::random(&mut OsRng);

        let mut header = Header::new(Algorithm::ES256);
        header.typ = Some(String::from("kb+jwt"));

        let err = KeyBindingJwt::parse_and_verify(
            example_kb_jwt(&signing_key, header).await.serialization(),
            &EcdsaDecodingKey::from(signing_key.verifying_key()),
            "aud",
            "def456",
            Duration::days(3),
            &MockTimeGenerator::default(),
        )
        .expect_err("should fail validation");
        assert_matches!(err, Error::Deserialization(msg) if msg == "invalid KB-JWT: unexpected nonce");
    }
}
