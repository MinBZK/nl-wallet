// Copyright 2020-2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;
use std::fmt::Display;
use std::iter::Peekable;
use std::ops::Deref;

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

use crate::decoder::SdObjectDecoder;
use crate::disclosure::Disclosure;
use crate::encoder::ARRAY_DIGEST_KEY;
use crate::encoder::DIGESTS_KEY;
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
    #[serde(default)]
    pub _sd: Vec<String>,
    pub _sd_alg: Option<String>,
    pub cnf: Option<RequiredKeyBinding>,
    #[serde(flatten)]
    pub properties: Map<String, Value>,
}

/// Representation of an SD-JWT of the format
/// `<Issuer-signed JWT>~<Disclosure 1>~<Disclosure 2>~...~<Disclosure N>~<optional KB-JWT>`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SdJwt {
    /// The Issuer-signed JWT part.
    issuer_signed_jwt: VerifiedJwt<SdJwtClaims>,
    /// The disclosures part.
    disclosures: Vec<Disclosure>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SdJwtPresentation {
    sd_jwt: SdJwt,

    /// The optional key binding JWT.
    key_binding_jwt: KeyBindingJwt,
}

impl SdJwtPresentation {
    /// Parses an SD-JWT into its components as [`SdJwt`].
    ///
    /// ## Error
    /// Returns [`Error::Deserialization`] if parsing fails.
    pub fn parse_and_verify(sd_jwt: &str, issuer_pubkey: &EcdsaDecodingKey) -> Result<Self> {
        let (rest, kb_segment) = sd_jwt
            .rsplit_once("~")
            .map(|(head, tail)| {
                let head_with_tilde = &sd_jwt[..head.len() + 1];
                (head_with_tilde, tail)
            })
            .ok_or(Error::Deserialization(
                "SD-JWT format is invalid, no segments found".to_string(),
            ))?;

        let sd_jwt = SdJwt::parse_and_verify(rest, issuer_pubkey)?;

        if let Some(RequiredKeyBinding::Jwk(jwk)) = sd_jwt.required_key_bind() {
            let key_binding_jwt =
                KeyBindingJwt::parse_and_verify(kb_segment, &EcdsaDecodingKey::from(&jwk_to_p256(jwk)?))?;

            Ok(Self {
                sd_jwt,
                key_binding_jwt,
            })
        } else {
            Err(Error::MissingJwkKeybinding)
        }
    }

    pub fn presentation(&self) -> String {
        let disclosures = self.sd_jwt.disclosures.iter().map(ToString::to_string).join("~");
        let key_bindings = self.key_binding_jwt.to_string();
        [self.sd_jwt.issuer_signed_jwt.jwt().clone().0, disclosures, key_bindings]
            .iter()
            .filter(|segment| !segment.is_empty())
            .join("~")
    }

    pub fn sd_jwt(&self) -> &SdJwt {
        &self.sd_jwt
    }

    pub fn key_binding_jwt(&self) -> &KeyBindingJwt {
        &self.key_binding_jwt
    }
}

impl Display for SdJwtPresentation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&(self.presentation()))
    }
}

impl SdJwt {
    /// Creates a new [`SdJwt`] from its components.
    pub(crate) fn new(jwt: VerifiedJwt<SdJwtClaims>, disclosures: Vec<Disclosure>) -> Self {
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

    pub fn disclosures(&self) -> &[Disclosure] {
        &self.disclosures
    }

    pub fn required_key_bind(&self) -> Option<&RequiredKeyBinding> {
        self.claims().cnf.as_ref()
    }

    /// Serializes the components into the final SD-JWT.
    pub fn presentation(&self) -> String {
        let disclosures = self.disclosures.iter().map(ToString::to_string).join("~");
        format!(
            "{}~",
            [self.issuer_signed_jwt.jwt().clone().0, disclosures]
                .iter()
                .filter(|segment| !segment.is_empty())
                .join("~")
        )
    }

    /// Parses an SD-JWT into its components as [`SdJwt`].
    ///
    /// ## Error
    /// Returns [`Error::Deserialization`] if parsing fails.
    pub fn parse_and_verify(sd_jwt: &str, pubkey: &EcdsaDecodingKey) -> Result<Self> {
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
            .map(Disclosure::parse)
            .try_collect()?;

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
        kb_jwt_builder: KeyBindingJwtBuilder,
        hasher: &dyn Hasher,
    ) -> Result<SdJwtPresentationBuilder> {
        SdJwtPresentationBuilder::new(self, kb_jwt_builder, hasher)
    }

    /// Returns the JSON object obtained by replacing all disclosures into their
    /// corresponding JWT concealable claims.
    pub fn into_disclosed_object(self, hasher: &dyn Hasher) -> Result<Map<String, Value>> {
        let decoder = SdObjectDecoder;
        let object = serde_json::to_value(self.claims())?;

        let disclosure_map = self
            .disclosures
            .into_iter()
            .map(|disclosure| (hasher.encoded_digest(disclosure.as_str()), disclosure))
            .collect();

        decoder.decode(object.as_object().unwrap(), &disclosure_map)
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
    object: Value,
    hasher: &'a dyn Hasher,
}

impl Deref for SdJwtPresentationBuilder<'_> {
    type Target = SdJwt;
    fn deref(&self) -> &Self::Target {
        &self.sd_jwt
    }
}

impl<'a> SdJwtPresentationBuilder<'a> {
    pub fn new(sd_jwt: SdJwt, kb_jwt_builder: KeyBindingJwtBuilder, hasher: &'a dyn Hasher) -> Result<Self> {
        let required_hasher = sd_jwt.claims()._sd_alg.as_deref().unwrap_or(SHA_ALG_NAME);
        if required_hasher != hasher.alg_name() {
            return Err(Error::InvalidHasher(format!(
                "hasher \"{}\" was provided, but \"{required_hasher} is required\"",
                hasher.alg_name()
            )));
        }

        let disclosures = sd_jwt
            .disclosures
            .clone()
            .into_iter()
            .map(|disclosure| (hasher.encoded_digest(disclosure.as_str()), disclosure))
            .collect();

        let payload = sd_jwt.issuer_signed_jwt.payload().clone();
        let object = {
            let sd = payload._sd.into_iter().map(Value::String).collect();
            let mut object = Value::Object(payload.properties);
            object
                .as_object_mut()
                .unwrap()
                .insert(DIGESTS_KEY.to_string(), Value::Array(sd));

            object
        };

        Ok(Self {
            sd_jwt,
            kb_jwt_builder,
            disclosures,
            removed_disclosures: vec![],
            object,
            hasher,
        })
    }

    /// Removes the disclosure for the property at `path`, concealing it.
    ///
    /// ## Notes
    /// - When concealing a claim more than one disclosure may be removed: the disclosure for the claim itself and the
    ///   disclosures for any concealable sub-claim.
    pub fn conceal(mut self, path: &str) -> Result<Self> {
        let path_segments = path.trim_start_matches('/').split('/').peekable();
        let digests_to_remove = conceal(&self.object, path_segments, &self.disclosures)?
            .into_iter()
            // needed, since some strings are borrowed for the lifetime of the borrow of `self.disclosures`.
            .map(ToOwned::to_owned)
            // needed, to drop borrow `self.disclosures`.
            .collect_vec();

        digests_to_remove
            .into_iter()
            .flat_map(|digest| self.disclosures.shift_remove(&digest))
            .for_each(|disclosure| self.removed_disclosures.push(disclosure));

        Ok(self)
    }

    /// Returns the resulting [`SdJwtPresentation`] together with all removed disclosures.
    /// ## Errors
    /// - Fails with [`Error::MissingKeyBindingJwt`] if this [`SdJwt`] requires a key binding but none was provided.
    pub async fn finish(
        self,
        alg: Algorithm,
        signing_key: &impl EcdsaKeySend,
    ) -> Result<(SdJwtPresentation, Vec<Disclosure>)> {
        // Put everything back in its place.
        let SdJwtPresentationBuilder {
            mut sd_jwt,
            disclosures,
            removed_disclosures,
            kb_jwt_builder,
            ..
        } = self;
        sd_jwt.disclosures = disclosures.into_values().collect_vec();

        let key_binding_jwt = kb_jwt_builder.finish(&sd_jwt, self.hasher, alg, signing_key).await?;

        let sd_jwt_presentation = SdJwtPresentation {
            sd_jwt,
            key_binding_jwt,
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

fn conceal<'p, 'o, 'd, I>(
    object: &'o Value,
    mut path: Peekable<I>,
    disclosures: &'d IndexMap<String, Disclosure>,
) -> Result<Vec<&'o str>>
where
    I: Iterator<Item = &'p str>,
    'd: 'o,
{
    let element_key = path
        .next()
        .ok_or_else(|| Error::InvalidPath("element at path doesn't exist or is not disclosable".to_string()))?;
    let has_next = path.peek().is_some();
    match object {
        // We are just traversing to a deeper part of the object.
        Value::Object(object) if has_next => {
            let next_object = object
                .get(element_key)
                .or_else(|| {
                    find_disclosure(object, element_key, disclosures)
                        .and_then(|digest| disclosures.get(digest))
                        .map(|disclosure| &disclosure.claim_value)
                })
                .ok_or_else(|| {
                    Error::InvalidPath("the referenced element doesn't exist or is not concealable".to_string())
                })?;

            conceal(next_object, path, disclosures)
        }
        // We reached the parent of the value we want to conceal.
        // Make sure its concealable by finding its disclosure.
        Value::Object(object) => {
            let digest = find_disclosure(object, element_key, disclosures).ok_or_else(|| {
                Error::InvalidPath("the referenced element doesn't exist or is not concealable".to_string())
            })?;
            let disclosure = disclosures.get(digest).unwrap();
            let mut sub_disclosures: Vec<&str> =
                get_all_sub_disclosures(&disclosure.claim_value, disclosures).collect();
            sub_disclosures.push(digest);
            Ok(sub_disclosures)
        }
        // Traversing an array
        Value::Array(arr) if has_next => {
            let index = element_key
                .parse::<usize>()
                .ok()
                .filter(|idx| arr.len() > *idx)
                .ok_or_else(|| Error::InvalidPath(String::default()))?;
            let next_object = arr.get(index).ok_or_else(|| {
                Error::InvalidPath("the referenced element doesn't exist or is not concealable".to_string())
            })?;

            conceal(next_object, path, disclosures)
        }
        // Concealing an array's entry.
        Value::Array(arr) => {
            let index = element_key
                .parse::<usize>()
                .ok()
                .filter(|idx| arr.len() > *idx)
                .ok_or_else(|| Error::InvalidPath(String::default()))?;
            let digest = arr
                .get(index)
                .unwrap()
                .as_object()
                .and_then(|entry| find_disclosure(entry, "", disclosures))
                .ok_or_else(|| {
                    Error::InvalidPath("the referenced element doesn't exist or is not concealable".to_string())
                })?;
            let disclosure = disclosures.get(digest).unwrap();
            let mut sub_disclosures: Vec<&str> =
                get_all_sub_disclosures(&disclosure.claim_value, disclosures).collect();
            sub_disclosures.push(digest);
            Ok(sub_disclosures)
        }
        _ => Err(Error::InvalidPath(String::default())),
    }
}

fn find_disclosure<'o>(
    object: &'o Map<String, Value>,
    key: &str,
    disclosures: &IndexMap<String, Disclosure>,
) -> Option<&'o str> {
    let maybe_disclosable_array_entry = || {
        object
            .get(ARRAY_DIGEST_KEY)
            .and_then(|value| value.as_str())
            .filter(|_| object.len() == 1)
    };
    // Try to find the digest for disclosable property `key` in
    // the `_sd` field of `object`.
    object
        .get(DIGESTS_KEY)
        .and_then(|value| value.as_array())
        .iter()
        .flat_map(|values| values.iter())
        .flat_map(|value| value.as_str())
        .find(|digest| {
            disclosures
                .get(*digest)
                .and_then(|disclosure| disclosure.claim_name.as_deref())
                .is_some_and(|name| name == key)
        })
        // If no result is found try checking `object` as a disclosable array entry.
        .or_else(maybe_disclosable_array_entry)
}

fn get_all_sub_disclosures<'v, 'd>(
    start: &'v Value,
    disclosures: &'d IndexMap<String, Disclosure>,
) -> Box<dyn Iterator<Item = &'v str> + 'v>
where
    'd: 'v,
{
    match start {
        // `start` is a JSON object, check if it has a "_sd" array + recursively
        // check all its properties
        Value::Object(object) => {
            let direct_sds = object
                .get(DIGESTS_KEY)
                .and_then(|sd| sd.as_array())
                .map(|sd| sd.iter())
                .unwrap_or_default()
                .flat_map(|value| value.as_str())
                .filter(|digest| disclosures.contains_key(*digest));
            let sub_sds = object
                .values()
                .flat_map(|value| get_all_sub_disclosures(value, disclosures));
            Box::new(itertools::chain!(direct_sds, sub_sds))
        }
        // `start` is a JSON array, check for disclosable values `{"...", <digest>}` +
        // recursively check all its values.
        Value::Array(arr) => {
            let mut digests = vec![];
            for value in arr {
                if let Some(Value::String(digest)) = value.get(ARRAY_DIGEST_KEY) {
                    if disclosures.contains_key(digest) {
                        digests.push(digest.as_str());
                    }
                } else {
                    get_all_sub_disclosures(value, disclosures).for_each(|digest| digests.push(digest));
                }
            }
            Box::new(digests.into_iter())
        }
        _ => Box::new(std::iter::empty()),
    }
}

#[cfg(test)]
mod test {
    use rstest::rstest;

    use crate::examples::*;
    use crate::sd_jwt::SdJwt;
    use crate::sd_jwt::SdJwtPresentation;

    #[rstest]
    #[case(SIMPLE_STRUCTURED_SD_JWT)]
    #[case(COMPLEX_STRUCTURED_SD_JWT)]
    #[case(SD_JWT_VC)]
    fn parse_various(#[case] encoded_sd_jwt: &str) {
        SdJwt::parse_and_verify(encoded_sd_jwt, &examples_sd_jwt_decoding_key()).unwrap();
    }

    #[test]
    fn parse_kb() {
        SdJwtPresentation::parse_and_verify(WITH_KB_SD_JWT, &examples_sd_jwt_decoding_key()).unwrap();
    }

    #[test]
    fn parse() {
        let sd_jwt = SdJwt::parse_and_verify(SIMPLE_STRUCTURED_SD_JWT, &examples_sd_jwt_decoding_key()).unwrap();
        assert_eq!(sd_jwt.disclosures.len(), 2);
    }

    #[test]
    fn round_trip_ser_des() {
        let sd_jwt = SdJwt::parse_and_verify(SIMPLE_STRUCTURED_SD_JWT, &examples_sd_jwt_decoding_key()).unwrap();
        assert_eq!(&sd_jwt.to_string(), SIMPLE_STRUCTURED_SD_JWT);
    }
}
