// Copyright 2020-2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0
use chrono::Duration;
use chrono::serde::ts_seconds;
use derive_more::Display;
use jsonwebtoken::Algorithm;
use jsonwebtoken::Header;
use jsonwebtoken::Validation;
use jsonwebtoken::jwk::Jwk;
use serde::Deserialize;
use serde::Serialize;
use serde_with::chrono::DateTime;
use serde_with::chrono::Utc;

use crypto::EcdsaKeySend;
use jwt::EcdsaDecodingKey;
use jwt::UnverifiedJwt;
use jwt::VerifiedJwt;

use crate::error;
use crate::error::Error;
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
    ) -> error::Result<Self> {
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

        if (verified_jwt.payload().iat + iat_acceptance_window) < Utc::now() {
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
    alg: Algorithm,
    iat: DateTime<Utc>,
    aud: String,
    nonce: String,
}

impl KeyBindingJwtBuilder {
    pub fn new(iat: DateTime<Utc>, aud: String, nonce: String, alg: Algorithm) -> Self {
        Self { alg, iat, aud, nonce }
    }

    /// Builds an [`KeyBindingJwt`] from the data provided to builder.
    pub(crate) async fn finish(self, sd_jwt: &SdJwt, signing_key: &impl EcdsaKeySend) -> Result<KeyBindingJwt, Error> {
        let hasher = sd_jwt.claims()._sd_alg.unwrap_or_default().hasher()?;
        let sd_hash = hasher.encoded_digest(&sd_jwt.to_string());

        // TODO use HeaderWithTyp (PVW-4868)
        let header = Header {
            typ: Some(String::from(KB_JWT_HEADER_TYP)),
            alg: self.alg,
            ..Default::default()
        };

        let claims = KeyBindingJwtClaims {
            iat: self.iat,
            aud: self.aud,
            nonce: self.nonce,
            sd_hash,
        };

        let verified_jwt = VerifiedJwt::sign(header, claims, signing_key).await?;

        Ok(KeyBindingJwt(verified_jwt))
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

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;
    use chrono::Duration;
    use chrono::Utc;
    use jsonwebtoken::Algorithm;
    use jsonwebtoken::Header;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;

    use jwt::EcdsaDecodingKey;
    use jwt::VerifiedJwt;

    use crate::error::Error;
    use crate::examples::SIMPLE_STRUCTURED_SD_JWT;
    use crate::examples::examples_sd_jwt_decoding_key;
    use crate::hasher::Hasher;
    use crate::hasher::Sha256Hasher;
    use crate::key_binding_jwt_claims::KeyBindingJwt;
    use crate::key_binding_jwt_claims::KeyBindingJwtBuilder;
    use crate::key_binding_jwt_claims::KeyBindingJwtClaims;
    use crate::sd_jwt::SdJwt;

    async fn example_kb_jwt(signing_key: &SigningKey, header: Header) -> VerifiedJwt<KeyBindingJwtClaims> {
        VerifiedJwt::sign(
            header,
            KeyBindingJwtClaims {
                iat: Utc::now() - Duration::days(2),
                aud: String::from("aud"),
                nonce: String::from("abc123"),
                sd_hash: String::from("sd_hash"),
            },
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

    #[tokio::test]
    async fn test_parse_should_validate() {
        let signing_key = SigningKey::random(&mut OsRng);

        let mut header = Header::new(Algorithm::ES256);
        header.typ = Some(String::from("kb+jwt"));

        KeyBindingJwt::parse_and_verify(
            example_kb_jwt(&signing_key, header).await.jwt().serialization(),
            &EcdsaDecodingKey::from(signing_key.verifying_key()),
            "aud",
            "abc123",
            Duration::days(3),
        )
        .unwrap();
    }

    #[tokio::test]
    async fn test_parse_should_error_for_wrong_typ() {
        let signing_key = SigningKey::random(&mut OsRng);

        let err = KeyBindingJwt::parse_and_verify(
            example_kb_jwt(&signing_key, Header::new(Algorithm::ES256))
                .await
                .jwt()
                .serialization(),
            &EcdsaDecodingKey::from(signing_key.verifying_key()),
            "aud",
            "abc123",
            Duration::days(3),
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
            example_kb_jwt(&signing_key, header).await.jwt().serialization(),
            &EcdsaDecodingKey::from(signing_key.verifying_key()),
            "aud",
            "abc123",
            Duration::days(1),
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
            example_kb_jwt(&signing_key, header).await.jwt().serialization(),
            &EcdsaDecodingKey::from(signing_key.verifying_key()),
            "aud",
            "def456",
            Duration::days(3),
        )
        .expect_err("should fail validation");
        assert_matches!(err, Error::Deserialization(msg) if msg == "invalid KB-JWT: unexpected nonce");
    }
}
