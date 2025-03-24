// Copyright 2020-2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::fmt::Display;
use std::iter::Peekable;
use std::ops::Deref;
use std::ops::DerefMut;
use std::str::FromStr;

use indexmap::IndexMap;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use wallet_common::vec_at_least::VecNonEmpty;

use crate::decoder::SdObjectDecoder;
use crate::disclosure::Disclosure;
use crate::encoder::ARRAY_DIGEST_KEY;
use crate::encoder::DIGESTS_KEY;
use crate::error::Error;
use crate::error::Result;
use crate::hasher::Hasher;
use crate::hasher::SHA_ALG_NAME;
use crate::jwt::Jwt;
use crate::key_binding_jwt_claims::DigitalSignaturAlgorithm;
use crate::key_binding_jwt_claims::KeyBindingJwt;
use crate::key_binding_jwt_claims::KeyBindingJwtBuilder;
use crate::key_binding_jwt_claims::RequiredKeyBinding;
use crate::signer::JsonObject;
use crate::signer::JwsSigner;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct SdJwtClaims {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub _sd: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _sd_alg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cnf: Option<RequiredKeyBinding>,
    #[serde(flatten)]
    properties: JsonObject,
}

impl Deref for SdJwtClaims {
    type Target = JsonObject;

    fn deref(&self) -> &Self::Target {
        &self.properties
    }
}

impl DerefMut for SdJwtClaims {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.properties
    }
}

/// Representation of an SD-JWT of the format
/// `<Issuer-signed JWT>~<Disclosure 1>~<Disclosure 2>~...~<Disclosure N>~<optional KB-JWT>`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SdJwt {
    /// The JWT part.
    jwt: Jwt<SdJwtClaims>,
    /// The disclosures part.
    disclosures: Vec<Disclosure>,
    /// The optional key binding JWT.
    key_binding_jwt: Option<KeyBindingJwt>,
}

impl SdJwt {
    /// Creates a new [`SdJwt`] from its components.
    pub(crate) fn new(
        jwt: Jwt<SdJwtClaims>,
        disclosures: Vec<Disclosure>,
        key_binding_jwt: Option<KeyBindingJwt>,
    ) -> Self {
        Self {
            jwt,
            disclosures,
            key_binding_jwt,
        }
    }

    pub fn header(&self) -> &JsonObject {
        &self.jwt.header
    }

    pub fn claims(&self) -> &SdJwtClaims {
        &self.jwt.claims
    }

    /// Returns a mutable reference to this SD-JWT's claims.
    /// ## Warning
    /// Modifying the claims might invalidate the signature.
    /// Use this method carefully.
    pub fn claims_mut(&mut self) -> &mut SdJwtClaims {
        &mut self.jwt.claims
    }

    pub fn disclosures(&self) -> &[Disclosure] {
        &self.disclosures
    }

    pub fn required_key_bind(&self) -> Option<&RequiredKeyBinding> {
        self.claims().cnf.as_ref()
    }

    pub fn key_binding_jwt(&self) -> Option<&KeyBindingJwt> {
        self.key_binding_jwt.as_ref()
    }

    /// Serializes the components into the final SD-JWT.
    ///
    /// ## Error
    /// Returns [`Error::Deserialization`] if parsing fails.
    pub fn presentation(&self) -> String {
        let disclosures = self.disclosures.iter().map(ToString::to_string).join("~");
        let key_bindings = self
            .key_binding_jwt
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default();
        if disclosures.is_empty() {
            format!("{}~{}", self.jwt, key_bindings)
        } else {
            format!("{}~{}~{}", self.jwt, disclosures, key_bindings)
        }
    }

    /// Parses an SD-JWT into its components as [`SdJwt`].
    pub fn parse(sd_jwt: &str) -> Result<Self> {
        let sd_segments: VecNonEmpty<&str> = sd_jwt
            .split('~')
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_err| Error::Deserialization("SD-JWT format is invalid, less than 2 segments".to_string()))?;
        let num_of_segments = sd_segments.len().get();

        let jwt = sd_segments.first().parse()?;

        let disclosures = sd_segments[1..num_of_segments - 1]
            .iter()
            .map(|s| Disclosure::parse(s))
            .try_collect()?;

        let key_binding_jwt = Some(sd_segments.last())
            .filter(|segment| !segment.is_empty())
            .map(|segment| segment.parse())
            .transpose()?;

        Ok(Self {
            jwt,
            disclosures,
            key_binding_jwt,
        })
    }

    /// Prepares this [`SdJwt`] for a presentation, returning an [`SdJwtPresentationBuilder`].
    /// ## Errors
    /// - [`Error::InvalidHasher`] is returned if the provided `hasher`'s algorithm doesn't match the algorithm
    ///   specified by SD-JWT's `_sd_alg` claim. "sha-256" is used if the claim is missing.
    pub fn into_presentation(self, hasher: &dyn Hasher) -> Result<SdJwtPresentationBuilder> {
        SdJwtPresentationBuilder::new(self, hasher)
    }

    /// Returns the JSON object obtained by replacing all disclosures into their
    /// corresponding JWT concealable claims.
    pub fn into_disclosed_object(self, hasher: &dyn Hasher) -> Result<JsonObject> {
        let decoder = SdObjectDecoder;
        let object = serde_json::to_value(self.claims()).unwrap();

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

impl FromStr for SdJwt {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[derive(Debug, Clone)]
pub struct SdJwtPresentationBuilder {
    sd_jwt: SdJwt,
    kb_jwt_builder: Option<KeyBindingJwtBuilder>,
    disclosures: IndexMap<String, Disclosure>,
    removed_disclosures: Vec<Disclosure>,
    object: Value,
}

impl Deref for SdJwtPresentationBuilder {
    type Target = SdJwt;
    fn deref(&self) -> &Self::Target {
        &self.sd_jwt
    }
}

impl SdJwtPresentationBuilder {
    pub fn new(mut sd_jwt: SdJwt, hasher: &dyn Hasher) -> Result<Self> {
        let required_hasher = sd_jwt.claims()._sd_alg.as_deref().unwrap_or(SHA_ALG_NAME);
        if required_hasher != hasher.alg_name() {
            return Err(Error::InvalidHasher(format!(
                "hasher \"{}\" was provided, but \"{required_hasher} is required\"",
                hasher.alg_name()
            )));
        }

        let disclosures = std::mem::take(&mut sd_jwt.disclosures)
            .into_iter()
            .map(|disclosure| (hasher.encoded_digest(disclosure.as_str()), disclosure))
            .collect();

        let object = {
            let sd = std::mem::take(&mut sd_jwt.jwt.claims._sd)
                .into_iter()
                .map(Value::String)
                .collect();
            let mut object = Value::Object(std::mem::take(&mut sd_jwt.jwt.claims.properties));
            object
                .as_object_mut()
                .unwrap()
                .insert(DIGESTS_KEY.to_string(), Value::Array(sd));

            object
        };

        Ok(Self {
            sd_jwt,
            kb_jwt_builder: None,
            disclosures,
            removed_disclosures: vec![],
            object,
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

    // /// Adds a [`KeyBindingJwt`] to this [`SdJwt`]'s presentation.
    pub fn attach_key_binding_jwt(mut self, kb_jwt: KeyBindingJwtBuilder) -> Self {
        self.kb_jwt_builder = Some(kb_jwt);
        self
    }

    /// Returns the resulting [`SdJwt`] together with all removed disclosures.
    /// ## Errors
    /// - Fails with [`Error::MissingKeyBindingJwt`] if this [`SdJwt`] requires a key binding but none was provided.
    pub fn finish(self) -> Result<(SdJwt, Vec<Disclosure>)> {
        if self.sd_jwt.required_key_bind().is_some() && self.kb_jwt_builder.is_none() {
            return Err(Error::MissingKeyBindingJwt);
        }

        // Put everything back in its place.
        let SdJwtPresentationBuilder {
            mut sd_jwt,
            disclosures,
            removed_disclosures,
            object,
            ..
        } = self;
        sd_jwt.disclosures = disclosures.into_values().collect_vec();

        let Value::Object(mut obj) = object else {
            unreachable!();
        };
        let Value::Array(sd) = obj.remove(DIGESTS_KEY).unwrap_or(Value::Array(vec![])) else {
            unreachable!()
        };
        sd_jwt.jwt.claims._sd = sd
            .into_iter()
            .map(|value| {
                if let Value::String(s) = value {
                    s
                } else {
                    unreachable!()
                }
            })
            .collect();
        sd_jwt.jwt.claims.properties = obj;

        Ok((sd_jwt, removed_disclosures))
    }

    /// Returns the resulting [`SdJwt`] together with all removed disclosures.
    /// ## Errors
    /// - Fails with [`Error::MissingKeyBindingJwt`] if this [`SdJwt`] requires a key binding but none was provided.
    pub async fn finish_with_key_binding<S>(
        self,
        hasher: &dyn Hasher,
        alg: DigitalSignaturAlgorithm,
        signer: &S,
    ) -> Result<(SdJwt, Vec<Disclosure>)>
    where
        S: JwsSigner,
    {
        if self.sd_jwt.required_key_bind().is_some() && self.kb_jwt_builder.is_none() {
            return Err(Error::MissingKeyBindingJwt);
        }

        let kb_builder = self.kb_jwt_builder.clone().unwrap();
        let (mut sd_jwt, removed_disclosures) = self.finish()?;

        let kb_jwt = kb_builder.finish(&sd_jwt, hasher, alg, signer).await?;
        sd_jwt.key_binding_jwt = Some(kb_jwt);

        Ok((sd_jwt, removed_disclosures))
    }
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
    object: &'o JsonObject,
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
    use crate::sd_jwt::SdJwt;

    use rstest::rstest;

    // Taken from https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-17.html#name-simple-structured-sd-jwt
    const SIMPLE_STRUCTURED_SD_JWT: &str = include_str!("../examples/sd_jwt/simple_structured.jwt");

    // Taken from https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-17.html#name-complex-structured-sd-jwt
    const COMPLEX_STRUCTURED_SD_JWT: &str = include_str!("../examples/sd_jwt/complex_structured.jwt");

    // Taken from https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-17.html#name-sd-jwt-based-verifiable-cre
    const SD_JWT_VC: &str = include_str!("../examples/sd_jwt/sd_jwt_vc.jwt");

    // Taken from https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-17.html#name-presentation
    const WITH_KB_SD_JWT: &str = include_str!("../examples/sd_jwt/with_kb.jwt");

    #[rstest]
    #[case(SIMPLE_STRUCTURED_SD_JWT)]
    #[case(COMPLEX_STRUCTURED_SD_JWT)]
    #[case(SD_JWT_VC)]
    #[case(WITH_KB_SD_JWT)]
    fn parse_various(#[case] encoded_sd_jwt: &str) {
        SdJwt::parse(encoded_sd_jwt).unwrap();
    }

    #[test]
    fn parse() {
        let sd_jwt = SdJwt::parse(SIMPLE_STRUCTURED_SD_JWT).unwrap();
        assert_eq!(sd_jwt.disclosures.len(), 2);
        assert!(sd_jwt.key_binding_jwt.is_none());
    }

    #[test]
    fn round_trip_ser_des() {
        let sd_jwt = SdJwt::parse(SIMPLE_STRUCTURED_SD_JWT).unwrap();
        assert_eq!(&sd_jwt.to_string(), SIMPLE_STRUCTURED_SD_JWT);
    }
}
