use std::collections::HashSet;
use std::num::TryFromIntError;

use attestation_types::claim_path::ClaimPath;
use base64::prelude::*;
use chrono::NaiveDate;
use derive_more::AsRef;
use derive_more::Display;
use derive_more::From;
use indexmap::IndexMap;
use itertools::Itertools;
use mdoc::iso::mdocs::Entry;
use mdoc::iso::mdocs::NameSpace;
use sd_jwt::claims::ArrayClaim;
use sd_jwt::claims::ClaimNameError;
use sd_jwt::claims::ClaimValue;
use sd_jwt::claims::ObjectClaims;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Number;
use serde_with::base64::Base64;
use serde_with::base64::UrlSafe;
use serde_with::formats::Unpadded;
use serde_with::serde_as;
use utils::vec_at_least::VecNonEmpty;

#[serde_as]
#[derive(Debug, Clone, Display, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "lowercase")]
pub enum Attribute {
    Null,
    Number(serde_json::Number),
    Bool(bool),
    Text(String),
    #[display("[{}]", _0.iter().join(", "))]
    Array(Vec<Attribute>),
    #[display("{{{}}}", _0.iter().map(|(k, v)| format!("{}: {}", k, v)).join(", "))]
    Object(IndexMap<String, Attribute>),

    // mdoc only
    Date(chrono::NaiveDate),
    #[display("{}", BASE64_URL_SAFE_NO_PAD.encode(_0))]
    Bytes(#[serde_as(as = "Base64<UrlSafe, Unpadded>")] Vec<u8>),
}

#[derive(Debug, thiserror::Error)]
pub enum AttributeError {
    #[error("unable to convert mdoc cbor value: {0:?}")]
    FromCborConversion(Box<ciborium::Value>),

    #[error("unable to convert integer to cbor: {0}")]
    NumberFromCborIntegerConversion(#[source] TryFromIntError),

    #[error("unable to convert float {0} to cbor")]
    NumberFromFloatConversion(f64),

    #[error("unable to convert claim value: {0:?}")]
    FromClaimValueConversion(Box<ClaimValue>),

    #[error("unable to convert number from claim value: {0}")]
    NumberFromClaimValueConversion(Number),

    #[error("unable to convert map key, expected string: {0:?}")]
    MapKeyConversion(Box<ciborium::Value>),
}

#[derive(Debug, thiserror::Error)]
pub enum AttributesError {
    #[error("attributes without claim: {0:?}")]
    AttributesWithoutClaim(Vec<Vec<String>>),

    #[error("attribute error at: {0}")]
    Attribute(#[from] AttributeError),

    #[error("attribute error at {0}: {1}")]
    AttributeAtPath(String, #[source] AttributeError),

    #[error("some attributes have not been processed by metadata: {0:?}")]
    SomeAttributesNotProcessed(Box<IndexMap<String, Vec<Entry>>>),

    #[error("missing a mandatory attribute: {0:?}")]
    MissingMandatoryAttribute(Vec<VecNonEmpty<ClaimPath>>),
}

impl From<Attribute> for ciborium::Value {
    fn from(value: Attribute) -> Self {
        match value {
            Attribute::Number(number) if let Some(i) = number.as_i64() => ciborium::Value::Integer(i.into()),
            Attribute::Number(number) if let Some(f) = number.as_f64() => ciborium::Value::Float(f),
            Attribute::Number(_) => unimplemented!("number should be either i64 or f64"),
            Attribute::Bool(boolean) => ciborium::Value::Bool(boolean),
            Attribute::Text(text) => ciborium::Value::Text(text),
            Attribute::Null => ciborium::Value::Null,
            Attribute::Array(elements) => ciborium::Value::Array(elements.into_iter().map(Self::from).collect()),
            Attribute::Date(dt) => {
                ciborium::Value::Tag(1004, Box::new(ciborium::Value::Text(dt.format("%Y-%m-%d").to_string())))
            }
            Attribute::Bytes(items) => ciborium::Value::Bytes(items),
            Attribute::Object(map) => ciborium::Value::Map(
                map.into_iter()
                    .map(|(k, v)| (ciborium::Value::Text(k), ciborium::Value::from(v)))
                    .collect(),
            ),
        }
    }
}

impl TryFrom<Attribute> for ClaimValue {
    type Error = ClaimNameError;

    fn try_from(value: Attribute) -> Result<Self, Self::Error> {
        match value {
            Attribute::Null => Ok(ClaimValue::Null),
            Attribute::Number(number) => Ok(ClaimValue::Number(number)),
            Attribute::Bool(boolean) => Ok(ClaimValue::Bool(boolean)),
            Attribute::Text(text) => Ok(ClaimValue::String(text)),
            Attribute::Array(elements) => Ok(ClaimValue::Array(
                elements
                    .into_iter()
                    .map(|attr| Ok(ArrayClaim::Value(attr.try_into()?)))
                    .try_collect()?,
            )),
            Attribute::Object(map) => map_to_claim_value(map),
            Attribute::Date(_) => unimplemented!("Attribute::Date to ClaimValue conversion not supported"),
            Attribute::Bytes(_) => unimplemented!("Attribute::Bytes to ClaimValue conversion not supported"),
        }
    }
}

impl TryFrom<ciborium::Value> for Attribute {
    type Error = AttributeError;

    fn try_from(value: ciborium::Value) -> Result<Self, Self::Error> {
        match value {
            ciborium::Value::Text(text) => Ok(Attribute::Text(text)),
            ciborium::Value::Bool(bool) => Ok(Attribute::Bool(bool)),
            ciborium::Value::Integer(integer) => Ok(Attribute::Number(
                i64::try_from(integer)
                    .map_err(AttributeError::NumberFromCborIntegerConversion)?
                    .into(),
            )),
            ciborium::Value::Float(float) => Ok(Attribute::Number(
                Number::from_f64(float).ok_or(AttributeError::NumberFromFloatConversion(float))?,
            )),
            ciborium::Value::Null => Ok(Attribute::Null),
            ciborium::Value::Array(elements) => Ok(Attribute::Array(
                elements.into_iter().map(Attribute::try_from).try_collect()?,
            )),
            ciborium::Value::Bytes(bytes) => Ok(Attribute::Bytes(bytes)),
            ciborium::Value::Tag(1004, inner) => match *inner {
                ciborium::Value::Text(s) => NaiveDate::parse_from_str(&s, "%Y-%m-%d")
                    .map_err(|_| AttributeError::FromCborConversion(Box::new(ciborium::Value::Text(s.clone()))))
                    .map(Attribute::Date),
                other => Err(AttributeError::FromCborConversion(Box::new(ciborium::Value::Tag(
                    1004,
                    Box::new(other),
                )))),
            },
            ciborium::Value::Map(entries) => {
                let map = entries
                    .into_iter()
                    .map(|(k, v)| {
                        let key = match k {
                            ciborium::Value::Text(s) => Ok(s),
                            other => Err(AttributeError::MapKeyConversion(Box::new(other))),
                        }?;
                        Ok((key, Attribute::try_from(v)?))
                    })
                    .collect::<Result<IndexMap<_, _>, AttributeError>>()?;
                Ok(Attribute::Object(map))
            }
            _ => Err(AttributeError::FromCborConversion(Box::new(value))),
        }
    }
}

impl TryFrom<ClaimValue> for Attribute {
    type Error = AttributeError;

    fn try_from(value: ClaimValue) -> Result<Self, Self::Error> {
        match value {
            ClaimValue::Null => Ok(Attribute::Null),
            ClaimValue::Number(number) => Ok(Attribute::Number(number)),
            ClaimValue::Bool(boolean) => Ok(Attribute::Bool(boolean)),
            ClaimValue::String(text) => Ok(Attribute::Text(text)),
            ClaimValue::Array(elements) => Ok(Attribute::Array(
                elements
                    .into_iter()
                    .filter_map(|value| match value {
                        ArrayClaim::Value(claim_value) => Some(Attribute::try_from(claim_value)),
                        _ => None, // ignore hashes in Arrays
                    })
                    .try_collect()?,
            )),
            ClaimValue::Object(object_claims) => Ok(Attribute::Object(object_claims_to_map(object_claims)?)),
        }
    }
}

impl TryFrom<Attributes> for ClaimValue {
    type Error = ClaimNameError;

    fn try_from(value: Attributes) -> Result<Self, Self::Error> {
        map_to_claim_value(value.0)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, AsRef, From)]
pub struct Attributes(IndexMap<String, Attribute>);

impl TryFrom<ObjectClaims> for Attributes {
    type Error = AttributesError;

    fn try_from(value: ObjectClaims) -> Result<Self, Self::Error> {
        Ok(Attributes(object_claims_to_map(value)?))
    }
}

fn object_claims_to_map(object_claims: ObjectClaims) -> Result<IndexMap<String, Attribute>, AttributeError> {
    object_claims
        .claims
        .into_iter()
        .map(|(k, v)| Ok((k.into_inner(), v.try_into()?)))
        .collect::<Result<_, AttributeError>>()
}

fn map_to_claim_value(attributes: IndexMap<String, Attribute>) -> Result<ClaimValue, ClaimNameError> {
    Ok(ClaimValue::Object(ObjectClaims {
        _sd: None,
        claims: attributes
            .into_iter()
            .map(|(k, v)| Ok((k.parse()?, v.try_into()?)))
            .collect::<Result<_, ClaimNameError>>()?,
    }))
}

#[derive(Debug, Clone, Copy)]
pub enum AttributesTraversalBehaviour {
    AllPaths,
    OnlyLeaves,
}

impl Attributes {
    pub fn into_inner(self) -> IndexMap<String, Attribute> {
        self.0
    }

    /// Returns a flattened view of the attributes. The keys are the attribute paths, and the values are leaf
    /// attributes. These leafs are never object attributes.
    pub fn flattened(&self) -> IndexMap<VecNonEmpty<&str>, &Attribute> {
        /// Recursive depth first traversal helper to flatten all leaf nodes.
        ///
        /// - `prefix` is the path to the current level
        /// - `attrs` are the attributes at the current nesting level
        /// - `result` is a running index map of attributes by path (deepest-first)
        fn traverse_depth_first<'a>(
            prefix: &[&'a str],
            attrs: &'a IndexMap<String, Attribute>,
            result: &mut IndexMap<VecNonEmpty<&'a str>, &'a Attribute>,
        ) {
            attrs.iter().for_each(|(key, attr)| {
                let path = prefix
                    .iter()
                    .copied()
                    .chain(std::iter::once(key.as_str()))
                    .collect_vec();

                match attr {
                    Attribute::Object(nested) => {
                        traverse_depth_first(&path, nested, result);
                    }
                    attribute => {
                        result.insert(VecNonEmpty::try_from(path).unwrap(), attribute);
                    }
                }
            })
        }

        let mut result = IndexMap::with_capacity(self.0.len());
        traverse_depth_first(&[], &self.0, &mut result);
        result
    }

    pub fn validate(&self, type_metadata: &NormalizedTypeMetadata) -> Result<(), AttributesError> {
        let flattened_attributes = self.flattened();
        let claim_key_paths = type_metadata.claim_key_paths().collect_vec();

        let attributes_without_claim = flattened_attributes
            .keys()
            .filter(|path| !claim_key_paths.contains(path))
            .map(|path| path.iter().map(ToString::to_string).collect_vec())
            .collect_vec();

        if !attributes_without_claim.is_empty() {
            return Err(AttributesError::AttributesWithoutClaim(attributes_without_claim));
        }

        let missing_mandatory = type_metadata
            .mandatory_claims()
            .filter(|path| {
                let has_claim = path
                    .iter()
                    .map(ClaimPath::try_key_path)
                    .collect::<Option<Vec<_>>>()
                    .map(|path| {
                        // If a mandatory claim exists entirely of `SelectByKey` values, check that this path is present
                        // in the attributes.
                        let path = VecNonEmpty::try_from(path).expect("source of path is non-empty");
                        flattened_attributes.contains_key(&path)
                    })
                    // Otherwise, we know that this claim is not present, as non-`SelecByKey` paths are not supported.
                    .unwrap_or(false);

                !has_claim
            })
            .cloned()
            .collect_vec();

        if !missing_mandatory.is_empty() {
            return Err(AttributesError::MissingMandatoryAttribute(missing_mandatory));
        }

        // No internal attributes can be in the attributes as they are forbidden as claim in the type metadata
        Ok(())
    }

    /// Convert a map of namespaced entries (`Entry`) to a (nested) map of attributes by key.
    /// The namespace is required to consist of nested group names, joined by a '.' and prefixed
    /// with the attestation_type.
    ///
    /// If the `attributes` input parameter is as follows (denoted here in JSON):
    /// ```json
    /// {
    ///     "com.example.pid": {
    ///         "birthdate": "1963-08-12",
    ///     },
    ///     "com.example.pid.place_of_birth": {
    ///         "locality": "The Hague",
    ///     },
    ///     "com.example.pid.place_of_birth.country": {
    ///         "name": "The Netherlands",
    ///         "area_code": 31
    ///     }
    /// }
    /// ```
    ///
    /// Then the output is as follows (denoted here in JSON):
    /// ```json
    /// {
    ///     "birthdate": "1963-08-12",
    ///     "place_of_birth": {
    ///         "locality": "The Hague",
    ///         "country": {
    ///             "name": "The Netherlands",
    ///             "area_code": 31
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// Note in particular that attributes in the root mdoc namespace (see [`Self::find_mdoc_namespace_root`]) are
    /// mapped to the root level of the output. This root namespace does not necessarily equal the attestation_type
    /// in the metadata: e.g. ISO 18013-5 mDL uses doctype `org.iso.18013.5.1.mDL` but namespace
    /// `org.iso.18013.5.1`.
    pub fn from_mdoc_attributes(
        type_metadata: &NormalizedTypeMetadata,
        mut attributes: IndexMap<NameSpace, Vec<Entry>>,
    ) -> Result<Self, AttributesError> {
        // Get the claim paths consisting only out of claim key paths
        let key_paths = type_metadata.claim_key_paths().collect_vec();

        let mut result = IndexMap::with_capacity(key_paths.len());

        // Only proceed if a root namespace can be found; if not, the loop below is skipped and `attributes` is left
        // untouched, which is reported below as `SomeAttributesNotProcessed`.
        if let Some(namespace_root) = Self::find_mdoc_namespace_root(type_metadata.vct(), &attributes) {
            // The key paths of the claims determines the order of the attributes result
            for key_path in key_paths {
                Self::traverse_attributes_by_claim(&namespace_root, key_path.as_slice(), &mut attributes, &mut result)?;
            }
        }

        if !attributes.is_empty() {
            return Err(AttributesError::SomeAttributesNotProcessed(Box::new(attributes)));
        }

        Ok(Self(result))
    }

    fn is_root_namespace(candidate: &str, attributes: &IndexMap<NameSpace, Vec<Entry>>) -> bool {
        attributes.keys().all(|namespace| {
            namespace == candidate
                || namespace
                    .strip_prefix(candidate)
                    .is_some_and(|rest| rest.starts_with('.'))
        })
    }

    /// Find the root mdoc namespace. Prefer `attestation_type` itself if every namespace present is `attestation_type`
    /// or nested under it (`attestation_type.group...`), which covers the normal case where the mdoc namespace
    /// equals the `attestation_type`. If `attestation_type` doesn't work as a root (e.g. attestation types whose mdoc
    /// namespace was overridden to differ from `attestation_type`, such as ISO 18013-5 mDL), fall back to
    /// whichever namespace present is a common root for every other one.
    fn find_mdoc_namespace_root(
        attestation_type: &str,
        attributes: &IndexMap<NameSpace, Vec<Entry>>,
    ) -> Option<String> {
        if Self::is_root_namespace(attestation_type, attributes) {
            return Some(attestation_type.to_owned());
        }

        attributes
            .keys()
            .filter(|candidate| Self::is_root_namespace(candidate, attributes))
            .exactly_one()
            .ok()
            .cloned()
    }

    fn traverse_attributes_by_claim(
        prefix: &str,
        keys: &[&str],
        attributes: &mut IndexMap<String, Vec<Entry>>,
        result: &mut IndexMap<String, Attribute>,
    ) -> Result<(), AttributesError> {
        if attributes.is_empty() {
            return Ok(());
        }

        match *keys {
            [head] => {
                if let Some(entries) = attributes.get_mut(prefix) {
                    Self::insert_entry(head, entries, result)
                        .map_err(|error| AttributesError::AttributeAtPath(format!("{prefix}.{head}"), error))?;

                    if entries.is_empty() {
                        attributes.swap_remove(prefix);
                    }
                }
            }
            [head, ..] => {
                let prefixed_key = format!("{prefix}.{head}");

                if let Attribute::Object(result) = result
                    .entry(String::from(head))
                    .or_insert_with(|| Attribute::Object(IndexMap::new()))
                {
                    Self::traverse_attributes_by_claim(&prefixed_key, &keys[1..], attributes, result)?;
                }
            }
            [] => {
                panic!("Unexpected empty key path");
            }
        }

        Ok(())
    }

    fn insert_entry(
        key: &str,
        entries: &mut Vec<Entry>,
        group: &mut IndexMap<String, Attribute>,
    ) -> Result<(), AttributeError> {
        if let Some(index) = entries.iter().position(|entry| entry.name == key) {
            let entry = entries.swap_remove(index);
            group.insert(entry.name, entry.value.try_into()?);
        }

        Ok(())
    }

    /// Convert a (nested) map of keyed `Attribute`s into a map of namespaced entries. This is done by
    /// walking down the tree of attributes and using their keys as namespaces. For example, these
    /// nested attributes:
    /// ```json
    /// {
    ///     "attestation_type": "com.example.address",
    ///     "attributes": {
    ///         "city": "The Capital",
    ///         "street": "Main St.",
    ///         "house": {
    ///             "number": 1,
    ///             "letter": "A"
    ///         }
    ///     }
    /// }
    /// ```
    /// Turns into a flattened namespaced map of `Entry` with the following structure:
    /// ```json
    /// {
    ///     "com.example.address": {
    ///         "city": "The Capital",
    ///         "street": "Main St."
    ///     },
    ///     "com.example.address.house": {
    ///         "number": 1,
    ///         "letter": "A"
    ///     }
    /// }
    /// ```
    pub fn to_mdoc_attributes(self, attestation_type: &str) -> IndexMap<NameSpace, Vec<Entry>> {
        let mut result = IndexMap::new();
        for (path, attribute) in self.flattened() {
            let (path, name) = path.into_inner_last();
            let mut prefix = std::iter::once(attestation_type).chain(path.iter().copied());
            result.entry(prefix.join(".")).or_insert_with(Vec::new).push(Entry {
                name: name.to_string(),
                value: attribute.clone().into(),
            })
        }
        result
    }

    pub fn claim_paths(&self, behaviour: AttributesTraversalBehaviour) -> Vec<VecNonEmpty<ClaimPath>> {
        /// Recursive depth first traversal helper to collect all claim paths from nested attributes.
        ///
        /// Depth first is necessary because the SD-JWT conceal functionality for leafs (any attribute that is not an
        /// object) doesn't work properly if the parent node is already concealed (and therefore not present
        /// anymore in the resulting claims).
        ///
        /// - `prefix` is the path to the current level
        /// - `attrs` are the attributes at the current nesting level
        /// - `result` is a running collection of all full paths (deepest-first)
        fn traverse_depth_first(
            prefix: &[ClaimPath],
            attrs: &IndexMap<String, Attribute>,
            result: &mut Vec<VecNonEmpty<ClaimPath>>,
            behaviour: AttributesTraversalBehaviour,
        ) {
            for (key, attr) in attrs {
                let path = prefix
                    .iter()
                    .cloned()
                    .chain(std::iter::once(ClaimPath::SelectByKey(key.clone())))
                    .collect_vec();

                // If it's a nested attribute, recurse deeper first
                if let Attribute::Object(nested) = attr {
                    traverse_depth_first(&path, nested, result, behaviour);
                }

                match (attr, behaviour) {
                    (Attribute::Object(_), AttributesTraversalBehaviour::OnlyLeaves) => {}
                    (Attribute::Object(_), AttributesTraversalBehaviour::AllPaths) | (_, _) => {
                        // Push current path after children have been processed (post-order)
                        result.push(VecNonEmpty::try_from(path).unwrap());
                    }
                }
            }
        }

        let mut result = Vec::with_capacity(self.0.len());
        traverse_depth_first(&[], self.as_ref(), &mut result, behaviour);
        result
    }

    /// Retrieve the non-object attribute value at the specified location, if it exists.
    pub fn get(&self, claim_paths: &VecNonEmpty<ClaimPath>) -> Result<Option<&Attribute>, AttributesHandlingError> {
        let Some(mut attr) = self.as_ref().get(
            claim_paths
                .first()
                .try_key_path()
                .ok_or(AttributesHandlingError::InvalidClaimPath)?,
        ) else {
            return Ok(None);
        };

        // We already handled the first element above, so skip it here
        for claim_path in &claim_paths[1..] {
            let claim_path = claim_path
                .try_key_path()
                .ok_or(AttributesHandlingError::InvalidClaimPath)?;

            attr = match attr {
                Attribute::Object(map) => match map.get(claim_path) {
                    Some(map) => map,
                    None => return Ok(None),
                },
                _ => return Ok(None),
            };
        }

        let attr = match attr {
            Attribute::Object(_) => return Ok(None),
            value => value,
        };

        Ok(Some(attr))
    }

    /// Check if the a value exists at all of the provided claim paths.
    /// Note that an invalid path will result in a mismatch.
    pub fn has_claim_paths<'a, 'b>(
        &'a self,
        claim_paths: impl IntoIterator<Item = &'b VecNonEmpty<ClaimPath>>,
    ) -> bool {
        claim_paths
            .into_iter()
            .all(|claim_path| self.get(claim_path).unwrap_or(None).is_some())
    }

    /// Check if the a value exists at the provided claim path.
    /// Note that an invalid path will result in a mismatch.
    pub fn has_claim_path(&self, claim_path: &VecNonEmpty<ClaimPath>) -> bool {
        self.get(claim_path).unwrap_or(None).is_some()
    }

    /// Insert the specified attribute at the specified location.
    pub fn insert(
        &mut self,
        claim_paths: &VecNonEmpty<ClaimPath>,
        attribute: Attribute,
    ) -> Result<(), AttributesHandlingError> {
        let Self(root_map) = self;

        // Traverse the tree using all but the last claim path. This should always result
        // in a nested attribute, which we create if the attribute is entirely absent.
        let leaf_map = claim_paths
            .iter()
            // This is guaranteed to be at least 0 because `claims_paths` is not empty.
            .take(claim_paths.len().get() - 1)
            .try_fold(root_map, |map, claim_path| {
                let claim_path = claim_path
                    .try_key_path()
                    .ok_or(AttributesHandlingError::InvalidClaimPath)?;

                // Find the attribute at the path or create a new nested attribute.
                let attribute = map
                    .entry(claim_path.to_string())
                    .or_insert_with(|| Attribute::Object(IndexMap::new()));

                // If the attribute is a leaf the claim path is longer than expected and thus invalid.
                let child_map = match attribute {
                    Attribute::Object(map) => map,
                    _ => return Err(AttributesHandlingError::InvalidClaimPath),
                };

                Ok(child_map)
            })?;

        // If the last claim path is not already present, insert the attribute.
        let last_claim_path = claim_paths
            .last()
            .try_key_path()
            .ok_or(AttributesHandlingError::InvalidClaimPath)?;

        match leaf_map.get(last_claim_path) {
            Some(Attribute::Object(_)) => Err(AttributesHandlingError::InvalidClaimPath),
            Some(_) => Err(AttributesHandlingError::ClaimAlreadyExists),
            None => {
                leaf_map.insert(last_claim_path.to_string(), attribute);

                Ok(())
            }
        }
    }

    /// Prune attributes from a tree by only keeping those specified by a list of claim paths.
    pub fn prune<'a>(&mut self, keep_claim_paths: impl IntoIterator<Item = &'a VecNonEmpty<ClaimPath>>) {
        fn nested_prune(
            path_prefix: &[&str],
            attributes: &mut IndexMap<String, Attribute>,
            keep_claim_paths: &HashSet<Vec<&str>>,
        ) -> bool {
            attributes.retain(|path_element, attribute| {
                let path = path_prefix
                    .iter()
                    .copied()
                    .chain(std::iter::once(path_element.as_str()))
                    .collect_vec();

                match attribute {
                    Attribute::Object(attributes) => nested_prune(&path, attributes, keep_claim_paths),
                    _ => keep_claim_paths.contains(&path),
                }
            });

            !attributes.is_empty()
        }

        // Only key claim paths are supported for now, so if any path contains
        // an element that is not a key, filter out the entire path.
        let keep_claim_paths = keep_claim_paths
            .into_iter()
            .flat_map(|path| path.iter().map(ClaimPath::try_key_path).collect::<Option<Vec<_>>>())
            .collect::<HashSet<_>>();

        let Self(attributes) = self;
        nested_prune(&[], attributes, &keep_claim_paths);
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum AttributesHandlingError {
    #[error("invalid claim path")]
    InvalidClaimPath,
    #[error("cannot insert claim: already exists")]
    ClaimAlreadyExists,
}

#[cfg(any(test, feature = "example_credential_payloads"))]
mod examples {
    use attestation_types::claim_path::ClaimPath;
    use indexmap::IndexMap;
    use itertools::Itertools;

    use super::Attribute;
    use super::Attributes;

    impl Attributes {
        pub fn example<'a>(
            attributes: impl IntoIterator<Item = (impl IntoIterator<Item = &'a str>, Attribute)>,
        ) -> Self {
            attributes
                .into_iter()
                .fold(Self::from(IndexMap::new()), |mut attributes, (path, value)| {
                    let path = path
                        .into_iter()
                        .map(|element| ClaimPath::SelectByKey(element.to_string()))
                        .collect_vec()
                        .try_into()
                        .expect("path should consist of at least one path element");

                    attributes.insert(&path, value).expect("paths are inconsistent");

                    attributes
                })
        }
    }
}

#[cfg(feature = "mock")]
mod mock {
    use attestation_types::pid_constants::PID_ADDRESS_GROUP;
    use attestation_types::pid_constants::PID_AGE_OVER_18;
    use attestation_types::pid_constants::PID_BIRTH_DATE;
    use attestation_types::pid_constants::PID_BSN;
    use attestation_types::pid_constants::PID_FAMILY_NAME;
    use attestation_types::pid_constants::PID_GIVEN_NAME;
    use attestation_types::pid_constants::PID_RECOVERY_CODE;
    use attestation_types::pid_constants::PID_RESIDENT_CITY;
    use attestation_types::pid_constants::PID_RESIDENT_COUNTRY;
    use attestation_types::pid_constants::PID_RESIDENT_HOUSE_NUMBER;
    use attestation_types::pid_constants::PID_RESIDENT_POSTAL_CODE;
    use attestation_types::pid_constants::PID_RESIDENT_STREET;

    use super::Attribute;
    use super::Attributes;

    impl Attributes {
        pub fn nl_pid_example() -> Self {
            Self::example([
                ([PID_GIVEN_NAME], Attribute::Text("Willeke Liselotte".to_string())),
                ([PID_FAMILY_NAME], Attribute::Text("De Bruijn".to_string())),
                ([PID_BIRTH_DATE], Attribute::Text("1997-05-10".to_string())),
                ([PID_AGE_OVER_18], Attribute::Bool(true)),
                ([PID_BSN], Attribute::Text("999991772".to_string())),
                ([PID_RECOVERY_CODE], Attribute::Text("123".to_string())),
            ])
        }

        pub fn nl_pid_address_example() -> Self {
            Self::example([
                (
                    [PID_ADDRESS_GROUP, PID_RESIDENT_STREET],
                    Attribute::Text("Turfmarkt".to_string()),
                ),
                (
                    [PID_ADDRESS_GROUP, PID_RESIDENT_HOUSE_NUMBER],
                    Attribute::Text("147".to_string()),
                ),
                (
                    [PID_ADDRESS_GROUP, PID_RESIDENT_POSTAL_CODE],
                    Attribute::Text("2511 DP".to_string()),
                ),
                (
                    [PID_ADDRESS_GROUP, PID_RESIDENT_CITY],
                    Attribute::Text("Den Haag".to_string()),
                ),
                (
                    [PID_ADDRESS_GROUP, PID_RESIDENT_COUNTRY],
                    Attribute::Text("Nederland".to_string()),
                ),
            ])
        }
    }
}

#[cfg(test)]
pub mod test {
    use std::assert_matches;

    use attestation_types::claim_path::ClaimPath;
    use indexmap::IndexMap;
    use mdoc::Entry;
    use mdoc::NameSpace;
    use rstest::rstest;
    use sd_jwt_vc_metadata::NormalizedTypeMetadata;
    use serde_json::json;
    use serde_valid::json::ToJsonString;
    use utils::vec_at_least::VecNonEmpty;
    use utils::vec_nonempty;

    use super::Attribute;
    use super::Attributes;
    use super::AttributesError;
    use super::AttributesHandlingError;
    use super::AttributesTraversalBehaviour;

    pub fn complex_attributes() -> IndexMap<String, Attribute> {
        IndexMap::from([
            (String::from("birth_date"), Attribute::Text(String::from("1963-08-12"))),
            (
                String::from("place_of_birth"),
                Attribute::Object(IndexMap::from([
                    (String::from("locality"), Attribute::Text(String::from("The Hague"))),
                    (
                        String::from("country"),
                        Attribute::Object(IndexMap::from([
                            (String::from("name"), Attribute::Text(String::from("The Netherlands"))),
                            (String::from("area_code"), Attribute::Number(33.into())),
                        ])),
                    ),
                ])),
            ),
            (
                String::from("financial"),
                Attribute::Object(IndexMap::from([
                    (String::from("has_debt"), Attribute::Bool(true)),
                    (String::from("has_job"), Attribute::Bool(false)),
                    (String::from("debt_amount"), Attribute::Number((-10_000).into())),
                ])),
            ),
        ])
    }

    #[test]
    fn test_traverse_groups() {
        let metadata_json = json!({
            "vct": "com.example.pid",
            "display": [{"locale": "en", "name": "example"}],
            "claims": [{
                "path": ["birthdate"],
                "display": [{"locale": "en", "label": "birthdate"}],
            }, {
                "path": ["place_of_birth", "locality"],
                "display": [{"locale": "en", "label": "birth city"}],
            }, {
                "path": ["place_of_birth", "country", "name"],
                "display": [{"locale": "en", "label": "birth country"}],
            }, {
                "path": ["place_of_birth", "country", "area_code"],
                "display": [{"locale": "en", "label": "birth area code"}],
            }, {
                "path": ["a", "b", "c", "d", "e"],
                "display": [{"locale": "en", "label": "a b c d e"}],
            }, {
                "path": ["a", "b", "c1"],
                "display": [{"locale": "en", "label": "a b c1"}],
            }]
        });
        let type_metadata = NormalizedTypeMetadata::from_single_example(serde_json::from_value(metadata_json).unwrap());

        let mdoc_attributes = IndexMap::from([
            (
                String::from("com.example.pid"),
                vec![Entry {
                    name: String::from("birthdate"),
                    value: ciborium::Value::Tag(1004, Box::new(ciborium::Value::Text("1963-08-12".to_string()))),
                }],
            ),
            (
                String::from("com.example.pid.place_of_birth"),
                vec![Entry {
                    name: String::from("locality"),
                    value: ciborium::Value::Text(String::from("The Hague")),
                }],
            ),
            (
                String::from("com.example.pid.place_of_birth.country"),
                vec![
                    Entry {
                        name: String::from("name"),
                        value: ciborium::Value::Text(String::from("The Netherlands")),
                    },
                    Entry {
                        name: String::from("area_code"),
                        value: ciborium::Value::Integer(31.into()),
                    },
                ],
            ),
            (
                String::from("com.example.pid.a.b.c.d"),
                vec![Entry {
                    name: String::from("e"),
                    value: ciborium::Value::Text(String::from("abcd")),
                }],
            ),
            (
                String::from("com.example.pid.a.b"),
                vec![Entry {
                    name: String::from("c1"),
                    value: ciborium::Value::Text(String::from("abc")),
                }],
            ),
        ]);
        let result = Attributes::from_mdoc_attributes(&type_metadata, mdoc_attributes).unwrap();

        let expected_json = json!({
            "birthdate": {
                "type": "date",
                "value": "1963-08-12"
            },
            "place_of_birth": {
                "type": "object",
                "value": {
                    "locality": {
                        "type": "text",
                        "value": "The Hague"
                    },
                    "country": {
                        "type": "object",
                        "value": {
                            "name": {
                                "type": "text",
                                "value": "The Netherlands"
                            },
                            "area_code": {
                                "type": "number",
                                "value": 31
                            }
                        }
                    }
                }
            },
            "a": {
                "type": "object",
                "value": {
                    "b": {
                        "type": "object",
                        "value": {
                            "c": {
                                "type": "object",
                                "value": {
                                    "d": {
                                        "type": "object",
                                        "value": {
                                            "e": {
                                                "type": "text",
                                                "value": "abcd"
                                            }
                                        }
                                    }
                                }
                            },
                            "c1": {
                                "type": "text",
                                "value": "abc"
                            }
                        }
                    }
                }
            }
        });
        assert_eq!(
            serde_json::to_value(result).unwrap().to_json_string_pretty().unwrap(),
            expected_json.to_json_string_pretty().unwrap(),
        );
    }

    #[test]
    fn test_traverse_groups_for_dot_in_attribute_name() {
        let metadata_json = json!({
            "vct": "com.example.pid",
            "display": [{"locale": "en", "name": "example"}],
            "claims": [
                {
                    "path": ["root_entry"],
                    "display": [{"locale": "en", "label": "root entry"}],
                },
                {
                    "path": ["nest.ed", "birth.date"],
                    "display": [{"locale": "en", "label": "nested birthday"}],
                }
            ]
        });
        let type_metadata = NormalizedTypeMetadata::from_single_example(serde_json::from_value(metadata_json).unwrap());

        let mdoc_attributes = IndexMap::from([
            (
                "com.example.pid".to_owned(),
                vec![Entry {
                    name: "root_entry".to_owned(),
                    value: ciborium::Value::Text("x".to_owned()),
                }],
            ),
            (
                "com.example.pid.nest.ed".to_owned(),
                vec![Entry {
                    name: "birth.date".to_owned(),
                    value: ciborium::Value::Text("1963-08-12".to_owned()),
                }],
            ),
        ]);

        let result = Attributes::from_mdoc_attributes(&type_metadata, mdoc_attributes).unwrap();

        let expected_json = json!({
            "root_entry": {
                "type": "text",
                "value": "x"
            },
            "nest.ed": {
                "type": "object",
                "value": {
                    "birth.date": {
                        "type": "text",
                        "value": "1963-08-12"
                    }
                }
            }
        });
        assert_eq!(
            serde_json::to_value(result).unwrap().to_json_string_pretty().unwrap(),
            expected_json.to_json_string_pretty().unwrap(),
        );
    }

    #[test]
    fn test_traverse_groups_with_extra_entry_not_in_claim() {
        let metadata_json = json!({
            "vct": "com.example.pid",
            "display": [{"locale": "en", "name": "example"}],
            "claims": [
                {
                    "path": ["root_entry"],
                    "display": [{"locale": "en", "label": "root entry"}],
                },
                {
                    "path": ["a", "a1"],
                    "display": [{"locale": "en", "label": "a a1"}],
                },
                {
                    "path": ["a", "a2"],
                    "display": [{"locale": "en", "label": "a a1"}],
                }
            ]
        });
        let type_metadata = NormalizedTypeMetadata::from_single_example(serde_json::from_value(metadata_json).unwrap());

        let mdoc_attributes = IndexMap::from([
            (
                "com.example.pid".to_owned(),
                vec![Entry {
                    name: "root_entry".to_owned(),
                    value: ciborium::Value::Text("x".to_owned()),
                }],
            ),
            (
                String::from("com.example.pid.a"),
                vec![
                    Entry {
                        name: "a1".to_owned(),
                        value: ciborium::Value::Text("1".to_owned()),
                    },
                    Entry {
                        name: "a2".to_owned(),
                        value: ciborium::Value::Text("2".to_owned()),
                    },
                    Entry {
                        name: "a3".to_owned(),
                        value: ciborium::Value::Text("3".to_owned()),
                    },
                ],
            ),
        ]);

        let result = Attributes::from_mdoc_attributes(&type_metadata, mdoc_attributes);
        assert_matches!(result, Err(AttributesError::SomeAttributesNotProcessed(attrs))
        if *attrs == IndexMap::from([(
            String::from("com.example.pid.a"),
            vec![Entry { name: String::from("a3"), value: ciborium::Value::Text(String::from("3")) }]
        )]));
    }

    #[test]
    fn test_traverse_groups_claim_ordering() {
        let metadata_json = json!({
            "vct": "com.example.pid",
            "display": [{"locale": "en", "name": "example"}],
            "claims": [
                {
                    "path": ["root_entry"],
                    "display": [{"locale": "en", "label": "root entry"}],
                },
                {
                    "path": ["b", "b1"],
                    "display": [{"locale": "en", "label": "b b1"}],
                },
                {
                    "path": ["b", "b3"],
                    "display": [{"locale": "en", "label": "b b3"}],
                },
                {
                    "path": ["b", "b2"],
                    "display": [{"locale": "en", "label": "b b2"}],
                }
            ]
        });
        let type_metadata = NormalizedTypeMetadata::from_single_example(serde_json::from_value(metadata_json).unwrap());

        let mdoc_attributes = IndexMap::from([
            (
                "com.example.pid".to_owned(),
                vec![Entry {
                    name: "root_entry".to_owned(),
                    value: ciborium::Value::Text("x".to_owned()),
                }],
            ),
            (
                String::from("com.example.pid.b"),
                vec![
                    Entry {
                        name: "b1".to_owned(),
                        value: ciborium::Value::Text("1".to_owned()),
                    },
                    Entry {
                        name: "b2".to_owned(),
                        value: ciborium::Value::Text("2".to_owned()),
                    },
                    Entry {
                        name: "b3".to_owned(),
                        value: ciborium::Value::Text("3".to_owned()),
                    },
                ],
            ),
        ]);

        let result = Attributes::from_mdoc_attributes(&type_metadata, mdoc_attributes).unwrap();
        let expected_json = json!({
            "root_entry": {
                "type": "text",
                "value": "x"
            },
            "b": {
                "type": "object",
                "value": {
                    "b1": {
                        "type": "text",
                        "value": "1"
                    },
                    "b3": {
                        "type": "text",
                        "value": "3"
                    },
                    "b2": {
                        "type": "text",
                        "value": "2"
                    }
                }
            }
        });
        assert_eq!(
            serde_json::to_value(result).unwrap().to_json_string_pretty().unwrap(),
            expected_json.to_json_string_pretty().unwrap(),
        );
    }

    fn setup_issuable_attributes() -> Attributes {
        IndexMap::from_iter(vec![
            ("city".to_string(), Attribute::Text("The Capital".to_string())),
            ("postal_code".to_string(), Attribute::Null),
            ("street".to_string(), Attribute::Text("Main St.".to_string())),
            (
                "house".to_string(),
                Attribute::Object(IndexMap::from_iter(vec![
                    ("number".to_string(), Attribute::Number(1.into())),
                    ("letter".to_string(), Attribute::Text("A".to_string())),
                ])),
            ),
        ])
        .into()
    }

    #[test]
    fn test_serialize_attributes() {
        let attributes = setup_issuable_attributes();
        assert_eq!(
            serde_json::to_value(attributes).unwrap(),
            json!({
                "city": {
                    "type": "text",
                    "value": "The Capital"
                },
                "postal_code": {
                    "type": "null"
                },
                "street": {
                    "type": "text",
                    "value": "Main St."
                },
                "house": {
                    "type": "object",
                    "value": {
                        "number": {
                            "type": "number",
                            "value": 1
                        },
                        "letter": {
                            "type": "text",
                            "value": "A"
                        }
                    }
                }
            })
        );
    }

    #[rstest]
    #[case(
        vec_nonempty![ClaimPath::SelectByKey("house".to_string()), ClaimPath::SelectByKey("number".to_string())],
        Ok(Some(&Attribute::Number(1.into()))))
    ]
    #[case(
        vec_nonempty![ClaimPath::SelectByKey("house".to_string()), ClaimPath::SelectByKey("foobar".to_string())],
        Ok(None))
    ]
    #[case(
        vec_nonempty![ClaimPath::SelectByKey("foobar".to_string())],
        Ok(None))
    ]
    #[case(
        vec_nonempty![ClaimPath::SelectByKey("city".to_string()), ClaimPath::SelectByKey("number".to_string())],
        Ok(None))
    ]
    #[case(vec_nonempty![ClaimPath::SelectByKey("house".to_string())], Ok(None))]
    #[case(vec_nonempty![ClaimPath::SelectByIndex(1)], Err(AttributesHandlingError::InvalidClaimPath))]
    #[case(vec_nonempty![ClaimPath::SelectAll], Err(AttributesHandlingError::InvalidClaimPath))]
    fn test_attributes_get_and_has_claim_paths(
        #[case] claim_paths: VecNonEmpty<ClaimPath>,
        #[case] expected: Result<Option<&Attribute>, AttributesHandlingError>,
    ) {
        let attributes = setup_issuable_attributes();

        assert_eq!(attributes.get(&claim_paths), expected);

        let has_claim_path = attributes.has_claim_paths(&[claim_paths]);

        match expected {
            Ok(Some(_)) => assert!(has_claim_path),
            _ => assert!(!has_claim_path),
        }
    }

    #[rstest]
    #[case(
        vec![ClaimPath::SelectByKey("foo".to_string())],
        Ok(json!({
            "outer": {
                "type": "object",
                "value": {
                    "inner": {
                        "type": "text",
                        "value": "value"
                    }
                }
            },
            "foo": {
                "type": "bool",
                "value": true
            }
        }))
    )]
    #[case(
        vec![ClaimPath::SelectByKey("outer".to_string()), ClaimPath::SelectByKey("foo".to_string())],
        Ok(json!({
            "outer": {
                "type": "object",
                "value": {
                    "foo": {
                        "type": "bool",
                        "value": true
                    },
                    "inner": {
                        "type": "text",
                        "value": "value"
                    }
                }
            }
        }))
    )]
    #[case(
        vec![ClaimPath::SelectByKey("outer".to_string())],
        Err(AttributesHandlingError::InvalidClaimPath)
    )]
    #[case(
        vec![ClaimPath::SelectByKey("outer".to_string()), ClaimPath::SelectByKey("inner".to_string())],
        Err(AttributesHandlingError::ClaimAlreadyExists)
    )]
    #[case(
        vec![ClaimPath::SelectByIndex(0)],
        Err(AttributesHandlingError::InvalidClaimPath)
    )]
    #[case(
        vec![ClaimPath::SelectAll],
        Err(AttributesHandlingError::InvalidClaimPath)
    )]
    fn test_attributes_insert(
        #[case] claim_paths: Vec<ClaimPath>,
        #[case] expected: Result<serde_json::Value, AttributesHandlingError>,
    ) {
        let mut attributes: Attributes = IndexMap::from_iter([(
            "outer".to_string(),
            Attribute::Object(IndexMap::from_iter([(
                "inner".to_string(),
                Attribute::Text("value".to_string()),
            )])),
        )])
        .into();

        let result = attributes
            .insert(&claim_paths.try_into().unwrap(), Attribute::Bool(true))
            .map(|_| serde_json::to_value(attributes).unwrap());

        assert_eq!(result, expected);
    }

    fn readable_mdoc_attributes(
        attributes: IndexMap<NameSpace, Vec<Entry>>,
    ) -> IndexMap<String, IndexMap<String, ciborium::Value>> {
        attributes
            .into_iter()
            .map(|(namespace, entries)| {
                (
                    namespace,
                    entries.into_iter().map(|entry| (entry.name, entry.value)).collect(),
                )
            })
            .collect()
    }

    #[rstest]
    #[case(&[], IndexMap::new())]
    #[case(
        &[vec_nonempty![ClaimPath::SelectByKey("name".to_string())]],
        IndexMap::from([("name".to_string(), Attribute::Text("Wallet".to_string()))]),
    )]
    #[case(
        &[
            vec_nonempty![ClaimPath::SelectByKey("nothing".to_string())],
            vec_nonempty![ClaimPath::SelectByKey("address".to_string()), ClaimPath::SelectByKey("street".to_string())],
            vec_nonempty![ClaimPath::SelectAll],
            vec_nonempty![ClaimPath::SelectByKey("country".to_string()), ClaimPath::SelectByKey("iso".to_string())],
            vec_nonempty![ClaimPath::SelectByIndex(0)],
            vec_nonempty![ClaimPath::SelectByKey("address".to_string()), ClaimPath::SelectByKey("state".to_string())],
        ],
        IndexMap::from([
            (
                "country".to_string(),
                Attribute::Object(IndexMap::from([("iso".to_string(), Attribute::Text("NL".to_string()))])),
            ),
            (
                "address".to_string(),
                Attribute::Object(IndexMap::from([("street".to_string(), Attribute::Text("Gracht".to_string()))])),
            ),
        ]),
    )]
    fn test_attributes_prune(
        #[case] keep_claim_paths: &[VecNonEmpty<ClaimPath>],
        #[case] expected_attributes: IndexMap<String, Attribute>,
    ) {
        let mut attributes = example_attributes();
        attributes.prune(keep_claim_paths);

        assert_eq!(*attributes.as_ref(), expected_attributes);
    }

    #[test]
    fn test_attributes_to_mdoc_attributes() {
        let attributes = setup_issuable_attributes().to_mdoc_attributes("com.example.address");

        assert_eq!(
            serde_json::to_value(readable_mdoc_attributes(attributes)).unwrap(),
            json!({
                "com.example.address": {
                    "city": "The Capital",
                    "postal_code": null,
                    "street": "Main St.",
                },
                "com.example.address.house": {
                    "number": 1,
                    "letter": "A",
                },
            })
        );
    }

    #[test]
    fn test_attributes_to_mdoc_attributes_empty_root() {
        let attestation_type = "com.example.address";
        let nested_attributes: Attributes = IndexMap::from_iter(vec![(
            "house".to_string(),
            Attribute::Object(IndexMap::from_iter(vec![(
                "number".to_string(),
                Attribute::Number(1.into()),
            )])),
        )])
        .into();

        let attributes = nested_attributes.to_mdoc_attributes(attestation_type);

        assert_eq!(
            serde_json::to_value(readable_mdoc_attributes(attributes)).unwrap(),
            json!({
                "com.example.address.house": {
                    "number": 1
                }
            })
        );
    }

    fn example_attributes() -> Attributes {
        IndexMap::from([
            ("name".to_string(), Attribute::Text("Wallet".to_string())),
            (
                "address".to_string(),
                Attribute::Object(IndexMap::from([
                    ("street".to_string(), Attribute::Text("Gracht".to_string())),
                    ("number".to_string(), Attribute::Number(123.into())),
                ])),
            ),
            (
                "country".to_string(),
                Attribute::Object(IndexMap::from([
                    ("iso".to_string(), Attribute::Text("NL".to_string())),
                    ("area_code".to_string(), Attribute::Number(31.into())),
                ])),
            ),
            ("adult".to_string(), Attribute::Bool(true)),
        ])
        .into()
    }

    #[test]
    fn test_attributes_flattened() {
        assert_eq!(
            example_attributes().flattened(),
            IndexMap::from([
                (vec_nonempty!["name"], &Attribute::Text("Wallet".to_string())),
                (
                    vec_nonempty!["address", "street"],
                    &Attribute::Text("Gracht".to_string())
                ),
                (vec_nonempty!["address", "number"], &Attribute::Number(123.into())),
                (vec_nonempty!["country", "iso"], &Attribute::Text("NL".to_string())),
                (vec_nonempty!["country", "area_code"], &Attribute::Number(31.into())),
                (vec_nonempty!["adult"], &Attribute::Bool(true)),
            ]),
        );
    }

    #[test]
    fn test_validate_ok() {
        let metadata_json = json!({
            "vct": "com.example.pid",
            "display": [{"locale": "en", "name": "example"}],
            "claims": [
                {
                    "path": ["name"],
                    "display": [{"locale": "en", "label": "name"}],
                },
                {
                    "path": ["birth_date"],
                    "display": [{"locale": "en", "label": "birth date"}],
                },
                {
                    "path": ["address", "street"],
                    "display": [{"locale": "en", "label": "address street"}],
                },
                {
                    "path": ["address", "number"],
                    "display": [{"locale": "en", "label": "address number"}],
                },
                {
                    "path": ["country", "iso"],
                    "display": [{"locale": "en", "label": "country iso"}],
                },
                {
                    "path": ["country", "area_code"],
                    "display": [{"locale": "en", "label": "country area code"}],
                },
                {
                    "path": ["adult"],
                    "display": [{"locale": "en", "label": "adult"}],
                },
            ]
        });
        let type_metadata = NormalizedTypeMetadata::from_single_example(serde_json::from_value(metadata_json).unwrap());

        let result = example_attributes().validate(&type_metadata);
        assert_matches!(result, Ok(_));
    }

    #[test]
    fn test_validate_attributes_without_claim() {
        let metadata_json = json!({
            "vct": "com.example.pid",
            "display": [{"locale": "en", "name": "example"}],
            "claims": [
                {
                    "path": ["name"],
                    "display": [{"locale": "en", "label": "name"}],
                },
                {
                    "path": ["birth_date"],
                    "display": [{"locale": "en", "label": "birth date"}],
                },
                {
                    "path": ["address", "street"],
                    "display": [{"locale": "en", "label": "address street"}],
                },
                {
                    "path": ["address", "number"],
                    "display": [{"locale": "en", "label": "address number"}],
                },
                {
                    "path": ["country", "iso"],
                    "display": [{"locale": "en", "label": "country iso"}],
                },
                {
                    "path": ["adult"],
                    "display": [{"locale": "en", "label": "adult"}],
                },
                {
                    "path": ["country", 0, "area_code"],
                    "display": [{"locale": "en", "label": "first country area code"}],
                },
            ]
        });
        let type_metadata = NormalizedTypeMetadata::from_single_example(serde_json::from_value(metadata_json).unwrap());

        let result = example_attributes().validate(&type_metadata);
        assert_matches!(result, Err(AttributesError::AttributesWithoutClaim(message)) if message == vec![vec!["country", "area_code"]]);
    }

    #[test]
    fn test_validate_missing_mandatory_attribute() {
        let metadata_json = json!({
            "vct": "com.example.pid",
            "display": [{"locale": "en", "name": "example"}],
            "claims": [
                {
                    "path": ["name"],
                    "display": [{"locale": "en", "label": "name"}],
                },
                {
                    "path": ["birth_date"],
                    "display": [{"locale": "en", "label": "birth date"}],
                    "mandatory": true,
                },
                {
                    "path": ["birth", "city"],
                    "display": [{"locale": "en", "label": "birth city"}],
                    "mandatory": true,
                },
                {
                    "path": ["address", "street"],
                    "display": [{"locale": "en", "label": "address street"}],
                },
                {
                    "path": ["address", "number"],
                    "display": [{"locale": "en", "label": "address number"}],
                },
                {
                    "path": ["country", "iso"],
                    "display": [{"locale": "en", "label": "country iso"}],
                },
                {
                    "path": ["country", "area_code"],
                    "display": [{"locale": "en", "label": "country iso"}],
                },
                {
                    "path": ["adult"],
                    "display": [{"locale": "en", "label": "adult"}],
                },
                {
                    "path": ["nationality", null, "passport_number"],
                    "display": [{"locale": "en", "label": "passport number"}],
                    "mandatory": true,
                },
            ]
        });
        let type_metadata = NormalizedTypeMetadata::from_single_example(serde_json::from_value(metadata_json).unwrap());

        let result = example_attributes().validate(&type_metadata);
        let expected_paths = vec![
            vec_nonempty![ClaimPath::SelectByKey("birth_date".to_string())],
            vec_nonempty![
                ClaimPath::SelectByKey("birth".to_string()),
                ClaimPath::SelectByKey("city".to_string())
            ],
            vec_nonempty![
                ClaimPath::SelectByKey("nationality".to_string()),
                ClaimPath::SelectAll,
                ClaimPath::SelectByKey("passport_number".to_string())
            ],
        ];
        assert_matches!(
            result,
            Err(AttributesError::MissingMandatoryAttribute(paths)) if paths == expected_paths
        );
    }

    mod test_claim_paths_from_attributes {
        use attestation_types::claim_path::ClaimPath;

        use super::*;

        #[test]
        fn single_attribute_should_return_correct_claimpaths() {
            let result: Attributes =
                IndexMap::from([(String::from("a"), Attribute::Text(String::from("1234")))]).into();

            let expected = vec![vec_nonempty![ClaimPath::SelectByKey(String::from("a"))]];

            assert_eq!(result.claim_paths(AttributesTraversalBehaviour::AllPaths), expected);
            assert_eq!(result.claim_paths(AttributesTraversalBehaviour::OnlyLeaves), expected);
        }

        #[test]
        fn nested_attribute_should_return_correct_claimpaths() {
            let result: Attributes = IndexMap::from([
                (String::from("b"), Attribute::Text(String::from("1234"))),
                (
                    String::from("a"),
                    Attribute::Object(IndexMap::from([(
                        String::from("a1"),
                        Attribute::Object(IndexMap::from([
                            (String::from("a2"), Attribute::Text(String::from("1234"))),
                            (String::from("a3"), Attribute::Text(String::from("1234"))),
                        ])),
                    )])),
                ),
            ])
            .into();

            let expected_all = vec![
                vec_nonempty![ClaimPath::SelectByKey(String::from("b"))],
                vec_nonempty![
                    ClaimPath::SelectByKey(String::from("a")),
                    ClaimPath::SelectByKey(String::from("a1")),
                    ClaimPath::SelectByKey(String::from("a2")),
                ],
                vec_nonempty![
                    ClaimPath::SelectByKey(String::from("a")),
                    ClaimPath::SelectByKey(String::from("a1")),
                    ClaimPath::SelectByKey(String::from("a3")),
                ],
                vec_nonempty![
                    ClaimPath::SelectByKey(String::from("a")),
                    ClaimPath::SelectByKey(String::from("a1")),
                ],
                vec_nonempty![ClaimPath::SelectByKey(String::from("a"))],
            ];

            assert_eq!(result.claim_paths(AttributesTraversalBehaviour::AllPaths), expected_all);

            let expected_leaves = vec![
                vec_nonempty![ClaimPath::SelectByKey(String::from("b"))],
                vec_nonempty![
                    ClaimPath::SelectByKey(String::from("a")),
                    ClaimPath::SelectByKey(String::from("a1")),
                    ClaimPath::SelectByKey(String::from("a2")),
                ],
                vec_nonempty![
                    ClaimPath::SelectByKey(String::from("a")),
                    ClaimPath::SelectByKey(String::from("a1")),
                    ClaimPath::SelectByKey(String::from("a3")),
                ],
            ];

            assert_eq!(
                result.claim_paths(AttributesTraversalBehaviour::OnlyLeaves),
                expected_leaves
            );
        }

        #[test]
        fn test_complex() {
            let result: Attributes = complex_attributes().into();

            let expected_all = vec![
                vec_nonempty![ClaimPath::SelectByKey(String::from("birth_date"))],
                vec_nonempty![
                    ClaimPath::SelectByKey(String::from("place_of_birth")),
                    ClaimPath::SelectByKey(String::from("locality")),
                ],
                vec_nonempty![
                    ClaimPath::SelectByKey(String::from("place_of_birth")),
                    ClaimPath::SelectByKey(String::from("country")),
                    ClaimPath::SelectByKey(String::from("name")),
                ],
                vec_nonempty![
                    ClaimPath::SelectByKey(String::from("place_of_birth")),
                    ClaimPath::SelectByKey(String::from("country")),
                    ClaimPath::SelectByKey(String::from("area_code")),
                ],
                vec_nonempty![
                    ClaimPath::SelectByKey(String::from("place_of_birth")),
                    ClaimPath::SelectByKey(String::from("country")),
                ],
                vec_nonempty![ClaimPath::SelectByKey(String::from("place_of_birth"))],
                vec_nonempty![
                    ClaimPath::SelectByKey(String::from("financial")),
                    ClaimPath::SelectByKey(String::from("has_debt")),
                ],
                vec_nonempty![
                    ClaimPath::SelectByKey(String::from("financial")),
                    ClaimPath::SelectByKey(String::from("has_job")),
                ],
                vec_nonempty![
                    ClaimPath::SelectByKey(String::from("financial")),
                    ClaimPath::SelectByKey(String::from("debt_amount")),
                ],
                vec_nonempty![ClaimPath::SelectByKey(String::from("financial"))],
            ];

            assert_eq!(result.claim_paths(AttributesTraversalBehaviour::AllPaths), expected_all);

            let expected_leaves = vec![
                vec_nonempty![ClaimPath::SelectByKey(String::from("birth_date"))],
                vec_nonempty![
                    ClaimPath::SelectByKey(String::from("place_of_birth")),
                    ClaimPath::SelectByKey(String::from("locality")),
                ],
                vec_nonempty![
                    ClaimPath::SelectByKey(String::from("place_of_birth")),
                    ClaimPath::SelectByKey(String::from("country")),
                    ClaimPath::SelectByKey(String::from("name")),
                ],
                vec_nonempty![
                    ClaimPath::SelectByKey(String::from("place_of_birth")),
                    ClaimPath::SelectByKey(String::from("country")),
                    ClaimPath::SelectByKey(String::from("area_code")),
                ],
                vec_nonempty![
                    ClaimPath::SelectByKey(String::from("financial")),
                    ClaimPath::SelectByKey(String::from("has_debt")),
                ],
                vec_nonempty![
                    ClaimPath::SelectByKey(String::from("financial")),
                    ClaimPath::SelectByKey(String::from("has_job")),
                ],
                vec_nonempty![
                    ClaimPath::SelectByKey(String::from("financial")),
                    ClaimPath::SelectByKey(String::from("debt_amount")),
                ],
            ];

            assert_eq!(
                result.claim_paths(AttributesTraversalBehaviour::OnlyLeaves),
                expected_leaves
            );
        }
    }
}
