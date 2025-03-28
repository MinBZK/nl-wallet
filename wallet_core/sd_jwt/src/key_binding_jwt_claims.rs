// Copyright 2020-2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0
use std::borrow::Cow;
use std::fmt::Display;

use chrono::serde::ts_seconds;
use jsonwebtoken::jwk::Jwk;
use jsonwebtoken::Algorithm;
use jsonwebtoken::Header;
use jsonwebtoken::Validation;
use serde::Deserialize;
use serde::Serialize;
use serde_with::chrono::DateTime;
use serde_with::chrono::Utc;

use crypto::EcdsaKeySend;
use jwt::EcdsaDecodingKey;
use jwt::Jwt;
use jwt::VerifiedJwt;

use crate::error;
use crate::error::Error;
use crate::hasher::Hasher;
use crate::hasher::SHA_ALG_NAME;
use crate::sd_jwt::SdJwt;

pub const KB_JWT_HEADER_TYP: &str = "kb+jwt";

/// Representation of a [KB-JWT](https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-12.html#name-key-binding-jwt).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyBindingJwt(VerifiedJwt<KeyBindingJwtClaims>);

impl Display for KeyBindingJwt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0.jwt().0)
    }
}

impl KeyBindingJwt {
    pub fn parse(s: &str, pub_key: &EcdsaDecodingKey) -> error::Result<Self> {
        let jwt: Jwt<KeyBindingJwtClaims> = s.into();

        let (header, _) = jwt.parse_and_verify_with_header(pub_key, &kb_jwt_validation())?;

        let valid_jwt_type = &header.typ.is_some_and(|typ| typ == KB_JWT_HEADER_TYP);
        if !valid_jwt_type {
            return Err(Error::Deserialization(format!(
                "invalid KB-JWT: typ must be \"{KB_JWT_HEADER_TYP}\""
            )));
        }

        let verified_jwt = VerifiedJwt::<KeyBindingJwtClaims>::try_new(jwt, pub_key, &kb_jwt_validation())?;

        Ok(Self(verified_jwt))
    }

    /// Returns a [`KeyBindingJwtBuilder`] that allows the creation of a [`KeyBindingJwt`].
    pub fn builder() -> KeyBindingJwtBuilder {
        KeyBindingJwtBuilder::default()
    }

    /// Returns a reference to this [`KeyBindingJwt`] claim set.
    pub fn claims(&self) -> &KeyBindingJwtClaims {
        self.0.payload()
    }
}

fn kb_jwt_validation() -> Validation {
    let mut validation = Validation::new(Algorithm::ES256);
    validation.validate_exp = false;
    validation.validate_aud = false;
    // validator.set_audience(&["https://verifier.example.org"]); // TODO: validate aud?
    validation.set_required_spec_claims(&["aud"]);
    validation
}

/// Builder-style struct to ease the creation of an [`KeyBindingJwt`].
#[derive(Debug, Clone)]
pub struct KeyBindingJwtBuilder {
    header: Header,
    iat: Option<DateTime<Utc>>,
    aud: Option<String>,
    nonce: Option<String>,
}

impl Default for KeyBindingJwtBuilder {
    fn default() -> Self {
        let header = Header {
            typ: Some(String::from(KB_JWT_HEADER_TYP)),
            ..Default::default()
        };

        Self {
            header,
            iat: None,
            aud: None,
            nonce: None,
        }
    }
}

impl KeyBindingJwtBuilder {
    /// Sets the [iat](https://www.rfc-editor.org/rfc/rfc7519.html#section-4.1.6) property.
    pub fn iat(mut self, iat: DateTime<Utc>) -> Self {
        self.iat = Some(iat);
        self
    }

    /// Sets the [aud](https://www.rfc-editor.org/rfc/rfc7519.html#section-4.1.3) property.
    pub fn aud<'a, S>(mut self, aud: S) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        self.aud = Some(aud.into().into_owned());
        self
    }

    /// Sets the `nonce` property.
    pub fn nonce<'a, S>(mut self, nonce: S) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        self.nonce = Some(nonce.into().into_owned());
        self
    }

    /// Builds an [`KeyBindingJwt`] from the data provided to builder.
    pub async fn finish(
        self,
        sd_jwt: &SdJwt,
        hasher: &dyn Hasher,
        alg: Algorithm,
        signing_key: &impl EcdsaKeySend,
    ) -> Result<KeyBindingJwt, Error> {
        let sd_hash = hasher.encoded_digest(&sd_jwt.to_string());

        let claims = KeyBindingJwtClaims {
            iat: self.iat.ok_or(Error::MissingRequiredProperty(String::from("iat")))?,
            aud: self.aud.ok_or(Error::MissingRequiredProperty(String::from("aud")))?,
            nonce: self
                .nonce
                .ok_or(Error::MissingRequiredProperty(String::from("nonce")))?,
            sd_hash,
        };

        if sd_jwt.key_binding_jwt().is_some() {
            return Err(Error::DataTypeMismatch(
                "the provided SD-JWT already has a KB-JWT attached".to_string(),
            ));
        }

        if sd_jwt.claims()._sd_alg.as_deref().unwrap_or(SHA_ALG_NAME) != hasher.alg_name() {
            return Err(Error::InvalidHasher(format!(
                "invalid hashing algorithm \"{}\"",
                hasher.alg_name()
            )));
        }

        let mut header = self.header;
        header.alg = alg;

        let verified_jwt = VerifiedJwt::sign(&claims, &header, signing_key, &kb_jwt_validation()).await?;

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
    use chrono::Utc;
    use jsonwebtoken::Algorithm;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;

    use crate::error::Error;
    use crate::examples;
    use crate::examples::examples_sd_jwt_decoding_key;
    use crate::examples::SIMPLE_STRUCTURED_SD_JWT;
    use crate::hasher::Hasher;
    use crate::hasher::Sha256Hasher;
    use crate::key_binding_jwt_claims::KeyBindingJwtBuilder;
    use crate::sd_jwt::SdJwt;

    #[tokio::test]
    async fn test_key_binding_jwt_builder() {
        let sd_jwt = SdJwt::parse(SIMPLE_STRUCTURED_SD_JWT, &examples_sd_jwt_decoding_key(), None).unwrap();

        let signing_key = SigningKey::random(&mut OsRng);
        let hasher = Sha256Hasher::new();

        let iat = Utc::now();

        let kb_jwt = KeyBindingJwtBuilder::default()
            .aud("receiver")
            .iat(iat)
            .nonce("abc123")
            .finish(&sd_jwt, &hasher, Algorithm::ES256, &signing_key)
            .await
            .unwrap();

        let sd_hash = hasher.encoded_digest(SIMPLE_STRUCTURED_SD_JWT);

        assert_eq!(iat.timestamp(), kb_jwt.claims().iat.timestamp());
        assert_eq!(String::from("receiver"), kb_jwt.claims().aud);
        assert_eq!(String::from("abc123"), kb_jwt.claims().nonce);
        assert_eq!(sd_hash, kb_jwt.claims().sd_hash);
        assert_eq!(Some(String::from("kb+jwt")), kb_jwt.0.header().typ);
        assert_eq!(Algorithm::ES256, kb_jwt.0.header().alg);
    }

    #[tokio::test]
    async fn test_algorithm_should_match_sd_jwt() {
        let sd_jwt = examples::simple_structured_sd_jwt();

        let signing_key = SigningKey::random(&mut OsRng);

        struct TestHasher;
        impl Hasher for TestHasher {
            fn digest(&self, _input: &[u8]) -> Vec<u8> {
                vec![]
            }

            fn alg_name(&self) -> &str {
                "test_alg"
            }
        }

        let result = KeyBindingJwtBuilder::default()
            .aud("receiver")
            .iat(Utc::now())
            .nonce("abc123")
            .finish(&sd_jwt, &TestHasher, Algorithm::ES256, &signing_key)
            .await;

        assert_matches!(result, Err(Error::InvalidHasher(_)));
    }

    #[tokio::test]
    async fn test_should_error_for_existing_kb_sd_jwt() {
        let sd_jwt = examples::sd_jwt_kb();

        let signing_key = SigningKey::random(&mut OsRng);
        let hasher = Sha256Hasher::new();

        let iat = Utc::now();

        let result = KeyBindingJwtBuilder::default()
            .aud("receiver")
            .iat(iat)
            .nonce("abc123")
            .finish(&sd_jwt, &hasher, Algorithm::ES256, &signing_key)
            .await;

        assert_matches!(result, Err(Error::DataTypeMismatch(_)));
    }
}
