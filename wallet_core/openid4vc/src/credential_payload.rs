use std::collections::VecDeque;

use chrono::serde::ts_seconds;
use chrono::DateTime;
use chrono::Utc;
use indexmap::IndexMap;
use serde::Deserialize;
use serde::Serialize;

use nl_wallet_mdoc::holder::Mdoc;
use nl_wallet_mdoc::unsigned::Entry;
use nl_wallet_mdoc::unsigned::UnsignedMdoc;
use nl_wallet_mdoc::utils::auth::Organization;
use nl_wallet_mdoc::DataElementValue;
use nl_wallet_mdoc::NameSpace;

#[derive(Debug, thiserror::Error)]
pub enum CredentialPayloadError {
    #[error("error converting Mdoc to CredentialPaylod")]
    MdocConversion,

    #[error("unable to strip namespace '{namespace}' from key '{key}'")]
    NamespaceStripping { namespace: String, key: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttributeValue {
    Text(String),
    Number(i128),
    Bool(bool),
}

impl From<DataElementValue> for AttributeValue {
    fn from(value: DataElementValue) -> Self {
        match value {
            DataElementValue::Text(text) => AttributeValue::Text(text),
            DataElementValue::Bool(bool) => AttributeValue::Bool(bool),
            DataElementValue::Integer(integer) => AttributeValue::Number(integer.into()),
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Attribute {
    Single(AttributeValue),
    Nested(IndexMap<String, Attribute>),
}

/// This struct represents the Claims Set received from the issuer. Its JSON representation should be verifiable by the
/// JSON schema defined in the SD-JWT VC Type Metadata (`TypeMetadata`).
///
/// Converting both an (unsigned) mdoc and SD-JWT document to this struct should yield the same result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialPayload {
    #[serde(rename = "vct")]
    pub attestation_type: String,

    #[serde(rename = "iss")]
    pub issuer: Organization,

    #[serde(rename = "iat", with = "ts_seconds")]
    pub issued_at: DateTime<Utc>,

    #[serde(rename = "exp", with = "ts_seconds")]
    pub expires: DateTime<Utc>,

    #[serde(flatten)]
    pub attributes: IndexMap<String, Attribute>,
}

impl CredentialPayload {
    pub fn from_unsigned_mdoc(mdoc: &UnsignedMdoc, issuer: Organization) -> Result<Self, CredentialPayloadError> {
        Self::from_mdoc_attributes(mdoc.doc_type.to_string(), mdoc.attributes.as_ref(), issuer)
    }

    pub fn from_mdoc(mdoc: &Mdoc, issuer: Organization) -> Result<Self, CredentialPayloadError> {
        Self::from_mdoc_attributes(mdoc.doc_type.to_string(), &mdoc.attributes(), issuer)
    }

    /// Convert a map of namespaced entries (`Entry`) to a `CredentialPayload`. The namespace is assumed to consist of
    /// nested group names, joined by a '.' and prefixed with the attestation_type.
    ///
    /// The JSON representation of the input and output of this function is as follows:
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
    pub fn from_mdoc_attributes(
        doc_type: String,
        attributes: &IndexMap<NameSpace, Vec<Entry>>,
        issuer: Organization,
    ) -> Result<Self, CredentialPayloadError> {
        let mut attrs = IndexMap::new();
        Self::traverse_attributes(&doc_type, attributes, &mut attrs)?;

        let payload = Self {
            attestation_type: doc_type,
            issuer,
            issued_at: Utc::now(),
            expires: Utc::now(),
            attributes: attrs,
        };

        Ok(payload)
    }

    fn traverse_attributes(
        doc_type: &str,
        attributes: &IndexMap<String, Vec<Entry>>,
        result: &mut IndexMap<String, Attribute>,
    ) -> Result<(), CredentialPayloadError> {
        for (namespace, entries) in attributes {
            if namespace == doc_type {
                Self::insert_entries(entries, result);
            } else {
                let mut groups: VecDeque<String> = Self::split_namespace(namespace, doc_type).into();
                Self::traverse_groups(entries, &mut groups, result)?;
            }
        }

        Ok(())
    }

    fn traverse_groups(
        entries: &[Entry],
        groups: &mut VecDeque<String>,
        current_group: &mut IndexMap<String, Attribute>,
    ) -> Result<(), CredentialPayloadError> {
        if let Some(group_key) = groups.pop_front() {
            // If the group doesn't exist, add a new group to the current group.
            if !current_group.contains_key(&group_key) {
                current_group.insert(String::from(&group_key), Attribute::Nested(IndexMap::new()));
            }

            if let Some(Attribute::Nested(attr_group)) = current_group.get_mut(&group_key) {
                if groups.is_empty() {
                    Self::insert_entries(entries, attr_group);
                } else {
                    Self::traverse_groups(entries, groups, attr_group)?;
                }
            }
        }

        Ok(())
    }

    fn insert_entries(entries: &[Entry], group: &mut IndexMap<String, Attribute>) {
        for entry in entries.iter() {
            let key = entry.name.to_string();
            // Check if there already is an existing group to which the new attributes have to be added. Otherwise,
            // add them to the current group.
            if let Some(Attribute::Nested(existing_group)) = group.get_mut(&key) {
                existing_group.insert(key, Attribute::Single(entry.value.clone().into()));
            } else {
                group.insert(key, Attribute::Single(entry.value.clone().into()));
            }
        }
    }

    fn split_namespace(namespace: &str, doc_type: &str) -> Vec<String> {
        if namespace.len() == doc_type.len() {
            return vec![];
        }

        namespace[doc_type.len() + 1..].split('.').map(String::from).collect()
    }
}

#[cfg(test)]
mod test {
    use indexmap::IndexMap;
    use nl_wallet_mdoc::unsigned::Entry;
    use nl_wallet_mdoc::DataElementValue;
    use rstest::rstest;
    use serde_json::json;

    use crate::credential_payload::CredentialPayload;

    #[rstest]
    #[case(vec![], "com.example.pid")]
    #[case(vec!["place_of_birth"], "com.example.pid.place_of_birth")]
    #[case(vec!["place_of_birth", "country"], "com.example.pid.place_of_birth.country")]
    fn test_split_namespace(#[case] expected: Vec<&str>, #[case] namespace: &str) {
        assert_eq!(
            expected.into_iter().map(String::from).collect::<Vec<_>>(),
            CredentialPayload::split_namespace(namespace, "com.example.pid")
        );
    }

    #[test]
    fn test_traverse_groups() {
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
                    name: String::from("c"),
                    value: DataElementValue::Text(String::from("abc")),
                }],
            ),
        ]);
        let result = &mut IndexMap::new();
        CredentialPayload::traverse_attributes("com.example.pid", &mdoc_attributes, result).unwrap();

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
                        "c": "abc"
                    }
                }
            }
        });
        assert_eq!(expected_json, serde_json::to_value(result).unwrap());
    }
}
