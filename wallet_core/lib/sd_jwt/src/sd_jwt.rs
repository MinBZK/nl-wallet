// Copyright 2020-2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;
use std::fmt::Display;

use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use indexmap::IndexMap;
use itertools::Itertools;
use jsonwebtoken::Algorithm;
use jsonwebtoken::Header;
use jsonwebtoken::Validation;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Map;
use serde_json::Value;
use serde_with::skip_serializing_none;

use crypto::EcdsaKeySend;
use jwt::jwk::jwk_to_p256;
use jwt::EcdsaDecodingKey;
use jwt::VerifiedJwt;
use wallet_common::spec::SpecOptional;

use crate::decoder::SdObjectDecoder;
use crate::disclosure::Disclosure;
use crate::error::Error;
use crate::error::Result;
use crate::hasher::Hasher;
use crate::hasher::SHA_ALG_NAME;
use crate::key_binding_jwt_claims::KeyBindingJwt;
use crate::key_binding_jwt_claims::KeyBindingJwtBuilder;
use crate::key_binding_jwt_claims::RequiredKeyBinding;

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct SdJwtClaims {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub _sd: Vec<String>,

    pub _sd_alg: Option<String>,

    // Even though we want this to be mandatory, we allow it to be optional in order for the examples from the spec
    // to parse.
    pub cnf: Option<RequiredKeyBinding>,

    #[serde(flatten)]
    pub properties: Map<String, Value>,
}

/// Representation of an SD-JWT of the format
/// `<Issuer-signed JWT>~<Disclosure 1>~<Disclosure 2>~...~<Disclosure N>~<optional KB-JWT>`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SdJwt {
    issuer_signed_jwt: VerifiedJwt<SdJwtClaims>,
    disclosures: IndexMap<String, Disclosure>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SdJwtPresentation {
    sd_jwt: SdJwt,
    key_binding_jwt: SpecOptional<KeyBindingJwt>,
}

impl SdJwtPresentation {
    /// Parses an SD-JWT into its components as [`SdJwt`].
    ///
    /// ## Error
    /// Returns [`Error::Deserialization`] if parsing fails.
    pub fn parse_and_verify(
        sd_jwt: &str,
        issuer_pubkey: &EcdsaDecodingKey,
        hasher: &dyn Hasher,
        kb_expected_aud: &str,
        kb_expected_nonce: &str,
        kb_iat_acceptance_window: Duration,
    ) -> Result<Self> {
        let (rest, kb_segment) = sd_jwt
            .rsplit_once("~")
            .map(|(head, tail)| {
                let head_with_tilde = &sd_jwt[..head.len() + 1];
                (head_with_tilde, tail)
            })
            .ok_or(Error::Deserialization(
                "SD-JWT format is invalid, no segments found".to_string(),
            ))?;

        let sd_jwt = SdJwt::parse_and_verify(rest, issuer_pubkey, hasher)?;

        let Some(RequiredKeyBinding::Jwk(jwk)) = sd_jwt.required_key_bind() else {
            return Err(Error::MissingJwkKeybinding);
        };

        let key_binding_jwt = KeyBindingJwt::parse_and_verify(
            kb_segment,
            &EcdsaDecodingKey::from(&jwk_to_p256(jwk)?),
            kb_expected_aud,
            kb_expected_nonce,
            kb_iat_acceptance_window,
        )?;

        Ok(Self {
            sd_jwt,
            key_binding_jwt: key_binding_jwt.into(),
        })
    }

    pub fn presentation(&self) -> String {
        let disclosures = self.sd_jwt.disclosures.values().map(ToString::to_string).join("~");
        let key_bindings = self.key_binding_jwt.as_ref().to_string();
        [self.sd_jwt.issuer_signed_jwt.jwt().clone().0, disclosures, key_bindings]
            .iter()
            .filter(|segment| !segment.is_empty())
            .join("~")
    }

    pub fn sd_jwt(&self) -> &SdJwt {
        &self.sd_jwt
    }

    pub fn key_binding_jwt(&self) -> &KeyBindingJwt {
        self.key_binding_jwt.as_ref()
    }
}

impl Display for SdJwtPresentation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&(self.presentation()))
    }
}

impl SdJwt {
    /// Creates a new [`SdJwt`] from its components.
    pub(crate) fn new(jwt: VerifiedJwt<SdJwtClaims>, disclosures: IndexMap<String, Disclosure>) -> Self {
        Self {
            issuer_signed_jwt: jwt,
            disclosures,
        }
    }

    pub fn header(&self) -> &Header {
        self.issuer_signed_jwt.header()
    }

    pub fn claims(&self) -> &SdJwtClaims {
        self.issuer_signed_jwt.payload()
    }

    pub fn disclosures(&self) -> &IndexMap<String, Disclosure> {
        &self.disclosures
    }

    pub fn required_key_bind(&self) -> Option<&RequiredKeyBinding> {
        self.claims().cnf.as_ref()
    }

    /// Serializes the components into the final SD-JWT.
    pub fn presentation(&self) -> String {
        let disclosures = self.disclosures.values().join("~");
        format!("{}~{}~", self.issuer_signed_jwt.jwt().clone().0, disclosures)
    }

    /// Parses an SD-JWT into its components as [`SdJwt`].
    ///
    /// ## Error
    /// Returns [`Error::Deserialization`] if parsing fails.
    pub fn parse_and_verify(sd_jwt: &str, pubkey: &EcdsaDecodingKey, hasher: &dyn Hasher) -> Result<Self> {
        if !sd_jwt.ends_with("~") {
            return Err(Error::Deserialization(
                "SD-JWT format is invalid, input doesn't and with '~'".to_string(),
            ));
        }

        let (sd_jwt_segment, disclosure_segments) = sd_jwt.split_once('~').ok_or(Error::Deserialization(
            "SD-JWT format is invalid, input doesn't contain a '~'".to_string(),
        ))?;

        let jwt = VerifiedJwt::try_new(sd_jwt_segment.parse()?, pubkey, &sd_jwt_validation())?;

        let disclosures = disclosure_segments
            .split("~")
            .filter(|segment| !segment.is_empty())
            .try_fold(IndexMap::new(), |mut acc, segment| {
                let disclosure = Disclosure::parse(segment)?;
                acc.insert(hasher.encoded_digest(disclosure.as_str()), disclosure);
                Ok::<_, Error>(acc)
            })?;

        Ok(Self {
            issuer_signed_jwt: jwt,
            disclosures,
        })
    }

    /// Prepares this [`SdJwt`] for a presentation, returning an [`SdJwtPresentationBuilder`].
    /// ## Errors
    /// - [`Error::InvalidHasher`] is returned if the provided `hasher`'s algorithm doesn't match the algorithm
    ///   specified by SD-JWT's `_sd_alg` claim. "sha-256" is used if the claim is missing.
    pub fn into_presentation(
        self,
        hasher: &dyn Hasher,
        key_binding_iat: DateTime<Utc>,
        key_binding_aud: String,
        key_binding_nonce: String,
        key_binding_alg: Algorithm,
    ) -> Result<SdJwtPresentationBuilder> {
        SdJwtPresentationBuilder::new(
            self,
            KeyBindingJwtBuilder::new(key_binding_iat, key_binding_aud, key_binding_nonce, key_binding_alg),
            hasher,
        )
    }

    /// Returns the JSON object obtained by replacing all disclosures into their
    /// corresponding JWT concealable claims.
    pub fn into_disclosed_object(self) -> Result<Map<String, Value>> {
        let decoder = SdObjectDecoder;
        let object = serde_json::to_value(self.claims())?;

        decoder.decode(object.as_object().unwrap(), &self.disclosures)
    }
}

impl Display for SdJwt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&(self.presentation()))
    }
}

#[derive(Clone)]
pub struct SdJwtPresentationBuilder<'a> {
    sd_jwt: SdJwt,
    kb_jwt_builder: KeyBindingJwtBuilder,
    disclosures: IndexMap<String, Disclosure>,
    removed_disclosures: Vec<Disclosure>,
    hasher: &'a dyn Hasher,
}

impl<'a> SdJwtPresentationBuilder<'a> {
    pub(crate) fn new(sd_jwt: SdJwt, kb_jwt_builder: KeyBindingJwtBuilder, hasher: &'a dyn Hasher) -> Result<Self> {
        let required_hasher = sd_jwt.claims()._sd_alg.as_deref().unwrap_or(SHA_ALG_NAME);
        if required_hasher != hasher.alg_name() {
            return Err(Error::InvalidHasher(format!(
                "hasher \"{}\" was provided, but \"{required_hasher} is required\"",
                hasher.alg_name()
            )));
        }

        let disclosures = sd_jwt.disclosures.clone();

        Ok(Self {
            sd_jwt,
            kb_jwt_builder,
            disclosures,
            removed_disclosures: vec![],
            hasher,
        })
    }

    /// Returns the resulting [`SdJwtPresentation`] together with all removed disclosures.
    /// ## Errors
    /// - Fails with [`Error::MissingKeyBindingJwt`] if this [`SdJwt`] requires a key binding but none was provided.
    pub async fn finish(self, signing_key: &impl EcdsaKeySend) -> Result<(SdJwtPresentation, Vec<Disclosure>)> {
        // Put everything back in its place.
        let SdJwtPresentationBuilder {
            mut sd_jwt,
            disclosures,
            removed_disclosures,
            kb_jwt_builder,
            ..
        } = self;
        sd_jwt.disclosures = disclosures;

        let key_binding_jwt = kb_jwt_builder.finish(&sd_jwt, self.hasher, signing_key).await?;

        let sd_jwt_presentation = SdJwtPresentation {
            sd_jwt,
            key_binding_jwt: key_binding_jwt.into(),
        };

        Ok((sd_jwt_presentation, removed_disclosures))
    }
}

pub(crate) fn sd_jwt_validation() -> Validation {
    let mut validation = Validation::new(Algorithm::ES256);
    validation.validate_exp = false;
    validation.validate_aud = false;
    validation.required_spec_claims = HashSet::new();
    validation
}

// TODO: [PVW-4138] Add tests for conceal functionality (and probably refactor)
#[cfg(test)]
mod test {
    use chrono::Duration;
    use rstest::rstest;

    use crate::examples::*;
    use crate::hasher::Sha256Hasher;
    use crate::sd_jwt::SdJwt;
    use crate::sd_jwt::SdJwtPresentation;

    #[rstest]
    #[case(SIMPLE_STRUCTURED_SD_JWT)]
    #[case(COMPLEX_STRUCTURED_SD_JWT)]
    #[case(SD_JWT_VC)]
    fn parse_various(#[case] encoded_sd_jwt: &str) {
        SdJwt::parse_and_verify(encoded_sd_jwt, &examples_sd_jwt_decoding_key(), &Sha256Hasher).unwrap();
    }

    #[test]
    fn parse_kb() {
        SdJwtPresentation::parse_and_verify(
            WITH_KB_SD_JWT,
            &examples_sd_jwt_decoding_key(),
            &Sha256Hasher,
            WITH_KB_SD_JWT_AUD,
            WITH_KB_SD_JWT_NONCE,
            Duration::days(36500),
        )
        .unwrap();
    }

    #[test]
    fn parse() {
        let sd_jwt =
            SdJwt::parse_and_verify(SIMPLE_STRUCTURED_SD_JWT, &examples_sd_jwt_decoding_key(), &Sha256Hasher).unwrap();
        assert_eq!(sd_jwt.disclosures.len(), 2);
    }

    #[test]
    fn round_trip_ser_des() {
        let sd_jwt =
            SdJwt::parse_and_verify(SIMPLE_STRUCTURED_SD_JWT, &examples_sd_jwt_decoding_key(), &Sha256Hasher).unwrap();
        assert_eq!(&sd_jwt.to_string(), SIMPLE_STRUCTURED_SD_JWT);
    }
}
