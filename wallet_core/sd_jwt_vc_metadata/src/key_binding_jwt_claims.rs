// Copyright 2020-2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0
use std::borrow::Cow;
use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;

use anyhow::Context as _;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::error::Error;
use crate::hasher::Hasher;
use crate::hasher::SHA_ALG_NAME;
use crate::jwt::Jwt;
use crate::sd_jwt::SdJwt;
use crate::signer::JsonObject;
use crate::signer::JwsSigner;

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
            return Err(Error::DeserializationError(format!(
                "invalid KB-JWT: typ must be \"{KB_JWT_HEADER_TYP}\""
            )));
        }
        let valid_alg = jwt.header.get("alg").is_some_and(|alg| alg != "none");
        if !valid_alg {
            return Err(Error::DeserializationError(
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
#[derive(Debug, Default, Clone)]
pub struct KeyBindingJwtBuilder {
    header: JsonObject,
    payload: JsonObject,
}

impl KeyBindingJwtBuilder {
    /// Creates a new [`KeyBindingJwtBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new [`KeyBindingJwtBuilder`] using `object` as its payload.
    pub fn from_object(object: JsonObject) -> Self {
        Self {
            header: JsonObject::default(),
            payload: object,
        }
    }

    /// Sets the JWT's header.
    pub fn header(mut self, header: JsonObject) -> Self {
        self.header = header;
        self
    }

    /// Sets the [iat](https://www.rfc-editor.org/rfc/rfc7519.html#section-4.1.6) property.
    pub fn iat(mut self, iat: i64) -> Self {
        self.payload.insert("iat".to_string(), iat.into());
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

    /// Inserts a given property with key `name` and value `value` in the payload.
    pub fn insert_property(mut self, name: &str, value: Value) -> Self {
        self.payload.insert(name.to_string(), value);
        self
    }

    /// Builds an [`KeyBindingJwt`] from the data provided to builder.
    pub async fn finish<S>(
        self,
        sd_jwt: &SdJwt,
        hasher: &dyn Hasher,
        alg: &str,
        signer: &S,
    ) -> Result<KeyBindingJwt, Error>
    where
        S: JwsSigner,
    {
        let mut claims = self.payload;
        if alg == "none" {
            return Err(Error::DataTypeMismatch(
                "A KeyBindingJwt cannot use algorithm \"none\"".to_string(),
            ));
        }

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
        header.insert("alg".to_string(), alg.to_owned().into());
        header
            .entry("typ")
            .or_insert_with(|| KB_JWT_HEADER_TYP.to_owned().into());

        // Validate claims
        let parsed_claims = serde_json::from_value::<KeyBindingJwtClaims>(claims.clone().into())
            .map_err(|e| Error::DeserializationError(format!("invalid KB-JWT claims: {e}")))?;

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
    pub iat: i64,
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
