use std::collections::VecDeque;
use std::num::TryFromIntError;

use indexmap::IndexMap;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;

use sd_jwt_vc_metadata::NormalizedTypeMetadata;

use crate::identifiers::DataElementIdentifier;
use crate::identifiers::NameSpace;

pub type DataElementValue = ciborium::Value;

/// An attribute name and value.
///
/// See also [`IssuerSignedItem`](super::IssuerSignedItem), which additionally contains the attribute's `random` and
/// `digestID`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub name: DataElementIdentifier,
    pub value: DataElementValue,
}

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

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Attributes(IndexMap<String, Attribute>);

impl Default for Attributes {
    fn default() -> Self {
        Self::new()
    }
}

impl From<IndexMap<String, Attribute>> for Attributes {
    fn from(attributes: IndexMap<String, Attribute>) -> Self {
        Self(attributes)
    }
}

impl Attributes {
    pub fn new() -> Self {
        IndexMap::new().into()
    }

    pub fn into_inner(self) -> IndexMap<String, Attribute> {
        self.0
    }

    pub fn as_inner(&self) -> &IndexMap<String, Attribute> {
        &self.0
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
    ) -> Result<Self, AttributeError> {
        let mut result = IndexMap::new();

        // The claims list determines the final order of the converted attributes.
        for claim in type_metadata.claims() {
            // First, confirm that the path is made up of key entries by converting to a `Vec<&str>`.
            let key_path = claim
                .path
                .iter()
                .filter_map(|path| path.try_key_path())
                .collect::<VecDeque<_>>();

            Self::traverse_attributes_by_claim(type_metadata.vct(), key_path, &mut attributes, &mut result)?;
        }

        if !attributes.is_empty() {
            return Err(AttributeError::SomeAttributesNotProcessed(attributes));
        }
        // No internal attributes can be in the array map as they are forbidden as claim in the type metadata

        Ok(result.into())
    }

    fn traverse_attributes_by_claim(
        prefix: &str,
        mut keys: VecDeque<&str>,
        attributes: &mut IndexMap<String, Vec<Entry>>,
        result: &mut IndexMap<String, Attribute>,
    ) -> Result<(), AttributeError> {
        if attributes.is_empty() {
            return Ok(());
        }

        if let Some(key) = keys.pop_front() {
            if keys.is_empty() {
                if let Some(entries) = attributes.get_mut(prefix) {
                    Self::insert_entry(key, entries, result)?;

                    if entries.is_empty() {
                        attributes.swap_remove(prefix);
                    }
                }
            } else {
                let prefixed_key = format!("{}.{}", prefix, key);

                if let Attribute::Nested(result) = result
                    .entry(String::from(key))
                    .or_insert_with(|| Attribute::Nested(IndexMap::new()))
                {
                    Self::traverse_attributes_by_claim(&prefixed_key, keys, attributes, result)?
                }
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
        let mut flattened = IndexMap::new();
        Self::walk_attributes_recursive(attestation_type, self.0, &mut flattened);
        flattened
    }

    fn walk_attributes_recursive(
        namespace: &str,
        attributes: IndexMap<String, Attribute>,
        result: &mut IndexMap<NameSpace, Vec<Entry>>,
    ) {
        let mut entries = vec![];
        for (key, value) in attributes {
            match value {
                Attribute::Single(single) => {
                    entries.push(Entry {
                        name: key,
                        value: single.into(),
                    });
                }
                Attribute::Nested(nested) => {
                    let key = format!("{}.{}", namespace, key);
                    Self::walk_attributes_recursive(key.as_str(), nested, result);
                }
            }
        }

        if !entries.is_empty() {
            result.insert(String::from(namespace), entries);
        }
    }
}

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;
    use indexmap::IndexMap;
    use serde_json::json;
    use serde_valid::json::ToJsonString;

    use sd_jwt_vc_metadata::NormalizedTypeMetadata;

    use crate::attributes::Attribute;
    use crate::attributes::AttributeError;
    use crate::attributes::AttributeValue;
    use crate::attributes::Attributes;
    use crate::identifiers::NameSpace;

    use super::Entry;

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
        assert_matches!(result, Err(AttributeError::SomeAttributesNotProcessed(attrs))
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
    fn test_attribute_to_mdoc_attributes() {
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
    fn test_attribute_to_mdoc_attributes_empty_root() {
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
}
