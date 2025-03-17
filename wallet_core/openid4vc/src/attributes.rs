use std::collections::VecDeque;
use std::num::TryFromIntError;

use indexmap::IndexMap;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;

use mdoc::unsigned::Entry;
use mdoc::DataElementValue;
use mdoc::NameSpace;
use sd_jwt::metadata::ClaimMetadata;
use sd_jwt::metadata::TypeMetadata;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttributeValue {
    Integer(i64),
    Bool(bool),
    Text(String),
}

#[derive(Debug, thiserror::Error)]
pub enum AttributeError {
    #[error("unable to convert mdoc value: {0:?}")]
    FromCborConversion(DataElementValue),

    #[error("unable to convert number to cbor: {0}")]
    NumberToCborConversion(#[from] TryFromIntError),

    #[error("attribute with name: {0} already exists")]
    DuplicateAttribute(String),

    #[error("some attributes have not been processed by metadata: {0:?}")]
    SomeAttributesNotProcessed(IndexMap<String, Vec<Entry>>),

    #[error("unable to convert from mdoc attributes because of unsupported claim path in: {0:?}")]
    UnsupportedClaimPath(ClaimMetadata),
}

impl From<&AttributeValue> for ciborium::Value {
    fn from(value: &AttributeValue) -> Self {
        match value {
            AttributeValue::Integer(number) => ciborium::Value::Integer((*number).into()),
            AttributeValue::Bool(boolean) => ciborium::Value::Bool(*boolean),
            AttributeValue::Text(text) => ciborium::Value::Text(text.to_owned()),
        }
    }
}

impl TryFrom<DataElementValue> for AttributeValue {
    type Error = AttributeError;

    fn try_from(value: DataElementValue) -> Result<Self, Self::Error> {
        match value {
            DataElementValue::Text(text) => Ok(AttributeValue::Text(text)),
            DataElementValue::Bool(bool) => Ok(AttributeValue::Bool(bool)),
            DataElementValue::Integer(integer) => Ok(AttributeValue::Integer(integer.try_into()?)),
            _ => Err(AttributeError::FromCborConversion(value)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Attribute {
    Single(AttributeValue),
    Nested(IndexMap<String, Attribute>),
}

impl Attribute {
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
        type_metadata: &TypeMetadata,
        mut attributes: IndexMap<NameSpace, Vec<Entry>>,
    ) -> Result<IndexMap<String, Self>, AttributeError> {
        let mut result = IndexMap::new();

        // The claims list determines the final order of the converted attributes.
        for claim in &type_metadata.as_ref().claims {
            // First, confirm that the path is made up of key entries by converting to a `Vec<&str>`.
            let key_path = claim
                .path
                .iter()
                .map(|path| {
                    path.try_key_path()
                        .ok_or(AttributeError::UnsupportedClaimPath(claim.clone()))
                })
                .collect::<Result<VecDeque<_>, _>>()?;

            Self::traverse_attributes_by_claim(&type_metadata.as_ref().vct, key_path, &mut attributes, &mut result)?;
        }

        if !attributes.is_empty() {
            return Err(AttributeError::SomeAttributesNotProcessed(attributes));
        }

        Ok(result)
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
                let prefixed_key = [prefix, key].join(".");

                if !result.contains_key(key) {
                    result.insert(String::from(key), Attribute::Nested(IndexMap::new()));
                }

                if let Some(Attribute::Nested(result)) = result.get_mut(key) {
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
        if let Some((index, _)) = entries.iter().find_position(|entry| entry.name == key) {
            // TODO: PVW-4188: this test will probably be obsolete after checking the internal consistency
            if group.contains_key(key) {
                return Err(AttributeError::DuplicateAttribute(String::from(key)));
            }

            let entry = entries.swap_remove(index);
            group.insert(entry.name, Attribute::Single(entry.value.try_into()?));
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;
    use indexmap::IndexMap;
    use serde_json::json;
    use serde_valid::json::ToJsonString;

    use mdoc::unsigned::Entry;
    use mdoc::DataElementValue;
    use sd_jwt::metadata::TypeMetadata;

    use crate::attributes::Attribute;
    use crate::attributes::AttributeError;

    #[test]
    fn test_traverse_groups() {
        let metadata_json = json!({
            "vct": "com.example.pid",
            "claims": [{
                "path": ["birthdate"],
            }, {
                "path": ["place_of_birth", "locality"],
            }, {
                "path": ["place_of_birth", "country", "name"],
            }, {
                "path": ["place_of_birth", "country", "area_code"],
            }, {
                "path": ["a", "b", "c", "d", "e"],
            }, {
                "path": ["a", "b", "c1"],
            }],
            "schema": { "properties": {} }
        });
        let type_metadata: TypeMetadata = serde_json::from_value(metadata_json).unwrap();

        let mdoc_attributes = IndexMap::from([
            (
                String::from("com.example.pid"),
                vec![Entry {
                    name: String::from("birthdate"),
                    value: DataElementValue::Text(String::from("1963-08-12")),
                }],
            ),
            (
                String::from("com.example.pid.place_of_birth"),
                vec![Entry {
                    name: String::from("locality"),
                    value: DataElementValue::Text(String::from("The Hague")),
                }],
            ),
            (
                String::from("com.example.pid.place_of_birth.country"),
                vec![
                    Entry {
                        name: String::from("name"),
                        value: DataElementValue::Text(String::from("The Netherlands")),
                    },
                    Entry {
                        name: String::from("area_code"),
                        value: DataElementValue::Integer(31.into()),
                    },
                ],
            ),
            (
                String::from("com.example.pid.a.b.c.d"),
                vec![Entry {
                    name: String::from("e"),
                    value: DataElementValue::Text(String::from("abcd")),
                }],
            ),
            (
                String::from("com.example.pid.a.b"),
                vec![Entry {
                    name: String::from("c1"),
                    value: DataElementValue::Text(String::from("abc")),
                }],
            ),
        ]);
        let result = Attribute::from_mdoc_attributes(&type_metadata, mdoc_attributes).unwrap();

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

    // TODO: PVW-4188: this test will probably be obsolete after checking the internal consistency
    #[test]
    fn test_traverse_groups_should_fail_for_duplicate_attribute() {
        let metadata_json = json!({
            "vct": "com.example.pid",
            "claims": [
                { "path": ["a", "b", "c", "d", "e"] },
                { "path": ["a", "b", "c"] },
            ],
            "schema": { "properties": {} }
        });
        let type_metadata: TypeMetadata = serde_json::from_value(metadata_json).unwrap();

        let mdoc_attributes = IndexMap::from([
            (
                String::from("com.example.pid.a.b.c.d"),
                vec![Entry {
                    name: String::from("e"),
                    value: DataElementValue::Text(String::from("abcd")),
                }],
            ),
            (
                String::from("com.example.pid.a.b"),
                vec![Entry {
                    name: String::from("c"),
                    value: DataElementValue::Text(String::from("abc")),
                }],
            ),
        ]);

        let result = Attribute::from_mdoc_attributes(&type_metadata, mdoc_attributes);
        assert_matches!(result, Err(AttributeError::DuplicateAttribute(key)) if key == *"c");
    }

    #[test]
    fn test_traverse_groups_for_dot_in_attribute_name() {
        let metadata_json = json!({
            "vct": "com.example.pid",
            "claims": [
                { "path": ["nest.ed", "birth.date"] }
            ],
            "schema": { "properties": {} }
        });
        let type_metadata: TypeMetadata = serde_json::from_value(metadata_json).unwrap();

        let mdoc_attributes = IndexMap::from([(
            String::from("com.example.pid.nest.ed"),
            vec![Entry {
                name: String::from("birth.date"),
                value: DataElementValue::Text(String::from("1963-08-12")),
            }],
        )]);

        let result = Attribute::from_mdoc_attributes(&type_metadata, mdoc_attributes).unwrap();

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
            "claims": [
                { "path": ["a", "a1"] },
                { "path": ["a", "a2"] }
            ],
            "schema": { "properties": {} }
        });
        let type_metadata: TypeMetadata = serde_json::from_value(metadata_json).unwrap();

        let mdoc_attributes = IndexMap::from([(
            String::from("com.example.pid.a"),
            vec![
                Entry {
                    name: String::from("a1"),
                    value: DataElementValue::Text(String::from("1")),
                },
                Entry {
                    name: String::from("a2"),
                    value: DataElementValue::Text(String::from("2")),
                },
                Entry {
                    name: String::from("a3"),
                    value: DataElementValue::Text(String::from("3")),
                },
            ],
        )]);

        let result = Attribute::from_mdoc_attributes(&type_metadata, mdoc_attributes);
        assert_matches!(result, Err(AttributeError::SomeAttributesNotProcessed(attrs))
        if attrs == IndexMap::from([(
            String::from("com.example.pid.a"),
            vec![Entry { name: String::from("a3"), value: DataElementValue::Text(String::from("3")) }]
        )]));
    }

    #[test]
    fn test_traverse_groups_claim_ordering() {
        let metadata_json = json!({
            "vct": "com.example.pid",
            "claims": [
                { "path": ["b", "b1"] },
                { "path": ["b", "b3"] },
                { "path": ["b", "b2"] }
            ],
            "schema": { "properties": {} }
        });
        let type_metadata: TypeMetadata = serde_json::from_value(metadata_json).unwrap();

        let mdoc_attributes = IndexMap::from([(
            String::from("com.example.pid.b"),
            vec![
                Entry {
                    name: String::from("b1"),
                    value: DataElementValue::Text(String::from("1")),
                },
                Entry {
                    name: String::from("b2"),
                    value: DataElementValue::Text(String::from("2")),
                },
                Entry {
                    name: String::from("b3"),
                    value: DataElementValue::Text(String::from("3")),
                },
            ],
        )]);

        let result = Attribute::from_mdoc_attributes(&type_metadata, mdoc_attributes).unwrap();
        let expected_json = json!({"b": { "b1": "1", "b3": "3", "b2": "2" }});
        assert_eq!(
            serde_json::to_value(result).unwrap().to_json_string_pretty().unwrap(),
            expected_json.to_json_string_pretty().unwrap(),
        );
    }
}
