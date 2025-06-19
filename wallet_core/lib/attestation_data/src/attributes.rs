use std::collections::VecDeque;
use std::num::TryFromIntError;

use derive_more::AsRef;
use derive_more::From;
use indexmap::IndexMap;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;

use mdoc::iso::mdocs::Entry;
use mdoc::iso::mdocs::NameSpace;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;
use utils::vec_at_least::VecNonEmpty;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttributeValue {
    Integer(i64),
    Bool(bool),
    Text(String),
    Array(Vec<AttributeValue>),
}

#[derive(Debug, thiserror::Error)]
pub enum AttributeError {
    #[error("unable to convert mdoc cbor value: {0:?}")]
    FromCborConversion(ciborium::Value),

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
            ciborium::Value::Array(elements) => Ok(AttributeValue::Array(
                elements.into_iter().map(Self::try_from).try_collect()?,
            )),
            _ => Err(AttributeError::FromCborConversion(value)),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Attribute {
    Single(AttributeValue),
    Nested(IndexMap<String, Attribute>),
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize, AsRef, From)]
pub struct Attributes(#[from] IndexMap<String, Attribute>);

impl Attributes {
    pub fn into_inner(self) -> IndexMap<String, Attribute> {
        self.0
    }

    /// Returns a flattened view of the attribute values
    pub fn flattened(&self) -> IndexMap<VecNonEmpty<&str>, &AttributeValue> {
        let mut result = IndexMap::with_capacity(self.0.len());
        let mut to_process: VecDeque<(Vec<&str>, &IndexMap<String, Attribute>)> = VecDeque::from([(vec![], &self.0)]);

        while let Some((prefix, attributes)) = to_process.pop_front() {
            let mut to_add = Vec::with_capacity(attributes.len());
            for (name, attribute) in attributes {
                let path = prefix.iter().copied().chain([name.as_str()]).collect();
                match attribute {
                    Attribute::Single(attribute) => {
                        // Guaranteed to have single entry
                        result.insert(VecNonEmpty::try_from(path).unwrap(), attribute);
                    }
                    Attribute::Nested(nested) => {
                        to_add.push((path, nested));
                    }
                }
            }

            // Push items in reverse to front to maintain order
            to_add.into_iter().rev().for_each(|item| to_process.push_front(item))
        }
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
        // No internal attributes can be in the array map as they are forbidden as claim in the type metadata
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

        if keys.len() == 1 {
            if let Some(entries) = attributes.get_mut(prefix) {
                Self::insert_entry(keys[0], entries, result)
                    .map_err(|error| AttributesError::AttributeError(format!("{}.{}", prefix, keys[0]), error))?;

                if entries.is_empty() {
                    attributes.swap_remove(prefix);
                }
            }
        } else {
            let prefixed_key = format!("{}.{}", prefix, keys[0]);

            if let Attribute::Nested(result) = result
                .entry(String::from(keys[0]))
                .or_insert_with(|| Attribute::Nested(IndexMap::new()))
            {
                Self::traverse_attributes_by_claim(&prefixed_key, &keys[1..], attributes, result)?
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
            let mut prefix: Vec<&str> = std::iter::once(attestation_type).chain(path.iter().copied()).collect();
            // path is non-empty so it has at least one element
            let name = prefix.pop().unwrap().to_string();
            result
                .entry(prefix.iter().join("."))
                .or_insert_with(Vec::new)
                .push(Entry {
                    name,
                    value: attribute.clone().into(),
                })
        }
        result
    }
}

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;
    use indexmap::IndexMap;
    use serde_json::json;
    use serde_valid::json::ToJsonString;

    use mdoc::Entry;
    use mdoc::NameSpace;
    use sd_jwt_vc_metadata::NormalizedTypeMetadata;
    use utils::vec_at_least::VecNonEmpty;

    use crate::attributes::Attribute;
    use crate::attributes::AttributeValue;
    use crate::attributes::Attributes;
    use crate::attributes::AttributesError;

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
                "house": {
                    "number": 1,
                    "letter": "A"
                }
            })
        );
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
            ],
            "schema": { "properties": {} }
        });
        let type_metadata = NormalizedTypeMetadata::from_single_example(serde_json::from_value(metadata_json).unwrap());

        let result = example_attributes().validate(&type_metadata);
        assert_matches!(result, Err(AttributesError::AttributesWithoutClaim(message)) if message == vec![vec!["country", "area_code"]]);
    }
}
