use std::collections::VecDeque;

use chrono::DateTime;
use chrono::ParseError;
use chrono::Utc;
use http::Uri;
use indexmap::IndexMap;
use serde::Deserialize;
use serde::Serialize;
use serde_with::serde_as;
use serde_with::TimestampSeconds;

use error_category::ErrorCategory;
use nl_wallet_mdoc::holder::Mdoc;
use nl_wallet_mdoc::unsigned::Entry;
use nl_wallet_mdoc::unsigned::UnsignedMdoc;
use nl_wallet_mdoc::DataElementValue;
use nl_wallet_mdoc::NameSpace;

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum CredentialPayloadError {
    #[error("unable to strip namespace '{namespace}' from key '{key}'")]
    #[category(pd)]
    NamespaceStripping { namespace: String, key: String },

    #[error("unable to convert mdoc TDate to DateTime<Utc>")]
    #[category(critical)]
    DateConversion(#[from] ParseError),

    #[error("unable to convert mdoc value to AttributeValue: {0:?}")]
    #[category(pd)]
    ValueConversion(DataElementValue),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttributeValue {
    Text(String),
    Number(i128),
    Bool(bool),
}

impl TryFrom<DataElementValue> for AttributeValue {
    type Error = CredentialPayloadError;

    fn try_from(value: DataElementValue) -> Result<Self, Self::Error> {
        match value {
            DataElementValue::Text(text) => Ok(AttributeValue::Text(text)),
            DataElementValue::Bool(bool) => Ok(AttributeValue::Bool(bool)),
            DataElementValue::Integer(integer) => Ok(AttributeValue::Number(integer.into())),
            _ => Err(CredentialPayloadError::ValueConversion(value)),
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
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialPayload {
    #[serde(rename = "vct")]
    pub attestation_type: String,

    #[serde(rename = "iss", with = "http_serde::uri")]
    pub issuer: Uri,

    #[serde(rename = "iat")]
    #[serde_as(as = "Option<TimestampSeconds<i64>>")]
    pub issued_at: Option<DateTime<Utc>>,

    #[serde(rename = "exp")]
    #[serde_as(as = "Option<TimestampSeconds<i64>>")]
    pub expires: Option<DateTime<Utc>>,

    #[serde(rename = "nbf")]
    #[serde_as(as = "Option<TimestampSeconds<i64>>")]
    pub not_before: Option<DateTime<Utc>>,

    #[serde(flatten)]
    pub attributes: IndexMap<String, Attribute>,
}

impl CredentialPayload {
    pub fn from_unsigned_mdoc(mdoc: &UnsignedMdoc, issuer: Uri) -> Result<Self, CredentialPayloadError> {
        Self::from_mdoc_attributes(
            mdoc.doc_type.to_string(),
            mdoc.attributes.as_ref(),
            issuer,
            None,
            Some((&mdoc.valid_until).try_into()?),
            Some((&mdoc.valid_from).try_into()?),
        )
    }

    pub fn from_mdoc(mdoc: &Mdoc, issuer: Uri) -> Result<Self, CredentialPayloadError> {
        Self::from_mdoc_attributes(
            mdoc.doc_type().to_string(),
            &mdoc.attributes(),
            issuer,
            Some((&mdoc.validity_info().signed).try_into()?),
            Some((&mdoc.validity_info().valid_until).try_into()?),
            Some((&mdoc.validity_info().valid_from).try_into()?),
        )
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
        issuer: Uri,
        issued_at: Option<DateTime<Utc>>,
        expires: Option<DateTime<Utc>>,
        not_before: Option<DateTime<Utc>>,
    ) -> Result<Self, CredentialPayloadError> {
        let mut attrs = IndexMap::new();
        Self::traverse_attributes(&doc_type, attributes, &mut attrs)?;

        let payload = Self {
            attestation_type: doc_type,
            issuer,
            issued_at,
            expires,
            not_before,
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
                Self::insert_entries(entries, result)?;
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
                    Self::insert_entries(entries, attr_group)?;
                } else {
                    Self::traverse_groups(entries, groups, attr_group)?;
                }
            }
        }

        Ok(())
    }

    fn insert_entries(
        entries: &[Entry],
        group: &mut IndexMap<String, Attribute>,
    ) -> Result<(), CredentialPayloadError> {
        for entry in entries.iter() {
            let key = entry.name.to_string();
            // Check if there already is an existing group to which the new attributes have to be added. Otherwise,
            // add them to the current group.
            if let Some(Attribute::Nested(existing_group)) = group.get_mut(&key) {
                existing_group.insert(key, Attribute::Single(entry.value.clone().try_into()?));
            } else {
                group.insert(key, Attribute::Single(entry.value.clone().try_into()?));
            }
        }

        Ok(())
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
    #[case(vec![], "com.example.pid", "com.example.pid")]
    #[case(vec!["place_of_birth"], "com.example.pid.place_of_birth", "com.example.pid")]
    #[case(vec!["place_of_birth", "country"], "com.example.pid.place_of_birth.country", "com.example.pid")]
    fn test_split_namespace(#[case] expected: Vec<&str>, #[case] namespace: &str, #[case] doc_type: &str) {
        assert_eq!(
            expected.into_iter().map(String::from).collect::<Vec<_>>(),
            CredentialPayload::split_namespace(namespace, doc_type)
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
