// Copyright 2020-2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0
use std::borrow::Cow;
use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;

use crate::error::Error;
use crate::hasher::Hasher;
use crate::hasher::SHA_ALG_NAME;
use crate::jwt::Jwt;
use crate::sd_jwt::SdJwt;
use crate::signer::JsonObject;
use crate::signer::JwsSigner;

use anyhow::Context as _;
use chrono::serde::ts_seconds;
use derive_more::Display;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use serde_with::chrono::DateTime;
use serde_with::chrono::Utc;

pub const KB_JWT_HEADER_TYP: &str = "kb+jwt";

/// Representation of a [KB-JWT](https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-12.html#name-key-binding-jwt).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyBindingJwt(Jwt<KeyBindingJwtClaims>);

impl Display for KeyBindingJwt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl FromStr for KeyBindingJwt {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let jwt = Jwt::<KeyBindingJwtClaims>::from_str(s)?;
        let valid_jwt_type = jwt.header.get("typ").is_some_and(|typ| typ == KB_JWT_HEADER_TYP);
        if !valid_jwt_type {
            return Err(Error::Deserialization(format!(
                "invalid KB-JWT: typ must be \"{KB_JWT_HEADER_TYP}\""
            )));
        }
        let valid_alg = jwt.header.get("alg").is_some_and(|alg| alg != "none");
        if !valid_alg {
            return Err(Error::Deserialization(
                "invalid KB-JWT: alg must be set and cannot be \"none\"".to_string(),
            ));
        }

        Ok(Self(jwt))
    }
}

impl KeyBindingJwt {
    /// Returns a [`KeyBindingJwtBuilder`] that allows the creation of a [`KeyBindingJwt`].
    pub fn builder() -> KeyBindingJwtBuilder {
        KeyBindingJwtBuilder::default()
    }

    /// Returns a reference to this [`KeyBindingJwt`] claim set.
    pub fn claims(&self) -> &KeyBindingJwtClaims {
        &self.0.claims
    }
}

/// Builder-style struct to ease the creation of an [`KeyBindingJwt`].
#[derive(Debug, Clone)]
pub struct KeyBindingJwtBuilder {
    header: JsonObject,
    payload: JsonObject,
}

impl Default for KeyBindingJwtBuilder {
    fn default() -> Self {
        let mut header = JsonObject::default();
        header.insert(String::from("typ"), String::from(KB_JWT_HEADER_TYP).into());

        Self {
            header,
            payload: JsonObject::default(),
        }
    }
}

impl KeyBindingJwtBuilder {
    /// Sets the [iat](https://www.rfc-editor.org/rfc/rfc7519.html#section-4.1.6) property.
    pub fn iat(mut self, iat: DateTime<Utc>) -> Self {
        self.payload.insert("iat".to_string(), iat.timestamp().into());
        self
    }

    /// Sets the [aud](https://www.rfc-editor.org/rfc/rfc7519.html#section-4.1.3) property.
    pub fn aud<'a, S>(mut self, aud: S) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        self.payload.insert("aud".to_string(), aud.into().into_owned().into());
        self
    }

    /// Sets the `nonce` property.
    pub fn nonce<'a, S>(mut self, nonce: S) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        self.payload
            .insert("nonce".to_string(), nonce.into().into_owned().into());
        self
    }

    /// Builds an [`KeyBindingJwt`] from the data provided to builder.
    pub async fn finish<S>(
        self,
        sd_jwt: &SdJwt,
        hasher: &dyn Hasher,
        alg: DigitalSignaturAlgorithm,
        signer: &S,
    ) -> Result<KeyBindingJwt, Error>
    where
        S: JwsSigner,
    {
        let mut claims = self.payload;

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

        let sd_hash = hasher.encoded_digest(&sd_jwt.to_string());
        claims.insert("sd_hash".to_string(), sd_hash.into());

        let mut header = self.header;
        header.insert("alg".to_string(), alg.to_string().into());

        // Validate claims
        let parsed_claims = serde_json::from_value::<KeyBindingJwtClaims>(claims.clone().into())
            .map_err(|e| Error::Deserialization(format!("invalid KB-JWT claims: {e}")))?;

        let jws = signer
            .sign(&header, &claims)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))
            .and_then(|jws_bytes| String::from_utf8(jws_bytes).context("invalid JWS"))
            .map_err(|e| Error::JwsSignerFailure(e.to_string()))?;

        Ok(KeyBindingJwt(Jwt {
            header,
            claims: parsed_claims,
            jws,
        }))
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
    #[serde(flatten)]
    properties: JsonObject,
}

impl Deref for KeyBindingJwtClaims {
    type Target = JsonObject;
    fn deref(&self) -> &Self::Target {
        &self.properties
    }
}

/// Proof of possession of a given key. See [RFC7800](https://www.rfc-editor.org/rfc/rfc7800.html#section-3) for more details.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum RequiredKeyBinding {
    /// Json Web Key (JWK).
    Jwk(JsonObject),
    /// Encoded JWK in its compact serialization form.
    Jwe(String),
    /// Key ID.
    Kid(String),
    /// JWK from a JWK set identified by `kid`.
    Jwu {
        /// URL of the JWK Set.
        jwu: String,
        /// kid of the referenced JWK.
        kid: String,
    },
    /// Non standard key-bind.
    #[serde(untagged)]
    Custom(Value),
}

/// JSON Web Algorithms (JWA) [RFC5718](https://www.rfc-editor.org/rfc/rfc7518.html)
#[derive(Debug, Copy, Clone, Display, Eq, PartialEq, Serialize, Deserialize)]
pub enum DigitalSignaturAlgorithm {
    ES256,
    HS256,
}

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;
    use async_trait::async_trait;
    use chrono::Utc;
    use josekit::jwk::KeyPair;
    use josekit::jws::alg::ecdsa::EcdsaJwsSigner;
    use josekit::jws::JwsHeader;
    use josekit::jws::ES256;
    use josekit::jwt;
    use josekit::jwt::JwtPayload;

    use crate::error::Error;
    use crate::hasher::Hasher;
    use crate::hasher::Sha256Hasher;
    use crate::key_binding_jwt_claims::DigitalSignaturAlgorithm;
    use crate::key_binding_jwt_claims::KeyBindingJwtBuilder;
    use crate::sd_jwt::SdJwt;
    use crate::signer::JsonObject;
    use crate::signer::JwsSigner;

    // Taken from https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-17.html#name-simple-structured-sd-jwt
    const SIMPLE_STRUCTURED_SD_JWT: &str = include_str!("../examples/sd_jwt/simple_structured.jwt");

    // Taken from https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-17.html#name-presentation
    const WITH_KB_SD_JWT: &str = include_str!("../examples/sd_jwt/with_kb.jwt");

    struct EcdsaSigner(EcdsaJwsSigner);

    #[async_trait]
    impl JwsSigner for EcdsaSigner {
        type Error = josekit::JoseError;

        async fn sign(&self, header: &JsonObject, payload: &JsonObject) -> Result<Vec<u8>, Self::Error> {
            let header = JwsHeader::from_map(header.clone())?;
            let payload = JwtPayload::from_map(payload.clone())?;

            jwt::encode_with_signer(&payload, &header, &self.0).map(String::into_bytes)
        }
    }

    #[tokio::test]
    async fn test_key_binding_jwt_builder() {
        let sd_jwt = SdJwt::parse(SIMPLE_STRUCTURED_SD_JWT).unwrap();

        let key_pair = ES256.generate_key_pair().unwrap();
        let signer = EcdsaSigner(ES256.signer_from_der(key_pair.to_der_private_key()).unwrap());
        let hasher = Sha256Hasher::new();

        let iat = Utc::now();

        let kb_jwt = KeyBindingJwtBuilder::default()
            .aud("receiver")
            .iat(iat)
            .nonce("abc123")
            .finish(&sd_jwt, &hasher, DigitalSignaturAlgorithm::ES256, &signer)
            .await
            .unwrap();

        let sd_hash = hasher.encoded_digest(SIMPLE_STRUCTURED_SD_JWT);

        assert_eq!(iat.timestamp(), kb_jwt.claims().iat.timestamp());
        assert_eq!(String::from("receiver"), kb_jwt.claims().aud);
        assert_eq!(String::from("abc123"), kb_jwt.claims().nonce);
        assert_eq!(sd_hash, kb_jwt.claims().sd_hash);
        assert_eq!("kb+jwt", kb_jwt.0.header.get("typ").unwrap());
        assert_eq!("ES256", kb_jwt.0.header.get("alg").unwrap());
    }

    #[tokio::test]
    async fn test_algorithm_should_match_sd_jwt() {
        let sd_jwt = SdJwt::parse(SIMPLE_STRUCTURED_SD_JWT).unwrap();

        let key_pair = ES256.generate_key_pair().unwrap();
        let signer = EcdsaSigner(ES256.signer_from_der(key_pair.to_der_private_key()).unwrap());

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
            .finish(&sd_jwt, &TestHasher, DigitalSignaturAlgorithm::ES256, &signer)
            .await;

        assert_matches!(result, Err(Error::InvalidHasher(_)));
    }

    #[tokio::test]
    async fn test_should_error_for_existing_kb_sd_jwt() {
        let sd_jwt = SdJwt::parse(WITH_KB_SD_JWT).unwrap();

        let key_pair = ES256.generate_key_pair().unwrap();
        let signer = EcdsaSigner(ES256.signer_from_der(key_pair.to_der_private_key()).unwrap());
        let hasher = Sha256Hasher::new();

        let iat = Utc::now();

        let result = KeyBindingJwtBuilder::default()
            .aud("receiver")
            .iat(iat)
            .nonce("abc123")
            .finish(&sd_jwt, &hasher, DigitalSignaturAlgorithm::ES256, &signer)
            .await;

        assert_matches!(result, Err(Error::DataTypeMismatch(_)));
    }
}
