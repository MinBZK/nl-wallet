use std::collections::HashMap;
use std::iter::Peekable;
use std::iter::repeat_n;

use derive_more::Display;
use indexmap::IndexMap;
use indexmap::IndexSet;
use itertools::Itertools;
use nutype::nutype;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Number;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use thiserror::Error;

use attestation_types::claim_path::ClaimPath;
use sd_jwt_vc_metadata::ClaimSelectiveDisclosureMetadata;
use utils::single_unique::MultipleItemsFound;
use utils::single_unique::SingleUnique;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_at_least::VecNonEmptyUnique;
use utils::vec_nonempty;

use crate::disclosure::Disclosure;
use crate::disclosure::DisclosureContent;
use crate::disclosure::DisclosureContentSerializationError;
use crate::encoder::SdObjectEncoder;
use crate::error::ClaimError;
use crate::error::EncoderError;
use crate::hasher::Hasher;
use crate::sd_jwt::verify_selective_disclosability;

#[nutype(
    validate(predicate = |name| !["...", "_sd"].contains(&name)),
    derive(Debug, Clone, TryFrom, FromStr, Into, AsRef, Display, PartialEq, Eq, Hash, Serialize, Deserialize)
)]
pub struct ClaimName(String);

impl ClaimName {
    #[inline]
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct ObjectClaims {
    /// Selectively disclosable claims of the SD-JWT.
    pub _sd: Option<VecNonEmptyUnique<String>>,

    /// Non-selectively disclosable claims of the SD-JWT.
    #[serde(flatten)]
    pub claims: IndexMap<ClaimName, ClaimValue>,
}

impl ObjectClaims {
    pub fn digests(&self) -> Vec<(String, ClaimType)> {
        let object_digests = self
            ._sd
            .iter()
            .flat_map(|digests| digests.iter().map(|digest| (digest.clone(), ClaimType::Object)));

        self.claims
            .values()
            .flat_map(ClaimValue::digests)
            .chain(object_digests)
            .collect()
    }

    pub fn get(&self, key: &ClaimName) -> Option<&ClaimValue> {
        self.claims.get(key)
    }

    fn remove(&mut self, key: &ClaimName) -> Option<ClaimValue> {
        self.claims.shift_remove(key)
    }

    fn insert(&mut self, key: ClaimName, value: ClaimValue) -> Option<ClaimValue> {
        self.claims.insert(key, value)
    }

    fn conceal<H: Hasher>(&mut self, key: ClaimName, salt: String, hasher: &H) -> Result<Disclosure, EncoderError> {
        // Remove the value from the object
        let value_to_conceal = self
            .remove(&key)
            .ok_or_else(|| ClaimError::ObjectFieldNotFound(key.clone(), Box::new(self.clone())))?;

        // Create a disclosure for the value
        let disclosure = Disclosure::try_new(DisclosureContent::ObjectProperty(salt, key, value_to_conceal)).map_err(
            |DisclosureContentSerializationError { content, error }| {
                let DisclosureContent::ObjectProperty(_, key, value) = *content else {
                    unreachable!()
                };
                // In case of an error, restore the removed entry so that the original object is intact
                self.insert(key, value);
                error
            },
        )?;

        // Hash the disclosure.
        let digest = hasher.encoded_digest(disclosure.encoded());

        // Add the digest to the "_sd" array if it exists; otherwise, create the array and insert the digest.
        self.push_digest(digest);

        Ok(disclosure)
    }

    /// Push `new_digest` to the digests in `_sd`. Maintains alphabetical ordering if possible, as recommended in:
    /// <https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-22.html#section-4.2.4.1>
    fn push_digest(&mut self, new_digest: String) {
        if self._sd.is_none() {
            // `try_new` will always return `Ok` because the newly created vec is not empty with a single unique value
            self._sd = VecNonEmptyUnique::try_from(vec![new_digest]).ok();
        } else {
            // Make sure the digests are sorted.
            let (new_digest_option, mut digests) = self._sd.take().unwrap().into_inner().into_iter().fold(
                (Some(new_digest), vec![]),
                |(new_digest, mut digests), digest| match new_digest {
                    Some(new_digest) if digest.as_str() > new_digest.as_str() => {
                        digests.push(new_digest);
                        digests.push(digest);
                        (None, digests)
                    }
                    new_digest_option => {
                        digests.push(digest);
                        (new_digest_option, digests)
                    }
                },
            );
            if let Some(new_digest) = new_digest_option {
                digests.push(new_digest);
            }

            // `try_new` will always return `Ok` because digests is non-empty
            self._sd = VecNonEmptyUnique::try_from(digests).ok();
        }
    }

    fn digests_to_disclose<'a, I>(
        &'a self,
        path: &mut Peekable<I>,
        disclosures: &'a IndexMap<String, Disclosure>,
        element_key: &'a ClaimPath,
        has_next: bool,
    ) -> Result<Vec<&'a str>, ClaimError>
    where
        I: ExactSizeIterator<Item = &'a ClaimPath>,
    {
        // Holds all digests that should be disclosed based on the `path`
        let mut digests = vec![];

        match element_key {
            // We are just traversing to a deeper part of the object.
            ClaimPath::SelectByKey(key) if has_next => {
                let next_object = match self.claims.get(&key.parse::<ClaimName>()?) {
                    Some(claim_value) => claim_value,
                    None => {
                        let disclosure = self.find_disclosure_digest(key, disclosures).and_then(|digest| {
                            // We're disclosing something within the current object, which is selectively disclosable.
                            // For the verifier to be able to verify that, we'll also have to disclose the current
                            // object.
                            digests.push(digest);
                            disclosures.get(digest)
                        });
                        if let Some(disclosure) = disclosure {
                            let (_, _, claim_value) = disclosure.content.try_as_object_property(key)?;
                            claim_value
                        } else {
                            return Err(ClaimError::IntermediateElementNotFound(key.clone()));
                        }
                    }
                };

                digests.append(&mut next_object.digests_to_disclose(path, disclosures, false)?);
                Ok(digests)
            }
            // We reached the the value we want to disclose, so add it to the list of digests
            ClaimPath::SelectByKey(key) => {
                // If the value exists within the object, it is not selectively disclosable and we do not have to look
                // for the associated disclosure.
                // Otherwise we do look for the associated disclosure.
                if !self.claims.contains_key(&key.parse::<ClaimName>()?) {
                    let digest = self
                        .find_disclosure_digest(key, disclosures)
                        .ok_or_else(|| ClaimError::ElementNotFound(key.clone()))?;

                    digests.push(digest);
                }
                Ok(digests)
            }
            _ => Err(ClaimError::UnexpectedElement(
                Box::new(ClaimValue::Object(self.clone())),
                path.cloned().collect_vec(),
            )),
        }
    }

    fn find_disclosure_digest<'a>(
        &'a self,
        key: &str,
        disclosures: &'a IndexMap<String, Disclosure>,
    ) -> Option<&'a str> {
        self._sd.as_ref().and_then(|digests| {
            digests.iter().map(String::as_str).find(|digest| {
                disclosures
                    .get(*digest)
                    .and_then(|disclosure| match &disclosure.content {
                        DisclosureContent::ObjectProperty(_, name, _) => Some(name),
                        _ => None,
                    })
                    .is_some_and(|name| name.as_str() == key)
            })
        })
    }

    fn verify_selective_disclosability(
        &self,
        claim_path: &[ClaimPath],
        claim_path_index: usize,
        disclosures: &IndexMap<String, Disclosure>,
        sd_metadata: &HashMap<Vec<ClaimPath>, ClaimSelectiveDisclosureMetadata>,
        claim: &str,
    ) -> Result<(), ClaimError> {
        let head = &claim_path[..=claim_path_index];

        let has_next = !claim_path[claim_path_index + 1..].is_empty();

        let should_be_disclosable = sd_metadata
            .get(&head.to_vec())
            .unwrap_or(&ClaimSelectiveDisclosureMetadata::Allowed);

        let claim_value = if let Some(claim_value) = self.get(&claim.parse()?) {
            verify_selective_disclosability(should_be_disclosable, false, head)?;
            claim_value
        } else if let Some(digest) = self.find_disclosure_digest(claim, disclosures) {
            verify_selective_disclosability(should_be_disclosable, true, head)?;
            // unwrap is safe, because `find_disclosure_digest` returned `Some`
            let disclosure = disclosures.get(digest).unwrap();
            let (_, _, claim_value) = disclosure.content.try_as_object_property(digest)?;
            claim_value
        } else {
            return Err(ClaimError::ObjectFieldNotFound(claim.parse()?, Box::new(self.clone())));
        };

        if has_next {
            claim_value.verify_selective_disclosability(claim_path, claim_path_index + 1, disclosures, sd_metadata)?;
        }
        Ok(())
    }

    fn non_selectively_disclosable_claims(
        &self,
    ) -> Result<IndexSet<VecNonEmpty<ClaimPath>>, NonSelectivelyDisclosableClaimsError> {
        self.claims
            .iter()
            .map(|(name, claim)| {
                let path = ClaimPath::SelectByKey(name.to_string());
                let sub_claims = claim.non_selectively_disclosable_claims()?;
                Ok(prefix_all(sub_claims, path))
            })
            .flatten_ok()
            .try_collect()
    }
}

#[derive(Debug, Error)]
pub enum NonSelectivelyDisclosableClaimsError {
    #[error("invalid array structure")]
    ArrayStructure(#[from] MultipleItemsFound),
}

/// Inserts `[path]` at the beginning of all elements in `[sub_claims]`.
fn prefix_all(sub_claims: IndexSet<VecNonEmpty<ClaimPath>>, path: ClaimPath) -> IndexSet<VecNonEmpty<ClaimPath>> {
    let sub_claims_len = sub_claims.len();
    sub_claims
        .into_iter()
        .zip(repeat_n(path.clone(), sub_claims_len))
        .map(|(mut claim, path)| {
            claim.insert(0, path);
            claim
        })
        .chain([vec_nonempty![path]])
        .collect()
}

#[cfg_attr(test, derive(derive_more::Unwrap), unwrap(ref))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum ClaimValue {
    Array(Vec<ArrayClaim>),
    Object(ObjectClaims),
    #[default]
    Null,
    Bool(bool),
    Number(Number),
    String(String),
}

impl ClaimValue {
    pub(crate) fn traverse_by_claim_paths<'a, 'b>(
        &'a mut self,
        mut claim_paths: impl Iterator<Item = &'b ClaimPath>,
    ) -> Result<Option<&'a mut ClaimValue>, ClaimError> {
        claim_paths.try_fold(Some(self), |maybe_object, claim_path| {
            maybe_object.map_or(Ok(None), |object| object.traverse(claim_path))
        })
    }

    fn traverse<'a>(&'a mut self, claim_path: &ClaimPath) -> Result<Option<&'a mut ClaimValue>, ClaimError> {
        match (self, claim_path) {
            (ClaimValue::Array(array), ClaimPath::SelectByIndex(index)) => match array.get_mut(*index) {
                Some(ArrayClaim::Value(value)) => Ok(Some(value)),
                Some(ArrayClaim::Hash { .. }) => Err(ClaimError::ExpectedArrayElement(claim_path.to_owned())),
                None => Ok(None),
            },
            (ClaimValue::Object(object), ClaimPath::SelectByKey(key)) => {
                Ok(object.claims.get_mut(&key.parse::<ClaimName>()?))
            }
            (_, ClaimPath::SelectAll) => Err(ClaimError::UnsupportedTraversalPath(ClaimPath::SelectAll)),
            (element, path) => Err(ClaimError::UnexpectedElement(
                Box::new(element.clone()),
                vec![path.clone()],
            )),
        }
    }

    /// Recursively discover all placeholder digests for arrays and objects.
    pub fn digests(&self) -> Vec<(String, ClaimType)> {
        match self {
            ClaimValue::Array(claims) => claims.iter().flat_map(ArrayClaim::digests).collect(),
            ClaimValue::Object(object) => object.digests(),
            // There are no digests in any primitive value.
            _ => Default::default(),
        }
    }

    pub(crate) fn conceal<H: Hasher>(
        &mut self,
        path: &ClaimPath,
        salt: String,
        hasher: &H,
    ) -> Result<Disclosure, EncoderError> {
        match path {
            ClaimPath::SelectByKey(key) => {
                let Self::Object(object) = self else {
                    return Err(ClaimError::ClaimTypeMismatch {
                        expected: ClaimType::Object,
                        actual: self.clone().into(),
                        path: path.to_owned(),
                    })?;
                };
                object.conceal(key.parse().map_err(ClaimError::ReservedClaimName)?, salt, hasher)
            }
            ClaimPath::SelectByIndex(index) => {
                let Self::Array(array) = self else {
                    return Err(ClaimError::ClaimTypeMismatch {
                        expected: ClaimType::Array,
                        actual: self.clone().into(),
                        path: path.to_owned(),
                    })?;
                };
                array
                    .get_mut(*index)
                    .map(|value| {
                        let element = std::mem::take(value);
                        let disclosure = Disclosure::try_new(DisclosureContent::ArrayElement(salt, element)).map_err(
                            |DisclosureContentSerializationError { content, error, .. }| {
                                // In case of an error, restore the removed entry so that the original array is intact
                                let DisclosureContent::ArrayElement(_, array_element) = *content else {
                                    unreachable!()
                                };
                                *value = array_element;
                                error
                            },
                        )?;
                        let digest = hasher.encoded_digest(disclosure.encoded());
                        *value = ArrayClaim::Hash { digest };
                        Ok(disclosure)
                    })
                    .unwrap_or_else(|| Err(ClaimError::IndexOutOfBounds(*index, array.clone()))?)
            }
            ClaimPath::SelectAll => Err(ClaimError::UnsupportedTraversalPath(path.clone()))?,
        }
    }

    pub(crate) fn add_decoy<H: Hasher>(
        &mut self,
        path: &[ClaimPath],
        hasher: &H,
        salt_len: usize,
    ) -> Result<(), EncoderError> {
        let Some(parent) = self.traverse_by_claim_paths(path.iter())? else {
            return Err(ClaimError::ParentNotFound(path.to_vec()))?;
        };
        parent.add_decoy_here(hasher, salt_len)
    }

    fn add_decoy_here<H: Hasher>(&mut self, hasher: &H, salt_len: usize) -> Result<(), EncoderError> {
        match self {
            ClaimValue::Array(array) => {
                array.push(ArrayClaim::Hash {
                    digest: SdObjectEncoder::random_digest(hasher, salt_len, true)?,
                });
                Ok(())
            }
            ClaimValue::Object(object) => {
                object.push_digest(SdObjectEncoder::random_digest(hasher, salt_len, false)?);
                Ok(())
            }
            _ => Err(ClaimError::UnexpectedElement(Box::new(self.clone()), vec![]))?,
        }
    }

    pub(crate) fn digests_to_disclose<'a, I>(
        &'a self,
        path: &mut Peekable<I>,
        disclosures: &'a IndexMap<String, Disclosure>,
        traversing_array: bool,
    ) -> Result<Vec<&'a str>, ClaimError>
    where
        I: ExactSizeIterator<Item = &'a ClaimPath>,
    {
        // Holds all digests that should be disclosed based on the `path`
        let mut digests = vec![];

        // If we are traversing an array, peekable shouldn't consume the next value
        let (element_key, has_next) = if traversing_array {
            (*path.peek().ok_or(ClaimError::EmptyPath)?, path.len() > 1)
        } else {
            (path.next().ok_or(ClaimError::EmptyPath)?, path.peek().is_some())
        };

        match (self, element_key) {
            (ClaimValue::Object(object_claims), _) => {
                object_claims.digests_to_disclose(path, disclosures, element_key, has_next)
            }
            (ClaimValue::Array(array_claims), ClaimPath::SelectByIndex(index)) if has_next => {
                let entry = array_claims
                    .get(*index)
                    .ok_or_else(|| ClaimError::ElementNotFoundInArray(element_key.clone()))?;

                if let Some(next_object) = entry.process_digests_to_disclose(disclosures, &mut digests)? {
                    digests.append(&mut next_object.digests_to_disclose(path, disclosures, false)?);
                } else {
                    return Err(ClaimError::ElementNotFoundInArray(element_key.clone()));
                }

                Ok(digests)
            }
            (ClaimValue::Array(array_claims), ClaimPath::SelectByIndex(index)) => {
                let entry = array_claims
                    .get(*index)
                    .ok_or_else(|| ClaimError::ElementNotFoundInArray(element_key.clone()))?;

                // If the array entry is an array-selective-disclosure object, then we'll add the digest to the
                // list of digests to disclose.
                if let ArrayClaim::Hash { digest } = entry {
                    digests.push(digest.as_ref());
                }
                Ok(digests)
            }
            (ClaimValue::Array(array_claims), ClaimPath::SelectAll) => {
                for entry in array_claims {
                    let next_object = entry
                        .process_digests_to_disclose(disclosures, &mut digests)?
                        .ok_or_else(|| ClaimError::ElementNotFoundInArray(element_key.clone()))?;

                    if has_next {
                        digests.append(&mut next_object.digests_to_disclose(path, disclosures, true)?);
                    }
                }
                Ok(digests)
            }
            (element, _) => Err(ClaimError::UnexpectedElement(
                Box::new(element.clone()),
                path.cloned().collect_vec(),
            )),
        }
    }

    /// Traverses the claim structure and disclosures for [`claim_path`] and verifies for each path element whether its
    /// selective disclosability matches with the `sd_metadata`.
    ///
    /// Errors when the [`claim_paths`] does not resolve to an existing claim or the selective disclosability accorting
    /// to the `sd_metadata`.
    pub(crate) fn verify_selective_disclosability(
        &self,
        claim_path: &[ClaimPath],
        claim_path_index: usize,
        disclosures: &IndexMap<String, Disclosure>,
        sd_metadata: &HashMap<Vec<ClaimPath>, ClaimSelectiveDisclosureMetadata>,
    ) -> Result<(), ClaimError> {
        // Verify whether the index is valid
        if claim_path.len() <= claim_path_index {
            return Err(ClaimError::EmptyPath);
        }

        let path_item = &claim_path[claim_path_index];

        match (path_item, self) {
            (ClaimPath::SelectByIndex(index), _) => {
                Err(ClaimError::UnsupportedTraversalPath(ClaimPath::SelectByIndex(*index)))
            }
            (ClaimPath::SelectByKey(key), ClaimValue::Object(object_claims)) => object_claims
                .verify_selective_disclosability(claim_path, claim_path_index, disclosures, sd_metadata, key),
            (ClaimPath::SelectAll, ClaimValue::Array(array_claims)) => {
                for array_claim in array_claims {
                    array_claim.verify_selective_disclosability(
                        claim_path,
                        claim_path_index,
                        disclosures,
                        sd_metadata,
                    )?;
                }
                Ok(())
            }
            (_, claim_value) => Err(ClaimError::UnexpectedElement(
                Box::new(claim_value.clone()),
                claim_path[..=claim_path_index].to_vec(),
            )),
        }
    }

    pub(crate) fn non_selectively_disclosable_claims(
        &self,
    ) -> Result<IndexSet<VecNonEmpty<ClaimPath>>, NonSelectivelyDisclosableClaimsError> {
        match self {
            ClaimValue::Array(array_claims) => {
                let non_sd_claims_per_element: Vec<_> = array_claims
                    .iter()
                    .filter_map(|claim| match claim {
                        ArrayClaim::Value(value) => Some(value),
                        ArrayClaim::Hash { .. } => None,
                    })
                    .map(ClaimValue::non_selectively_disclosable_claims)
                    .try_collect()?;

                if !non_sd_claims_per_element.is_empty() {
                    // The `single_unique` requires that all sub-elements in the array are uniform, i.e. have the same
                    // `ClaimPath` structure. This requirement can be invalidated when the array contains a combination
                    // of primitive values, objects and arrays, e.g.:
                    // - [ 1, [] ]
                    // - [ 1, { "claim": 2 } ]
                    // Instances like this are considered invalid, and should be reported back to the issuer.
                    match non_sd_claims_per_element.into_iter().single_unique()? {
                        Some(sub_claims) => Ok(prefix_all(sub_claims, ClaimPath::SelectAll)),
                        None => Ok(IndexSet::from_iter([vec_nonempty![ClaimPath::SelectAll]])),
                    }
                } else {
                    // Nothing non-selectively disclosable here
                    Ok(IndexSet::new())
                }
            }
            ClaimValue::Object(object_claims) => object_claims.non_selectively_disclosable_claims(),
            ClaimValue::Null => Ok(IndexSet::new()),
            ClaimValue::Bool(_) => Ok(IndexSet::new()),
            ClaimValue::Number(_) => Ok(IndexSet::new()),
            ClaimValue::String(_) => Ok(IndexSet::new()),
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum ArrayClaim {
    Hash {
        #[serde(rename = "...")]
        digest: String,
    },
    Value(ClaimValue),
}

impl ArrayClaim {
    pub fn digests(&self) -> Vec<(String, ClaimType)> {
        match &self {
            ArrayClaim::Hash { digest } => vec![(digest.to_string(), ClaimType::Array)],
            ArrayClaim::Value(value) => value.digests(),
        }
    }

    fn process_digests_to_disclose<'a>(
        &'a self,
        disclosures: &'a IndexMap<String, Disclosure>,
        digests: &mut Vec<&'a str>,
    ) -> Result<Option<&'a ClaimValue>, ClaimError> {
        match self {
            ArrayClaim::Hash { digest } => {
                // We're disclosing something within a selectively disclosable array entry.
                // For the verifier to be able to verify that, we'll also have to disclose that entry.
                digests.push(digest.as_ref());

                match disclosures.get(digest) {
                    Some(disclosure) => {
                        let (_, value) = disclosure.content.try_as_array_element(digest.as_ref())?;
                        value.process_digests_to_disclose(disclosures, digests)
                    }
                    None => Ok(None),
                }
            }
            ArrayClaim::Value(entry) => {
                // This array entry is not selectively disclosable, so we just return it verbatim.
                Ok(Some(entry))
            }
        }
    }

    pub(crate) fn resolve_to_value<'a>(
        &'a self,
        disclosures: &'a IndexMap<String, Disclosure>,
    ) -> Result<Option<&'a ClaimValue>, ClaimError> {
        match self {
            ArrayClaim::Hash { digest } => {
                if let Some(disclosure) = disclosures.get(digest) {
                    let (_, array_claim) = disclosure.content.try_as_array_element(digest)?;
                    array_claim.resolve_to_value(disclosures)
                } else {
                    Ok(None)
                }
            }
            ArrayClaim::Value(claim_value) => Ok(Some(claim_value)),
        }
    }

    fn verify_selective_disclosability(
        &self,
        claim_path: &[ClaimPath],
        claim_path_index: usize,
        disclosures: &IndexMap<String, Disclosure>,
        sd_metadata: &HashMap<Vec<ClaimPath>, ClaimSelectiveDisclosureMetadata>,
    ) -> Result<(), ClaimError> {
        let head = &claim_path[..=claim_path_index];

        let has_next = !claim_path[claim_path_index + 1..].is_empty();

        let should_be_disclosable = sd_metadata
            .get(&head.to_vec())
            .unwrap_or(&ClaimSelectiveDisclosureMetadata::Allowed);

        let value = match self {
            ArrayClaim::Hash { digest } => {
                verify_selective_disclosability(should_be_disclosable, true, head)?;
                let Some(disclosure) = disclosures.get(digest) else {
                    // There is no disclosure for this digest, this is not an error because this could be a decoy hash.
                    return Ok(());
                };
                let (_, array_claim) = disclosure.content.try_as_array_element(digest)?;
                array_claim.resolve_to_value(disclosures)?
            }
            ArrayClaim::Value(value) => {
                verify_selective_disclosability(should_be_disclosable, false, head)?;
                Some(value)
            }
        };

        if let Some(value) = value
            && has_next
        {
            value.verify_selective_disclosability(claim_path, claim_path_index + 1, disclosures, sd_metadata)?;
        }

        Ok(())
    }
}

impl From<ClaimValue> for ArrayClaim {
    fn from(value: ClaimValue) -> Self {
        ArrayClaim::Value(value)
    }
}

impl Default for ArrayClaim {
    fn default() -> Self {
        ArrayClaim::Value(Default::default())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum ClaimType {
    Array,
    Object,
    String,
    Number,
    Bool,
    Null,
}

impl From<ClaimValue> for ClaimType {
    fn from(value: ClaimValue) -> Self {
        match value {
            ClaimValue::Array(_) => ClaimType::Array,
            ClaimValue::Object(_) => ClaimType::Object,
            ClaimValue::Null => ClaimType::Null,
            ClaimValue::Bool(_) => ClaimType::Bool,
            ClaimValue::Number(_) => ClaimType::Number,
            ClaimValue::String(_) => ClaimType::String,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use assert_matches::assert_matches;
    use rstest::rstest;
    use serde_json::json;

    use crate::hasher::Sha256Hasher;

    use super::*;

    #[test]
    fn test_object_claims_conceal() {
        let mut object_claims: ObjectClaims = serde_json::from_value(json!({
            "name": "John Doe",
            "age": 30,
            "is_student": false
        }))
        .unwrap();

        let hasher = Sha256Hasher;
        let disclosure = object_claims
            .conceal("name".try_into().unwrap(), "salt123".to_string(), &hasher)
            .unwrap();

        assert_eq!(object_claims.claims.len(), 2);
        assert!(object_claims._sd.is_some());
        assert_eq!(object_claims._sd.unwrap().len(), NonZeroUsize::MIN);

        assert_eq!(
            disclosure.content,
            DisclosureContent::ObjectProperty(
                "salt123".to_string(),
                "name".parse().unwrap(),
                ClaimValue::String("John Doe".to_string())
            )
        );
    }

    #[test]
    fn test_claim_value_conceal_array() {
        let mut array_claims: ClaimValue = serde_json::from_value(json!(["John Doe", 30, false])).unwrap();

        let hasher = Sha256Hasher;
        let disclosure = array_claims
            .conceal(&ClaimPath::SelectByIndex(1), "salt123".to_string(), &hasher)
            .unwrap();

        // First conceal should result in an Array element disclosure with a value array claim.
        {
            let array_claims = array_claims.unwrap_array_ref();

            assert_eq!(array_claims.len(), 3);
            assert_matches!(array_claims[0], ArrayClaim::Value(_));
            assert_matches!(array_claims[1], ArrayClaim::Hash { .. });
            assert_matches!(array_claims[2], ArrayClaim::Value(_));

            assert_eq!(
                disclosure.content,
                DisclosureContent::ArrayElement(
                    "salt123".to_string(),
                    ArrayClaim::Value(ClaimValue::Number(30.into()))
                )
            );
        }

        let disclosure = array_claims
            .conceal(&ClaimPath::SelectByIndex(1), "salt123".to_string(), &hasher)
            .unwrap();

        // Second conceal should result in an Array element disclosure with a hash array claim.
        let array_claims = array_claims.unwrap_array_ref();

        assert_eq!(array_claims.len(), 3);
        assert_matches!(array_claims[0], ArrayClaim::Value(_));
        assert_matches!(array_claims[1], ArrayClaim::Hash { .. });
        assert_matches!(array_claims[2], ArrayClaim::Value(_));

        assert_matches!(
            disclosure.content,
            DisclosureContent::ArrayElement(_, ArrayClaim::Hash { .. })
        );
    }

    #[rstest]
    #[case(json!("John Doe"), "name".parse().unwrap(), ClaimError::ClaimTypeMismatch { expected: ClaimType::Object, actual: ClaimType::String, path: "name".parse().unwrap() })]
    #[case(json!("John Doe"), "0".parse().unwrap(), ClaimError::ClaimTypeMismatch { expected: ClaimType::Array, actual: ClaimType::String, path: ClaimPath::SelectByIndex(0) })]
    #[case(json!("John Doe"), ClaimPath::SelectAll, ClaimError::UnsupportedTraversalPath(ClaimPath::SelectAll))]
    #[case(json!(30), "name".parse().unwrap(), ClaimError::ClaimTypeMismatch { expected: ClaimType::Object, actual: ClaimType::Number, path: "name".parse().unwrap() })]
    #[case(json!(30), "0".parse().unwrap(), ClaimError::ClaimTypeMismatch { expected: ClaimType::Array, actual: ClaimType::Number, path: ClaimPath::SelectByIndex(0) })]
    #[case(json!(30), ClaimPath::SelectAll, ClaimError::UnsupportedTraversalPath(ClaimPath::SelectAll))]
    #[case(json!(false), "name".parse().unwrap(), ClaimError::ClaimTypeMismatch { expected: ClaimType::Object, actual: ClaimType::Bool, path: "name".parse().unwrap() })]
    #[case(json!(false), "0".parse().unwrap(), ClaimError::ClaimTypeMismatch { expected: ClaimType::Array, actual: ClaimType::Bool, path: ClaimPath::SelectByIndex(0) })]
    #[case(json!(false), ClaimPath::SelectAll, ClaimError::UnsupportedTraversalPath(ClaimPath::SelectAll))]
    #[case(json!(null), "name".parse().unwrap(), ClaimError::ClaimTypeMismatch { expected: ClaimType::Object, actual: ClaimType::Null, path: "name".parse().unwrap() })]
    #[case(json!(null), "0".parse().unwrap(), ClaimError::ClaimTypeMismatch { expected: ClaimType::Array, actual: ClaimType::Null, path: ClaimPath::SelectByIndex(0) })]
    #[case(json!(null), ClaimPath::SelectAll, ClaimError::UnsupportedTraversalPath(ClaimPath::SelectAll))]
    fn test_claim_value_conceal_primitives(
        #[case] value: serde_json::Value,
        #[case] path: ClaimPath,
        #[case] expected: ClaimError,
    ) {
        let mut claim_value: ClaimValue = serde_json::from_value(value).unwrap();

        let hasher = Sha256Hasher;
        let error = claim_value.conceal(&path, "salt123".to_string(), &hasher).unwrap_err();

        let EncoderError::ClaimStructure(claims_error) = error else {
            panic!("assertion failed: expected `EncoderError::ClaimsStructure(_)`");
        };
        assert_eq!(claims_error, expected);
    }

    #[test]
    fn test_object_claims_push_digest_alphabetic_ordering() {
        let mut object_claims: ObjectClaims = ObjectClaims::default();
        object_claims.push_digest("zebra".to_string());
        object_claims.push_digest("alpha".to_string());
        object_claims.push_digest("beta".to_string());
        object_claims.push_digest("gamma".to_string());
        object_claims.push_digest("delta".to_string());

        assert_eq!(
            object_claims
                ._sd
                .unwrap()
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            vec!["alpha", "beta", "delta", "gamma", "zebra"]
        );
    }

    #[test]
    fn deserialize_nested_string_array() {
        let expected = ClaimValue::Array(vec![ArrayClaim::Value(ClaimValue::Array(vec![ArrayClaim::Value(
            ClaimValue::String("string".to_string()),
        )]))]);

        let value = serde_json::to_value(&expected).unwrap();
        let claim: ClaimValue = serde_json::from_value(value).unwrap();

        assert_eq!(claim, expected);
    }

    #[rstest]
    #[case(json!(1), Ok(IndexSet::new()))]
    #[case(json!(true), Ok(IndexSet::new()))]
    #[case(json!(null), Ok(IndexSet::new()))]
    #[case(json!("".to_string()), Ok(IndexSet::new()))]
    #[case(json!({}), Ok(IndexSet::new()))]
    #[case(json!([]), Ok(IndexSet::new()))]
    #[case(json!({"value": 5}), Ok(IndexSet::from_iter([vec_nonempty![ClaimPath::SelectByKey("value".to_string())]])))]
    #[case(json!({"a": 1, "b": true}), Ok(IndexSet::from_iter([vec_nonempty![ClaimPath::SelectByKey("a".to_string())], vec_nonempty![ClaimPath::SelectByKey("b".to_string())]])))]
    #[case(json!([1, 2]), Ok(IndexSet::from_iter([vec_nonempty![ClaimPath::SelectAll]])))]
    #[case(json!([1, "a", true]), Ok(IndexSet::from_iter([vec_nonempty![ClaimPath::SelectAll]])))]
    #[case(json!([["a"], [2]]), Ok(IndexSet::from_iter([vec_nonempty![ClaimPath::SelectAll, ClaimPath::SelectAll], vec_nonempty![ClaimPath::SelectAll]])))]
    #[case(json!([{"a": 1}, {"a": 2}]), Ok(IndexSet::from_iter([vec_nonempty![ClaimPath::SelectAll, ClaimPath::SelectByKey("a".to_string())], vec_nonempty![ClaimPath::SelectAll]])))]
    #[case(json!({"a": [1, 2]}), Ok(IndexSet::from_iter([vec_nonempty![ClaimPath::SelectByKey("a".to_string()), ClaimPath::SelectAll], vec_nonempty![ClaimPath::SelectByKey("a".to_string())]])))]
    #[case(json!([1, { "a": 2 }]), Err(NonSelectivelyDisclosableClaimsError::ArrayStructure(MultipleItemsFound)))]
    #[case(json!([1, [2]]), Err(NonSelectivelyDisclosableClaimsError::ArrayStructure(MultipleItemsFound)))]
    #[case(json!([1, 2, 3, { "...": "some_digest" }]), Ok(IndexSet::from_iter([vec_nonempty![ClaimPath::SelectAll]])))]
    #[case(json!([{ "...": "some_digest" }]), Ok(IndexSet::new()))]
    fn non_selectively_disclosable_claims(
        #[case] value: serde_json::Value,
        #[case] expected_result: Result<IndexSet<VecNonEmpty<ClaimPath>>, NonSelectivelyDisclosableClaimsError>,
    ) {
        let value: ClaimValue = serde_json::from_value(value).unwrap();

        let result = value.non_selectively_disclosable_claims();
        match expected_result {
            Ok(expected) => assert_eq!(result.unwrap(), expected),
            Err(NonSelectivelyDisclosableClaimsError::ArrayStructure(_)) => {
                assert_matches!(
                    result.unwrap_err(),
                    NonSelectivelyDisclosableClaimsError::ArrayStructure(_)
                )
            }
        }
    }
}
