use std::num::TryFromIntError;

use derive_more::AsRef;
use derive_more::Display;
use derive_more::From;
use indexmap::IndexMap;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;

use mdoc::iso::mdocs::Entry;
use mdoc::iso::mdocs::NameSpace;
use sd_jwt_vc_metadata::ClaimPath;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;
use utils::vec_at_least::VecNonEmpty;

#[derive(Debug, Clone, Display, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttributeValue {
    Null,
    Integer(i64),
    Bool(bool),
    Text(String),
    #[display("[{}]", _0.iter().join(", "))]
    Array(Vec<AttributeValue>),
}

#[derive(Debug, thiserror::Error)]
pub enum AttributeError {
    #[error("unable to convert mdoc cbor value: {0:?}")]
    FromCborConversion(Box<ciborium::Value>),

    #[error("unable to convert number to cbor: {0}")]
    NumberToCborConversion(#[from] TryFromIntError),
}

#[derive(Debug, thiserror::Error)]
pub enum AttributesError {
    #[error("attributes without claim: {0:?}")]
    AttributesWithoutClaim(Vec<Vec<String>>),

    #[error("attribute error at {0}: {1}")]
    AttributeError(String, #[source] AttributeError),

    #[error("some attributes have not been processed by metadata: {0:?}")]
    SomeAttributesNotProcessed(IndexMap<String, Vec<Entry>>),
}

impl From<AttributeValue> for ciborium::Value {
    fn from(value: AttributeValue) -> Self {
        match value {
            AttributeValue::Integer(number) => ciborium::Value::Integer(number.into()),
            AttributeValue::Bool(boolean) => ciborium::Value::Bool(boolean),
            AttributeValue::Text(text) => ciborium::Value::Text(text),
            AttributeValue::Null => ciborium::Value::Null,
            AttributeValue::Array(elements) => ciborium::Value::Array(elements.into_iter().map(Self::from).collect()),
        }
    }
}

impl TryFrom<ciborium::Value> for AttributeValue {
    type Error = AttributeError;

    fn try_from(value: ciborium::Value) -> Result<Self, Self::Error> {
        match value {
            ciborium::Value::Text(text) => Ok(AttributeValue::Text(text)),
            ciborium::Value::Bool(bool) => Ok(AttributeValue::Bool(bool)),
            ciborium::Value::Integer(integer) => Ok(AttributeValue::Integer(integer.try_into()?)),
            ciborium::Value::Null => Ok(AttributeValue::Null),
            ciborium::Value::Array(elements) => Ok(AttributeValue::Array(
                elements.into_iter().map(Self::try_from).try_collect()?,
            )),
            _ => Err(AttributeError::FromCborConversion(Box::new(value))),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "test"), derive(derive_more::Unwrap))]
#[serde(untagged)]
pub enum Attribute {
    Single(AttributeValue),
    Nested(IndexMap<String, Attribute>),
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize, AsRef, From)]
pub struct Attributes(IndexMap<String, Attribute>);

impl Attributes {
    pub fn into_inner(self) -> IndexMap<String, Attribute> {
        self.0
    }

    /// Returns a flattened view of the attribute values
    pub fn flattened(&self) -> IndexMap<VecNonEmpty<&str>, &AttributeValue> {
        /// Recursive depth first traversal helper to flatten all leaf nodes.
        ///
        /// - `prefix` is the path to the current level
        /// - `attrs` are the attributes at the current nesting level
        /// - `result` is a running index map of attributes by path (deepest-first)
        fn traverse_depth_first<'a>(
            prefix: &[&'a str],
            attrs: &'a IndexMap<String, Attribute>,
            result: &mut IndexMap<VecNonEmpty<&'a str>, &'a AttributeValue>,
        ) {
            attrs.iter().for_each(|(key, attr)| {
                let path = prefix
                    .iter()
                    .copied()
                    .chain(std::iter::once(key.as_str()))
                    .collect_vec();

                match attr {
                    Attribute::Nested(nested) => {
                        traverse_depth_first(&path, nested, result);
                    }
                    Attribute::Single(attribute) => {
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
        let mut flattened_attributes = self.flattened();
        for claim_key_path in type_metadata.claim_key_paths() {
            flattened_attributes.swap_remove(&claim_key_path);
        }
        if !flattened_attributes.is_empty() {
            return Err(AttributesError::AttributesWithoutClaim(
                flattened_attributes
                    .into_iter()
                    .map(|(path, _)| path.into_iter().map(ToString::to_string).collect())
                    .collect(),
            ));
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
    /// Note in particular that attributes in a namespace whose names equals the attestation_type in the metadata
    /// are mapped to the root level of the output.
    pub fn from_mdoc_attributes(
        type_metadata: &NormalizedTypeMetadata,
        mut attributes: IndexMap<NameSpace, Vec<Entry>>,
    ) -> Result<Self, AttributesError> {
        // Get the claim paths consisting only out of claim key paths
        let key_paths = type_metadata.claim_key_paths();

        let mut result = IndexMap::with_capacity(key_paths.len());

        // The key paths of the claims determines the order of the attributes result
        for key_path in key_paths {
            Self::traverse_attributes_by_claim(type_metadata.vct(), key_path.as_slice(), &mut attributes, &mut result)?;
        }

        if !attributes.is_empty() {
            return Err(AttributesError::SomeAttributesNotProcessed(attributes));
        }

        Ok(Self(result))
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
                        .map_err(|error| AttributesError::AttributeError(format!("{prefix}.{head}"), error))?;

                    if entries.is_empty() {
                        attributes.swap_remove(prefix);
                    }
                }
            }
            [head, ..] => {
                let prefixed_key = format!("{prefix}.{head}");

                if let Attribute::Nested(result) = result
                    .entry(String::from(head))
                    .or_insert_with(|| Attribute::Nested(IndexMap::new()))
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
            group.insert(entry.name, Attribute::Single(entry.value.try_into()?));
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

    pub fn claim_paths(&self) -> Vec<VecNonEmpty<ClaimPath>> {
        /// Recursive depth first traversal helper to collect all claim paths from nested attributes.
        ///
        /// Depth first is necessary because the SD-JWT conceal functionality for leaves doesn't work properly if the
        /// parent node is already concealed (and therefore not present anymore in the resulting claims).
        ///
        /// - `prefix` is the path to the current level
        /// - `attrs` are the attributes at the current nesting level
        /// - `result` is a running collection of all full paths (deepest-first)
        fn traverse_depth_first(
            prefix: &[ClaimPath],
            attrs: &IndexMap<String, Attribute>,
            result: &mut Vec<VecNonEmpty<ClaimPath>>,
        ) {
            for (key, attr) in attrs {
                let path = prefix
                    .iter()
                    .cloned()
                    .chain(std::iter::once(ClaimPath::SelectByKey(key.clone())))
                    .collect_vec();

                // If it's a nested attribute, recurse deeper first
                if let Attribute::Nested(nested) = attr {
                    traverse_depth_first(&path, nested, result);
                }

                // Push current path after children have been processed (post-order)
                result.push(VecNonEmpty::try_from(path).unwrap());
            }
        }

        let mut result = Vec::with_capacity(self.0.len());
        traverse_depth_first(&[], self.as_ref(), &mut result);
        result
    }

    /// Retrieve the attribute value at the specified location, if it exists.
    ///
    /// NB: for now only all claim paths must be strings.
    pub fn get(
        &self,
        claim_paths: &VecNonEmpty<ClaimPath>,
    ) -> Result<Option<&AttributeValue>, AttributesHandlingError> {
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
                Attribute::Single(_) => return Ok(None),
                Attribute::Nested(map) => match map.get(claim_path) {
                    Some(map) => map,
                    None => return Ok(None),
                },
            };
        }

        let attr = match attr {
            Attribute::Single(value) => value,
            _ => return Ok(None),
        };

        Ok(Some(attr))
    }

    /// Insert the specified attribute at the specified location.
    ///
    /// NB: for now only all claim paths must be strings.
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
                    .or_insert_with(|| Attribute::Nested(IndexMap::new()));

                // If the attribute is a leaf the claim path is longer than expected and thus invalid.
                let child_map = match attribute {
                    Attribute::Single(_) => return Err(AttributesHandlingError::InvalidClaimPath),
                    Attribute::Nested(map) => map,
                };

                Ok(child_map)
            })?;

        // If the last claim path is not already present, insert the attribute.
        let last_claim_path = claim_paths
            .last()
            .try_key_path()
            .ok_or(AttributesHandlingError::InvalidClaimPath)?;

        match leaf_map.get(last_claim_path) {
            Some(Attribute::Single(_)) => Err(AttributesHandlingError::ClaimAlreadyExists),
            Some(Attribute::Nested(_)) => Err(AttributesHandlingError::InvalidClaimPath),
            None => {
                leaf_map.insert(last_claim_path.to_string(), attribute);

                Ok(())
            }
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum AttributesHandlingError {
    #[error("invalid claim path")]
    InvalidClaimPath,
    #[error("cannot insert claim: already exists")]
    ClaimAlreadyExists,
}

#[cfg(test)]
pub mod test {
    use assert_matches::assert_matches;
    use indexmap::IndexMap;
    use rstest::rstest;
    use serde_json::json;
    use serde_valid::json::ToJsonString;

    use mdoc::Entry;
    use mdoc::NameSpace;
    use sd_jwt_vc_metadata::ClaimPath;
    use sd_jwt_vc_metadata::NormalizedTypeMetadata;
    use utils::vec_at_least::VecNonEmpty;

    use crate::attributes::Attribute;
    use crate::attributes::AttributeValue;
    use crate::attributes::Attributes;
    use crate::attributes::AttributesError;
    use crate::attributes::AttributesHandlingError;

    pub fn complex_attributes() -> IndexMap<String, Attribute> {
        IndexMap::from([
            (
                String::from("birth_date"),
                Attribute::Single(AttributeValue::Text(String::from("1963-08-12"))),
            ),
            (
                String::from("place_of_birth"),
                Attribute::Nested(IndexMap::from([
                    (
                        String::from("locality"),
                        Attribute::Single(AttributeValue::Text(String::from("The Hague"))),
                    ),
                    (
                        String::from("country"),
                        Attribute::Nested(IndexMap::from([
                            (
                                String::from("name"),
                                Attribute::Single(AttributeValue::Text(String::from("The Netherlands"))),
                            ),
                            (
                                String::from("area_code"),
                                Attribute::Single(AttributeValue::Integer(33)),
                            ),
                        ])),
                    ),
                ])),
            ),
            (
                String::from("financial"),
                Attribute::Nested(IndexMap::from([
                    (String::from("has_debt"), Attribute::Single(AttributeValue::Bool(true))),
                    (String::from("has_job"), Attribute::Single(AttributeValue::Bool(false))),
                    (
                        String::from("debt_amount"),
                        Attribute::Single(AttributeValue::Integer(-10_000)),
                    ),
                ])),
            ),
        ])
    }

    #[test]
    fn test_traverse_groups() {
        let metadata_json = json!({
            "vct": "com.example.pid",
            "display": [{"lang": "en", "name": "example"}],
            "claims": [{
                "path": ["birthdate"],
                "display": [{"lang": "en", "label": "birthdate"}],
            }, {
                "path": ["place_of_birth", "locality"],
                "display": [{"lang": "en", "label": "birth city"}],
            }, {
                "path": ["place_of_birth", "country", "name"],
                "display": [{"lang": "en", "label": "birth country"}],
            }, {
                "path": ["place_of_birth", "country", "area_code"],
                "display": [{"lang": "en", "label": "birth area code"}],
            }, {
                "path": ["a", "b", "c", "d", "e"],
                "display": [{"lang": "en", "label": "a b c d e"}],
            }, {
                "path": ["a", "b", "c1"],
                "display": [{"lang": "en", "label": "a b c1"}],
            }],
            "schema": { "properties": {} }
        });
        let type_metadata = NormalizedTypeMetadata::from_single_example(serde_json::from_value(metadata_json).unwrap());

        let mdoc_attributes = IndexMap::from([
            (
                String::from("com.example.pid"),
                vec![Entry {
                    name: String::from("birthdate"),
                    value: ciborium::Value::Text(String::from("1963-08-12")),
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
            "birthdate": "1963-08-12",
            "place_of_birth": {
                "locality": "The Hague",
                "country": {
                    "name": "The Netherlands",
                    "area_code": 31
                }
            },
            "a": {
                "b": {
                    "c": {
                        "d":{
                            "e": "abcd"
                        },
                    },
                    "c1": "abc",
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
            "display": [{"lang": "en", "name": "example"}],
            "claims": [
                {
                    "path": ["nest.ed", "birth.date"],
                    "display": [{"lang": "en", "label": "nested birthday"}],
                }
            ],
            "schema": { "properties": {} }
        });
        let type_metadata = NormalizedTypeMetadata::from_single_example(serde_json::from_value(metadata_json).unwrap());

        let mdoc_attributes = IndexMap::from([(
            String::from("com.example.pid.nest.ed"),
            vec![Entry {
                name: String::from("birth.date"),
                value: ciborium::Value::Text(String::from("1963-08-12")),
            }],
        )]);

        let result = Attributes::from_mdoc_attributes(&type_metadata, mdoc_attributes).unwrap();

        let expected_json = json!({"nest.ed": { "birth.date": "1963-08-12" }});
        assert_eq!(
            serde_json::to_value(result).unwrap().to_json_string_pretty().unwrap(),
            expected_json.to_json_string_pretty().unwrap(),
        );
    }

    #[test]
    fn test_traverse_groups_with_extra_entry_not_in_claim() {
        let metadata_json = json!({
            "vct": "com.example.pid",
            "display": [{"lang": "en", "name": "example"}],
            "claims": [
                {
                    "path": ["a", "a1"],
                    "display": [{"lang": "en", "label": "a a1"}],
                },
                {
                    "path": ["a", "a2"],
                    "display": [{"lang": "en", "label": "a a1"}],
                }
            ],
            "schema": { "properties": {} }
        });
        let type_metadata = NormalizedTypeMetadata::from_single_example(serde_json::from_value(metadata_json).unwrap());

        let mdoc_attributes = IndexMap::from([(
            String::from("com.example.pid.a"),
            vec![
                Entry {
                    name: String::from("a1"),
                    value: ciborium::Value::Text(String::from("1")),
                },
                Entry {
                    name: String::from("a2"),
                    value: ciborium::Value::Text(String::from("2")),
                },
                Entry {
                    name: String::from("a3"),
                    value: ciborium::Value::Text(String::from("3")),
                },
            ],
        )]);

        let result = Attributes::from_mdoc_attributes(&type_metadata, mdoc_attributes);
        assert_matches!(result, Err(AttributesError::SomeAttributesNotProcessed(attrs))
        if attrs == IndexMap::from([(
            String::from("com.example.pid.a"),
            vec![Entry { name: String::from("a3"), value: ciborium::Value::Text(String::from("3")) }]
        )]));
    }

    #[test]
    fn test_traverse_groups_claim_ordering() {
        let metadata_json = json!({
            "vct": "com.example.pid",
            "display": [{"lang": "en", "name": "example"}],
            "claims": [
                {
                    "path": ["b", "b1"],
                    "display": [{"lang": "en", "label": "b b1"}],
                },
                {
                    "path": ["b", "b3"],
                    "display": [{"lang": "en", "label": "b b3"}],
                },
                {
                    "path": ["b", "b2"],
                    "display": [{"lang": "en", "label": "b b2"}],
                }
            ],
            "schema": { "properties": {} }
        });
        let type_metadata = NormalizedTypeMetadata::from_single_example(serde_json::from_value(metadata_json).unwrap());

        let mdoc_attributes = IndexMap::from([(
            String::from("com.example.pid.b"),
            vec![
                Entry {
                    name: String::from("b1"),
                    value: ciborium::Value::Text(String::from("1")),
                },
                Entry {
                    name: String::from("b2"),
                    value: ciborium::Value::Text(String::from("2")),
                },
                Entry {
                    name: String::from("b3"),
                    value: ciborium::Value::Text(String::from("3")),
                },
            ],
        )]);

        let result = Attributes::from_mdoc_attributes(&type_metadata, mdoc_attributes).unwrap();
        let expected_json = json!({"b": { "b1": "1", "b3": "3", "b2": "2" }});
        assert_eq!(
            serde_json::to_value(result).unwrap().to_json_string_pretty().unwrap(),
            expected_json.to_json_string_pretty().unwrap(),
        );
    }

    fn setup_issuable_attributes() -> Attributes {
        IndexMap::from_iter(vec![
            (
                "city".to_string(),
                Attribute::Single(AttributeValue::Text("The Capital".to_string())),
            ),
            ("postal_code".to_string(), Attribute::Single(AttributeValue::Null)),
            (
                "street".to_string(),
                Attribute::Single(AttributeValue::Text("Main St.".to_string())),
            ),
            (
                "house".to_string(),
                Attribute::Nested(IndexMap::from_iter(vec![
                    ("number".to_string(), Attribute::Single(AttributeValue::Integer(1))),
                    (
                        "letter".to_string(),
                        Attribute::Single(AttributeValue::Text("A".to_string())),
                    ),
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
                "city": "The Capital",
                "street": "Main St.",
                "postal_code": null,
                "house": {
                    "number": 1,
                    "letter": "A"
                }
            })
        );
    }

    #[rstest]
    #[case(
        vec![ClaimPath::SelectByKey("house".to_string()), ClaimPath::SelectByKey("number".to_string())],
        Ok(Some(&AttributeValue::Integer(1))))
    ]
    #[case(
        vec![ClaimPath::SelectByKey("house".to_string()), ClaimPath::SelectByKey("foobar".to_string())],
        Ok(None))
    ]
    #[case(
        vec![ClaimPath::SelectByKey("foobar".to_string())],
        Ok(None))
    ]
    #[case(
        vec![ClaimPath::SelectByKey("city".to_string()), ClaimPath::SelectByKey("number".to_string())],
        Ok(None))
    ]
    #[case(vec![ClaimPath::SelectByKey("house".to_string())], Ok(None))]
    #[case(vec![ClaimPath::SelectByIndex(1)], Err(AttributesHandlingError::InvalidClaimPath))]
    #[case(vec![ClaimPath::SelectAll], Err(AttributesHandlingError::InvalidClaimPath))]
    fn test_attributes_get(
        #[case] claim_paths: Vec<ClaimPath>,
        #[case] expected: Result<Option<&AttributeValue>, AttributesHandlingError>,
    ) {
        let attributes = setup_issuable_attributes();
        let claim_paths = &claim_paths.try_into().unwrap();

        assert_eq!(attributes.get(claim_paths), expected);
    }

    #[rstest]
    #[case(
        vec![ClaimPath::SelectByKey("foo".to_string())],
        Ok(json!({
            "outer": { "inner": "value" },
            "foo": true
        }))
    )]
    #[case(
        vec![ClaimPath::SelectByKey("outer".to_string()), ClaimPath::SelectByKey("foo".to_string())],
        Ok(json!({
            "outer": { "inner": "value", "foo": true },
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
            Attribute::Nested(IndexMap::from_iter([(
                "inner".to_string(),
                Attribute::Single(AttributeValue::Text("value".to_string())),
            )])),
        )])
        .into();

        let result = attributes
            .insert(
                &claim_paths.try_into().unwrap(),
                Attribute::Single(AttributeValue::Bool(true)),
            )
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
            Attribute::Nested(IndexMap::from_iter(vec![(
                "number".to_string(),
                Attribute::Single(AttributeValue::Integer(1)),
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
            (
                "name".to_string(),
                Attribute::Single(AttributeValue::Text("Wallet".to_string())),
            ),
            (
                "address".to_string(),
                Attribute::Nested(IndexMap::from([
                    (
                        "street".to_string(),
                        Attribute::Single(AttributeValue::Text("Gracht".to_string())),
                    ),
                    ("number".to_string(), Attribute::Single(AttributeValue::Integer(123))),
                ])),
            ),
            (
                "country".to_string(),
                Attribute::Nested(IndexMap::from([
                    (
                        "iso".to_string(),
                        Attribute::Single(AttributeValue::Text("NL".to_string())),
                    ),
                    ("area_code".to_string(), Attribute::Single(AttributeValue::Integer(31))),
                ])),
            ),
            ("adult".to_string(), Attribute::Single(AttributeValue::Bool(true))),
        ])
        .into()
    }

    #[test]
    fn test_attributes_flattened() {
        assert_eq!(
            example_attributes().flattened(),
            IndexMap::from([
                (
                    VecNonEmpty::try_from(vec!["name"]).unwrap(),
                    &AttributeValue::Text("Wallet".to_string())
                ),
                (
                    VecNonEmpty::try_from(vec!["address", "street"]).unwrap(),
                    &AttributeValue::Text("Gracht".to_string())
                ),
                (
                    VecNonEmpty::try_from(vec!["address", "number"]).unwrap(),
                    &AttributeValue::Integer(123)
                ),
                (
                    VecNonEmpty::try_from(vec!["country", "iso"]).unwrap(),
                    &AttributeValue::Text("NL".to_string())
                ),
                (
                    VecNonEmpty::try_from(vec!["country", "area_code"]).unwrap(),
                    &AttributeValue::Integer(31)
                ),
                (
                    VecNonEmpty::try_from(vec!["adult"]).unwrap(),
                    &AttributeValue::Bool(true)
                ),
            ]),
        );
    }

    #[test]
    fn test_validate_ok() {
        let metadata_json = json!({
            "vct": "com.example.pid",
            "display": [{"lang": "en", "name": "example"}],
            "claims": [
                {
                    "path": ["name"],
                    "display": [{"lang": "en", "label": "name"}],
                },
                {
                    "path": ["birth_date"],
                    "display": [{"lang": "en", "label": "birth date"}],
                },
                {
                    "path": ["address", "street"],
                    "display": [{"lang": "en", "label": "address street"}],
                },
                {
                    "path": ["address", "number"],
                    "display": [{"lang": "en", "label": "address number"}],
                },
                {
                    "path": ["country", "iso"],
                    "display": [{"lang": "en", "label": "country iso"}],
                },
                {
                    "path": ["country", "area_code"],
                    "display": [{"lang": "en", "label": "country area code"}],
                },
                {
                    "path": ["adult"],
                    "display": [{"lang": "en", "label": "adult"}],
                },
            ],
            "schema": { "properties": {} }
        });
        let type_metadata = NormalizedTypeMetadata::from_single_example(serde_json::from_value(metadata_json).unwrap());

        let result = example_attributes().validate(&type_metadata);
        assert_matches!(result, Ok(_));
    }

    #[test]
    fn test_validate_missing() {
        let metadata_json = json!({
            "vct": "com.example.pid",
            "display": [{"lang": "en", "name": "example"}],
            "claims": [
                {
                    "path": ["name"],
                    "display": [{"lang": "en", "label": "name"}],
                },
                {
                    "path": ["birth_date"],
                    "display": [{"lang": "en", "label": "birth date"}],
                },
                {
                    "path": ["address", "street"],
                    "display": [{"lang": "en", "label": "address street"}],
                },
                {
                    "path": ["address", "number"],
                    "display": [{"lang": "en", "label": "address number"}],
                },
                {
                    "path": ["country", "iso"],
                    "display": [{"lang": "en", "label": "country iso"}],
                },
                {
                    "path": ["adult"],
                    "display": [{"lang": "en", "label": "adult"}],
                },
            ],
            "schema": { "properties": {} }
        });
        let type_metadata = NormalizedTypeMetadata::from_single_example(serde_json::from_value(metadata_json).unwrap());

        let result = example_attributes().validate(&type_metadata);
        assert_matches!(result, Err(AttributesError::AttributesWithoutClaim(message)) if message == vec![vec!["country", "area_code"]]);
    }

    mod test_claim_paths_from_attributes {
        use sd_jwt_vc_metadata::ClaimPath;
        use utils::vec_at_least::VecNonEmpty;

        use super::*;

        #[test]
        fn single_attribute_should_return_correct_claimpaths() {
            let result: Attributes = IndexMap::from([(
                String::from("a"),
                Attribute::Single(AttributeValue::Text(String::from("1234"))),
            )])
            .into();

            let expected: Vec<VecNonEmpty<_>> =
                vec![VecNonEmpty::try_from(vec![ClaimPath::SelectByKey(String::from("a"))]).unwrap()];

            assert_eq!(result.claim_paths(), expected);
        }

        #[test]
        fn nested_attribute_should_return_correct_claimpaths() {
            let result: Attributes = IndexMap::from([
                (
                    String::from("b"),
                    Attribute::Single(AttributeValue::Text(String::from("1234"))),
                ),
                (
                    String::from("a"),
                    Attribute::Nested(IndexMap::from([(
                        String::from("a1"),
                        Attribute::Nested(IndexMap::from([
                            (
                                String::from("a2"),
                                Attribute::Single(AttributeValue::Text(String::from("1234"))),
                            ),
                            (
                                String::from("a3"),
                                Attribute::Single(AttributeValue::Text(String::from("1234"))),
                            ),
                        ])),
                    )])),
                ),
            ])
            .into();

            let expected: Vec<VecNonEmpty<_>> = vec![
                vec![ClaimPath::SelectByKey(String::from("b"))],
                vec![
                    ClaimPath::SelectByKey(String::from("a")),
                    ClaimPath::SelectByKey(String::from("a1")),
                    ClaimPath::SelectByKey(String::from("a2")),
                ],
                vec![
                    ClaimPath::SelectByKey(String::from("a")),
                    ClaimPath::SelectByKey(String::from("a1")),
                    ClaimPath::SelectByKey(String::from("a3")),
                ],
                vec![
                    ClaimPath::SelectByKey(String::from("a")),
                    ClaimPath::SelectByKey(String::from("a1")),
                ],
                vec![ClaimPath::SelectByKey(String::from("a"))],
            ]
            .into_iter()
            .map(|v| VecNonEmpty::try_from(v).unwrap())
            .collect();

            assert_eq!(result.claim_paths(), expected);
        }

        #[test]
        fn test_complex() {
            let result: Attributes = complex_attributes().into();

            let expected: Vec<VecNonEmpty<_>> = vec![
                vec![ClaimPath::SelectByKey(String::from("birth_date"))],
                vec![
                    ClaimPath::SelectByKey(String::from("place_of_birth")),
                    ClaimPath::SelectByKey(String::from("locality")),
                ],
                vec![
                    ClaimPath::SelectByKey(String::from("place_of_birth")),
                    ClaimPath::SelectByKey(String::from("country")),
                    ClaimPath::SelectByKey(String::from("name")),
                ],
                vec![
                    ClaimPath::SelectByKey(String::from("place_of_birth")),
                    ClaimPath::SelectByKey(String::from("country")),
                    ClaimPath::SelectByKey(String::from("area_code")),
                ],
                vec![
                    ClaimPath::SelectByKey(String::from("place_of_birth")),
                    ClaimPath::SelectByKey(String::from("country")),
                ],
                vec![ClaimPath::SelectByKey(String::from("place_of_birth"))],
                vec![
                    ClaimPath::SelectByKey(String::from("financial")),
                    ClaimPath::SelectByKey(String::from("has_debt")),
                ],
                vec![
                    ClaimPath::SelectByKey(String::from("financial")),
                    ClaimPath::SelectByKey(String::from("has_job")),
                ],
                vec![
                    ClaimPath::SelectByKey(String::from("financial")),
                    ClaimPath::SelectByKey(String::from("debt_amount")),
                ],
                vec![ClaimPath::SelectByKey(String::from("financial"))],
            ]
            .into_iter()
            .map(|v| VecNonEmpty::try_from(v).unwrap())
            .collect();

            assert_eq!(result.claim_paths(), expected);
        }
    }
}
