use std::iter::Peekable;

use indexmap::IndexMap;
use itertools::Itertools;
use nutype::nutype;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Number;
use serde_with::FromInto;
use serde_with::serde_as;
use serde_with::skip_serializing_none;

use attestation_types::claim_path::ClaimPath;
use utils::vec_at_least::VecNonEmptyUnique;

use crate::disclosure::Disclosure;
use crate::disclosure::DisclosureContent;
use crate::disclosure::DisclosureContentSerializationError;
use crate::encoder::SdObjectEncoder;
use crate::error::Error;
use crate::error::Result;
use crate::hasher::Hasher;

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
    pub fn digests(&self) -> Vec<(String, HashType)> {
        let object_digests = self
            ._sd
            .iter()
            .flat_map(|digests| digests.iter().map(|digest| (digest.clone(), HashType::Object)));

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

    fn conceal<H: Hasher>(&mut self, key: ClaimName, salt: String, hasher: &H) -> Result<Disclosure> {
        // Remove the value from the object
        let value_to_conceal = self
            .remove(&key)
            .ok_or_else(|| Error::ObjectFieldNotFound(key.clone(), self.clone()))?;

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
        let hash = hasher.encoded_digest(disclosure.encoded());

        // Add the hash to the "_sd" array if exists; otherwise, create the array and insert the hash.
        self.push_digest(hash);

        Ok(disclosure)
    }

    /// Push `new_digest` to the digests in `_sd`. Maintains alphabetical ordering if possible.
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
    ) -> Result<Vec<&'a str>>
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
                            return Err(Error::IntermediateElementNotFound { path: key.clone() });
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
                        .ok_or_else(|| Error::ElementNotFound { path: key.clone() })?;

                    digests.push(digest);
                }
                Ok(digests)
            }
            _ => Err(Error::UnexpectedElement(
                ClaimValue::Object(self.clone()),
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
}

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
    ) -> Result<Option<&'a mut ClaimValue>> {
        claim_paths.try_fold(Some(self), |maybe_object, claim_path| {
            maybe_object.map_or(Ok(None), |object| object.traverse(claim_path))
        })
    }

    fn traverse<'a>(&'a mut self, claim_path: &ClaimPath) -> Result<Option<&'a mut ClaimValue>> {
        match (self, claim_path) {
            (ClaimValue::Array(array), ClaimPath::SelectByIndex(index)) => match array.get_mut(*index) {
                Some(array_claim) => Ok(Some(array_claim.as_mut_value()?)),
                None => Ok(None),
            },
            (ClaimValue::Object(object), ClaimPath::SelectByKey(key)) => {
                Ok(object.claims.get_mut(&key.parse::<ClaimName>()?))
            }
            (_, ClaimPath::SelectAll) => Err(Error::UnsupportedTraversalPath(ClaimPath::SelectAll)),
            (element, path) => Err(Error::UnexpectedElement(element.clone(), vec![path.clone()])),
        }
    }

    /// Recursively discover all placeholder digests for arrays and objects.
    pub fn digests(&self) -> Vec<(String, HashType)> {
        match self {
            ClaimValue::Array(claims) => claims.iter().flat_map(ArrayClaim::digests).collect(),
            ClaimValue::Object(object) => object.digests(),
            // There are no digests in any primitive value.
            _ => Default::default(),
        }
    }

    fn as_mut_object(&mut self) -> Result<&mut ObjectClaims> {
        if let Self::Object(object) = self {
            Ok(object)
        } else {
            Err(Error::DataTypeMismatch(format!(
                "expected JSON Object, but got {:?}",
                self
            )))
        }
    }

    fn as_mut_array(&mut self) -> Result<&mut Vec<ArrayClaim>> {
        if let Self::Array(array) = self {
            Ok(array)
        } else {
            Err(Error::DataTypeMismatch(format!(
                "expected JSON Array, but got {:?}",
                self
            )))
        }
    }

    pub(crate) fn conceal<H: Hasher>(&mut self, path: &ClaimPath, salt: String, hasher: &H) -> Result<Disclosure> {
        match path {
            ClaimPath::SelectByKey(key) => self.as_mut_object()?.conceal(key.parse()?, salt, hasher),
            ClaimPath::SelectByIndex(index) => {
                //
                let array = self.as_mut_array()?;
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
                        let hash = hasher.encoded_digest(disclosure.encoded());
                        *value = ArrayClaim::Hash(hash);
                        Ok(disclosure)
                    })
                    .unwrap_or_else(|| Err(Error::IndexOutOfBounds(*index, array.clone())))
            }
            ClaimPath::SelectAll => Err(Error::UnsupportedTraversalPath(path.clone())),
        }
    }

    pub(crate) fn add_decoy<H: Hasher>(&mut self, path: &[ClaimPath], hasher: &H, salt_len: usize) -> Result<()> {
        let Some(parent) = self.traverse_by_claim_paths(path.iter())? else {
            return Err(Error::ParentNotFound(path.to_vec()));
        };
        parent.add_decoy_here(hasher, salt_len)
    }

    fn add_decoy_here<H: Hasher>(&mut self, hasher: &H, salt_len: usize) -> Result<()> {
        match self {
            ClaimValue::Array(array) => {
                array.push(ArrayClaim::Hash(SdObjectEncoder::random_digest(
                    hasher, salt_len, true,
                )?));
                Ok(())
            }
            ClaimValue::Object(object) => {
                object.push_digest(SdObjectEncoder::random_digest(hasher, salt_len, false)?);
                Ok(())
            }
            _ => Err(Error::UnexpectedElement(self.clone(), vec![])),
        }
    }

    pub(crate) fn digests_to_disclose<'a, I>(
        &'a self,
        path: &mut Peekable<I>,
        disclosures: &'a IndexMap<String, Disclosure>,
        traversing_array: bool,
    ) -> Result<Vec<&'a str>>
    where
        I: ExactSizeIterator<Item = &'a ClaimPath>,
    {
        // Holds all digests that should be disclosed based on the `path`
        let mut digests = vec![];

        // If we are traversing an array, peekable shouldn't consume the next value
        let (element_key, has_next) = if traversing_array {
            (*path.peek().ok_or(Error::EmptyPath)?, path.len() > 1)
        } else {
            (path.next().ok_or(Error::EmptyPath)?, path.peek().is_some())
        };

        match (self, element_key) {
            (ClaimValue::Object(object_claims), _) => {
                object_claims.digests_to_disclose(path, disclosures, element_key, has_next)
            }
            (ClaimValue::Array(array_claims), ClaimPath::SelectByIndex(index)) if has_next => {
                let entry = array_claims.get(*index).ok_or_else(|| Error::ElementNotFoundInArray {
                    path: element_key.clone(),
                })?;

                if let Some(next_object) = entry.process_digests_to_disclose(disclosures, &mut digests)? {
                    digests.append(&mut next_object.digests_to_disclose(path, disclosures, false)?);
                } else {
                    return Err(Error::ElementNotFoundInArray {
                        path: element_key.clone(),
                    });
                }

                Ok(digests)
            }
            (ClaimValue::Array(array_claims), ClaimPath::SelectByIndex(index)) => {
                let entry = array_claims.get(*index).ok_or_else(|| Error::ElementNotFoundInArray {
                    path: element_key.clone(),
                })?;

                // If the array entry is an array-selective-disclosure object, then we'll add the digest to the
                // list of digests to disclose.
                if let ArrayClaim::Hash(digest) = entry {
                    digests.push(digest);
                }
                Ok(digests)
            }
            (ClaimValue::Array(array_claims), ClaimPath::SelectAll) => {
                for entry in array_claims {
                    let next_object = entry
                        .process_digests_to_disclose(disclosures, &mut digests)?
                        .ok_or_else(|| Error::ElementNotFoundInArray {
                            path: element_key.clone(),
                        })?;

                    if has_next {
                        digests.append(&mut next_object.digests_to_disclose(path, disclosures, true)?);
                    }
                }
                Ok(digests)
            }
            (element, _) => Err(Error::UnexpectedElement(element.clone(), path.cloned().collect_vec())),
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum ArrayClaim {
    Hash(#[serde_as(as = "FromInto<DisclosureHash>")] String),
    Value(ClaimValue),
}

impl ArrayClaim {
    pub fn digests(&self) -> Vec<(String, HashType)> {
        match &self {
            ArrayClaim::Hash(hash) => vec![(hash.clone(), HashType::Array)],
            ArrayClaim::Value(value) => value.digests(),
        }
    }

    fn as_mut_value(&mut self) -> Result<&mut ClaimValue> {
        if let Self::Value(value) = self {
            Ok(value)
        } else {
            Err(Error::DataTypeMismatch(
                "expected JSON array element, but got an array digest".to_string(),
            ))
        }
    }

    fn process_digests_to_disclose<'a>(
        &'a self,
        disclosures: &'a IndexMap<String, Disclosure>,
        digests: &mut Vec<&'a str>,
    ) -> Result<Option<&'a ClaimValue>> {
        let result = match self {
            ArrayClaim::Hash(digest) => {
                // We're disclosing something within a selectively disclosable array entry.
                // For the verifier to be able to verify that, we'll also have to disclose that entry.
                digests.push(digest);

                match disclosures.get(digest) {
                    Some(disclosure) => {
                        let (_, value) = disclosure.content.try_as_array_element(digest)?;
                        value.process_digests_to_disclose(disclosures, digests)?
                    }
                    None => None,
                }
            }
            ArrayClaim::Value(entry) => {
                // This array entry is not selectively disclosable, so we just return it verbatim.
                Some(entry)
            }
        };
        Ok(result)
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

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct DisclosureHash {
    #[serde(rename = "...")]
    hash: String,
}

impl From<String> for DisclosureHash {
    fn from(hash: String) -> Self {
        Self { hash }
    }
}

impl From<DisclosureHash> for String {
    fn from(value: DisclosureHash) -> Self {
        value.hash
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HashType {
    Array,
    Object,
}

#[cfg(test)]
mod tests {
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
        assert_eq!(object_claims._sd.unwrap().len(), 1.try_into().unwrap());

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
            let array_claims = array_claims.as_array().unwrap();

            assert_eq!(array_claims.len(), 3);
            assert_matches!(array_claims[0], ArrayClaim::Value(_));
            assert_matches!(array_claims[1], ArrayClaim::Hash(_));
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
        let array_claims = array_claims.as_array().unwrap();

        assert_eq!(array_claims.len(), 3);
        assert_matches!(array_claims[0], ArrayClaim::Value(_));
        assert_matches!(array_claims[1], ArrayClaim::Hash(_));
        assert_matches!(array_claims[2], ArrayClaim::Value(_));

        assert_matches!(
            disclosure.content,
            DisclosureContent::ArrayElement(_, ArrayClaim::Hash(_))
        );
    }

    #[rstest]
    #[case(json!("John Doe"), ClaimPath::SelectByKey("name".to_string()), "DataTypeMismatch(\"expected JSON Object, but got String(\\\"John Doe\\\")\")")]
    #[case(json!("John Doe"), ClaimPath::SelectByIndex(0), "DataTypeMismatch(\"expected JSON Array, but got String(\\\"John Doe\\\")\")")]
    #[case(json!("John Doe"), ClaimPath::SelectAll, "UnsupportedTraversalPath(SelectAll)")]
    #[case(json!(30), ClaimPath::SelectByKey("name".to_string()), "DataTypeMismatch(\"expected JSON Object, but got Number(Number(30))\")")]
    #[case(json!(30), ClaimPath::SelectByIndex(0), "DataTypeMismatch(\"expected JSON Array, but got Number(Number(30))\")")]
    #[case(json!(30), ClaimPath::SelectAll, "UnsupportedTraversalPath(SelectAll)")]
    #[case(json!(false), ClaimPath::SelectByKey("name".to_string()), "DataTypeMismatch(\"expected JSON Object, but got Bool(false)\")")]
    #[case(json!(false), ClaimPath::SelectByIndex(0), "DataTypeMismatch(\"expected JSON Array, but got Bool(false)\")")]
    #[case(json!(false), ClaimPath::SelectAll, "UnsupportedTraversalPath(SelectAll)")]
    #[case(json!(null), ClaimPath::SelectByKey("name".to_string()), "DataTypeMismatch(\"expected JSON Object, but got Null\")")]
    #[case(json!(null), ClaimPath::SelectByIndex(0), "DataTypeMismatch(\"expected JSON Array, but got Null\")")]
    #[case(json!(null), ClaimPath::SelectAll, "UnsupportedTraversalPath(SelectAll)")]
    fn test_claim_value_conceal_primitives(
        #[case] value: serde_json::Value,
        #[case] path: ClaimPath,
        #[case] expected: &str,
    ) {
        let mut claim_value: ClaimValue = serde_json::from_value(value).unwrap();

        let hasher = Sha256Hasher;
        let error = claim_value.conceal(&path, "salt123".to_string(), &hasher).unwrap_err();

        assert_eq!(format!("{error:?}").as_str(), expected);
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
}
