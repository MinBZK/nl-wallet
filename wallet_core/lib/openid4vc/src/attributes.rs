use std::collections::VecDeque;
use std::num::TryFromIntError;

use indexmap::IndexMap;
use serde::Deserialize;
use serde::Serialize;

use mdoc::unsigned::Entry;
use mdoc::DataElementValue;
use mdoc::NameSpace;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;

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
    #[error("unable to convert mdoc value: {0:?}")]
    FromCborConversion(DataElementValue),

    #[error("unable to convert number to cbor: {0}")]
    NumberToCborConversion(#[from] TryFromIntError),

    #[error("some attributes have not been processed by metadata: {0:?}")]
    SomeAttributesNotProcessed(IndexMap<String, Vec<Entry>>),
}

impl From<&AttributeValue> for ciborium::Value {
    fn from(value: &AttributeValue) -> Self {
        match value {
            AttributeValue::Integer(number) => ciborium::Value::Integer((*number).into()),
            AttributeValue::Bool(boolean) => ciborium::Value::Bool(*boolean),
            AttributeValue::Text(text) => ciborium::Value::Text(text.to_owned()),
            AttributeValue::Array(elements) => ciborium::Value::Array(elements.iter().map(Into::into).collect()),
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
        type_metadata: &NormalizedTypeMetadata,
        mut attributes: IndexMap<NameSpace, Vec<Entry>>,
    ) -> Result<IndexMap<String, Self>, AttributeError> {
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
}

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;
    use indexmap::IndexMap;
    use serde_json::json;
    use serde_valid::json::ToJsonString;

    use mdoc::unsigned::Entry;
    use mdoc::DataElementValue;
    use sd_jwt_vc_metadata::NormalizedTypeMetadata;

    use crate::attributes::Attribute;
    use crate::attributes::AttributeError;

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
