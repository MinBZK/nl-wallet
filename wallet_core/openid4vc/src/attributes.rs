use std::collections::VecDeque;
use std::num::TryFromIntError;

use indexmap::IndexMap;
use serde::Deserialize;
use serde::Serialize;

use nl_wallet_mdoc::unsigned::Entry;
use nl_wallet_mdoc::unsigned::UnsignedAttributesError;
use nl_wallet_mdoc::DataElementValue;
use nl_wallet_mdoc::NameSpace;
use sd_jwt::metadata::TypeMetadata;
use sd_jwt::metadata::TypeMetadataError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttributeValue {
    Number(i64),
    Bool(bool),
    Text(String),
}

#[derive(Debug, thiserror::Error)]
pub enum AttributeError {
    #[error("unable to convert mdoc value: {0:?}")]
    FromCborConversion(DataElementValue),

    #[error("unable to convert number to cbor: {0}")]
    NumberToCborConversion(#[from] TryFromIntError),

    #[error("unable to instantiate UnsignedAttributes: {0}")]
    UnsignedAttributes(#[from] UnsignedAttributesError),

    #[error(
        "The namespace is required to consist of nested group names, joined by a '.' and prefixed with the \
         attestation_type. Actual namespace: '{namespace}' and doc_type: '{doc_type}'"
    )]
    NamespacePreconditionFailed { namespace: String, doc_type: String },

    #[error("attribute with name: {0} already exists")]
    DuplicateAttribute(String),

    #[error("no JSON Schema found in metadata: {0}")]
    MetadataSchemaNotFound(#[from] TypeMetadataError),

    #[error("no metadata found for attribute: {0} in JSON Schema")]
    MetadataNotFoundForAttributeKey(String),
}

impl From<&AttributeValue> for ciborium::Value {
    fn from(value: &AttributeValue) -> Self {
        match value {
            AttributeValue::Number(number) => ciborium::Value::Integer((*number).into()),
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
            DataElementValue::Integer(integer) => Ok(AttributeValue::Number(integer.try_into()?)),
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
    pub fn from_mdoc_attributes(
        type_metadata: &TypeMetadata,
        attributes: IndexMap<NameSpace, Vec<Entry>>,
    ) -> Result<IndexMap<String, Self>, AttributeError> {
        let mut attrs = IndexMap::new();
        Self::traverse_attributes(type_metadata, attributes, &mut attrs)?;
        Ok(attrs)
    }

    fn traverse_attributes(
        type_metadata: &TypeMetadata,
        attributes: IndexMap<String, Vec<Entry>>,
        result: &mut IndexMap<String, Attribute>,
    ) -> Result<(), AttributeError> {
        for (namespace, entries) in attributes {
            if namespace == type_metadata.vct {
                Self::insert_entries(entries, result)?;
            } else {
                let mut groups: VecDeque<String> = Self::split_namespace(&namespace, &type_metadata.vct)?.into();
                Self::traverse_groups(entries, &mut groups, result)?;
            }
        }

        Ok(())
    }

    fn traverse_groups(
        entries: Vec<Entry>,
        groups: &mut VecDeque<String>,
        current_group: &mut IndexMap<String, Attribute>,
    ) -> Result<(), AttributeError> {
        if let Some(group_key) = groups.pop_front() {
            // If the group doesn't exist, add a new group to the current group.
            if !current_group.contains_key(&group_key) {
                current_group.insert(String::from(&group_key), Attribute::Nested(IndexMap::new()));
            }

            if let Some(Attribute::Nested(attr_group)) = current_group.get_mut(&group_key) {
                if groups.is_empty() {
                    Self::insert_entries(entries, attr_group)?;
                } else {
                    Self::traverse_groups(entries, groups, attr_group)?;
                }
            }
        }

        Ok(())
    }

    fn insert_entries(entries: Vec<Entry>, group: &mut IndexMap<String, Attribute>) -> Result<(), AttributeError> {
        for entry in entries {
            let key = entry.name;

            if group.contains_key(&key) {
                return Err(AttributeError::DuplicateAttribute(key));
            }

            group.insert(key, Attribute::Single(entry.value.try_into()?));
        }

        Ok(())
    }

    fn split_namespace(namespace: &str, doc_type: &str) -> Result<Vec<String>, AttributeError> {
        if !namespace.starts_with(doc_type) {
            return Err(AttributeError::NamespacePreconditionFailed {
                namespace: String::from(namespace),
                doc_type: String::from(doc_type),
            });
        }

        if namespace.len() == doc_type.len() {
            return Ok(vec![]);
        }

        let parts = namespace[doc_type.len() + 1..].split('.').map(String::from).collect();
        Ok(parts)
    }
}

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;
    use indexmap::IndexMap;
    use rstest::rstest;
    use serde_json::json;
    use serde_valid::json::ToJsonString;

    use nl_wallet_mdoc::unsigned::Entry;
    use nl_wallet_mdoc::DataElementValue;
    use sd_jwt::metadata::TypeMetadata;

    use crate::attributes::Attribute;
    use crate::attributes::AttributeError;

    #[rstest]
    #[case(vec![], "com.example.pid", "com.example.pid")]
    #[case(vec!["place_of_birth"], "com.example.pid.place_of_birth", "com.example.pid")]
    #[case(vec!["place_of_birth", "country"], "com.example.pid.place_of_birth.country", "com.example.pid")]
    fn test_split_namespace(#[case] expected: Vec<&str>, #[case] namespace: &str, #[case] doc_type: &str) {
        assert_eq!(
            expected.into_iter().map(String::from).collect::<Vec<_>>(),
            Attribute::split_namespace(namespace, doc_type).unwrap()
        );
    }

    #[test]
    fn test_traverse_groups() {
        let metadata_json = json!({
            "vct": "com.example.pid",
            "schema": {
                "type": "object",
                "properties": {
                    "birthdate": {
                        "type": "string"
                    },
                    "place_of_birth": {
                        "type": "object",
                        "properties": {
                            "locality": {
                                "type": "string"
                            },
                            "country": {
                                "type": "object",
                                "properties": {
                                    "name": {
                                        "type": "string"
                                    },
                                    "area_code": {
                                        "type": "number"
                                    },
                                }
                            }
                        }
                    },
                    "a": {
                        "type": "object",
                        "properties": {
                            "b": {
                                "type": "object",
                                "properties": {
                                    "c": {
                                        "type": "object",
                                        "properties": {
                                            "d": {
                                                "type": "object",
                                                "properties": {
                                                    "e": {
                                                        "type": "string"
                                                    }
                                                }
                                            }
                                        }
                                    },
                                    "c1": {
                                        "type": "string"
                                    }
                                }
                            }
                        }
                    }
                }
            }
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
        let result = &mut IndexMap::new();
        Attribute::traverse_attributes(&type_metadata, mdoc_attributes, result).unwrap();

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
    fn test_traverse_groups_should_fail_for_duplicate_attribute() {
        let metadata_json = json!({
            "vct": "com.example.pid",
            "schema": {
                "type": "object",
                "properties": {
                    "birthdate": {
                        "type": "string"
                    },
                    "place_of_birth": {
                        "type": "object",
                        "properties": {
                            "locality": {
                                "type": "string"
                            },
                            "country": {
                                "type": "object",
                                "properties": {
                                    "name": {
                                        "type": "string"
                                    },
                                    "area_code": {
                                        "type": "number"
                                    },
                                }
                            }
                        }
                    },
                    "a": {
                        "type": "object",
                        "properties": {
                            "b": {
                                "type": "object",
                                "properties": {
                                    "c": {
                                        "type": "object",
                                        "properties": {
                                            "d": {
                                                "type": "object",
                                                "properties": {
                                                    "e": {
                                                        "type": "string"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
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

        let result = Attribute::traverse_attributes(&type_metadata, mdoc_attributes, &mut IndexMap::new());
        assert_matches!(result, Err(AttributeError::DuplicateAttribute(key)) if key == *"c");
    }
}
