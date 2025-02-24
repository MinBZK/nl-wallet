use std::collections::VecDeque;

use chrono::DateTime;
use chrono::ParseError;
use chrono::Utc;
use indexmap::IndexMap;
use serde::Deserialize;
use serde::Serialize;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use serde_with::TimestampSeconds;

use error_category::ErrorCategory;
use nl_wallet_mdoc::holder::Mdoc;
use nl_wallet_mdoc::unsigned::Entry;
use nl_wallet_mdoc::unsigned::UnsignedMdoc;
use nl_wallet_mdoc::NameSpace;
use sd_jwt::metadata::TypeMetadataChain;
use sd_jwt::metadata::TypeMetadataError;
use wallet_common::urls::HttpsUri;

use crate::attributes::Attribute;
use crate::attributes::AttributeError;

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum CredentialPayloadError {
    #[error("unable to strip namespace '{namespace}' from key '{key}'")]
    #[category(pd)]
    NamespaceStripping { namespace: String, key: String },

    #[error(
        "The namespace is required to consist of nested group names, joined by a '.' and prefixed with the \
         attestation_type. Actual namespace: '{namespace}' and doc_type: '{doc_type}'"
    )]
    #[category(pd)]
    NamespacePreconditionFailed { namespace: String, doc_type: String },

    #[error("error converting to JSON: {0}")]
    #[category(pd)]
    JsonConversion(#[from] serde_json::Error),

    #[error("metadata validation error: {0}")]
    #[category(pd)]
    Metadata(#[from] TypeMetadataError),

    #[error("unable to convert mdoc TDate to DateTime<Utc>")]
    #[category(critical)]
    DateConversion(#[from] ParseError),

    #[error("mdoc error: {0}")]
    #[category(defer)]
    Mdoc(#[from] nl_wallet_mdoc::Error),

    #[error("attribute error: {0}")]
    #[category(pd)]
    Attribute(#[from] AttributeError),

    #[error("attribute with name: {0} already exists")]
    #[category(pd)]
    DuplicateAttribute(String),
}

/// This struct represents the Claims Set received from the issuer. Its JSON representation should be verifiable by the
/// JSON schema defined in the SD-JWT VC Type Metadata (`TypeMetadata`).
///
/// Converting both an (unsigned) mdoc and SD-JWT document to this struct should yield the same result.
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialPayload {
    #[serde(rename = "vct")]
    pub attestation_type: String,

    #[serde(rename = "iss")]
    pub issuer: HttpsUri,

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
    pub fn from_unsigned_mdoc(mdoc: &UnsignedMdoc) -> Result<Self, CredentialPayloadError> {
        Self::from_mdoc_attributes(
            mdoc.doc_type.to_string(),
            mdoc.attributes.as_ref(),
            mdoc.issuer_uri.clone(),
            Some(Utc::now()),
            Some((&mdoc.valid_until).try_into()?),
            Some((&mdoc.valid_from).try_into()?),
        )
    }

    pub fn from_mdoc(mdoc: &Mdoc) -> Result<Self, CredentialPayloadError> {
        Self::from_mdoc_attributes(
            mdoc.doc_type().to_string(),
            &mdoc.attributes(),
            mdoc.issuer_uri()?.clone(),
            Some((&mdoc.validity_info().signed).try_into()?),
            Some((&mdoc.validity_info().valid_until).try_into()?),
            Some((&mdoc.validity_info().valid_from).try_into()?),
        )
    }

    /// Convert a map of namespaced entries (`Entry`) to a `CredentialPayload`. The namespace is required to consist of
    /// nested group names, joined by a '.' and prefixed with the attestation_type.
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
    /// Note in particular that attributes in a namespace whose names equals the `doc_type` parameter are mapped to the
    /// root level of the output.
    pub fn from_mdoc_attributes(
        doc_type: String,
        attributes: &IndexMap<NameSpace, Vec<Entry>>,
        issuer: HttpsUri,
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
                let mut groups: VecDeque<String> = Self::split_namespace(namespace, doc_type)?.into();
                Self::traverse_groups(entries, &mut groups, result)?;
            }
        }

        Ok(())
    }

    pub fn collect_keys(&self) -> Vec<Vec<String>> {
        Self::collect_keys_recursive(&self.attributes, &[])
    }

    fn collect_keys_recursive(attributes: &IndexMap<String, Attribute>, groups: &[String]) -> Vec<Vec<String>> {
        attributes.iter().fold(vec![], |mut acc, (k, v)| {
            let mut keys = Vec::from(groups);
            keys.push(k.clone());

            match v {
                Attribute::Single(_) => acc.push(keys),
                Attribute::Nested(nested) => acc.append(&mut Self::collect_keys_recursive(nested, &keys)),
            }

            acc
        })
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

            if group.contains_key(&key) {
                return Err(CredentialPayloadError::DuplicateAttribute(key.clone()));
            }

            group.insert(key, Attribute::Single(entry.value.clone().try_into()?));
        }

        Ok(())
    }

    fn split_namespace(namespace: &str, doc_type: &str) -> Result<Vec<String>, CredentialPayloadError> {
        if !namespace.starts_with(doc_type) {
            return Err(CredentialPayloadError::NamespacePreconditionFailed {
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

    pub fn validate(&self, metadata_chain: &TypeMetadataChain) -> Result<(), CredentialPayloadError> {
        let metadata = metadata_chain.verify()?;
        metadata.validate(&serde_json::to_value(self)?)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;
    use chrono::TimeZone;
    use chrono::Utc;
    use indexmap::IndexMap;
    use rstest::rstest;
    use serde_json::json;
    use serde_valid::json::ToJsonString;

    use nl_wallet_mdoc::unsigned::Entry;
    use nl_wallet_mdoc::DataElementValue;
    use sd_jwt::metadata::TypeMetadata;
    use sd_jwt::metadata::TypeMetadataChain;

    use crate::attributes::Attribute;
    use crate::attributes::AttributeValue;
    use crate::credential_payload::CredentialPayload;
    use crate::credential_payload::CredentialPayloadError;

    #[rstest]
    #[case(vec![], "com.example.pid", "com.example.pid")]
    #[case(vec!["place_of_birth"], "com.example.pid.place_of_birth", "com.example.pid")]
    #[case(vec!["place_of_birth", "country"], "com.example.pid.place_of_birth.country", "com.example.pid")]
    fn test_split_namespace(#[case] expected: Vec<&str>, #[case] namespace: &str, #[case] doc_type: &str) {
        assert_eq!(
            expected.into_iter().map(String::from).collect::<Vec<_>>(),
            CredentialPayload::split_namespace(namespace, doc_type).unwrap()
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
                    name: String::from("c1"),
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
                    },
                    "c1": "abc",
                }
            }
        });
        assert_eq!(
            expected_json.to_json_string_pretty().unwrap(),
            serde_json::to_value(result).unwrap().to_json_string_pretty().unwrap()
        );
    }

    #[test]
    fn test_traverse_groups_should_fail_for_duplicate_attribute() {
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

        let result = CredentialPayload::traverse_attributes("com.example.pid", &mdoc_attributes, &mut IndexMap::new());
        assert_matches!(result, Err(CredentialPayloadError::DuplicateAttribute(key)) if key == *"c");
    }

    #[test]
    fn test_serialize_deserialize_and_validate() {
        let payload = CredentialPayload {
            attestation_type: String::from("com.example.pid"),
            issuer: "https://com.example.org/pid/issuer".parse().unwrap(),
            issued_at: Some(Utc.with_ymd_and_hms(1970, 1, 1, 0, 1, 1).unwrap()),
            expires: None,
            not_before: None,
            attributes: IndexMap::from([
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
                                (String::from("area_code"), Attribute::Single(AttributeValue::Number(33))),
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
                            Attribute::Single(AttributeValue::Number(-10_000)),
                        ),
                    ])),
                ),
            ]),
        };

        let expected_json = json!({
            "vct": "com.example.pid",
            "iss": "https://com.example.org/pid/issuer",
            "iat": 61,
            "birth_date": "1963-08-12",
            "place_of_birth": {
                "locality": "The Hague",
                "country": {
                    "name": "The Netherlands",
                    "area_code": 33
                }
            },
            "financial": {
                "has_debt": true,
                "has_job": false,
                "debt_amount": -10000
            }
        });

        assert_eq!(
            serde_json::to_value(payload).unwrap().to_json_string_pretty().unwrap(),
            expected_json.to_json_string_pretty().unwrap()
        );

        let payload = serde_json::from_value::<CredentialPayload>(expected_json).unwrap();

        let metadata = TypeMetadata::example();
        let metadata_chain = TypeMetadataChain::create(metadata, vec![]).unwrap();

        payload.validate(&metadata_chain).unwrap();
    }

    #[test]
    fn test_collect_keys() {
        let json = json!({
            "vct": "com.example.pid",
            "iss": "https://com.example.org/pid/issuer",
            "birthdate": "1963-08-12",
            "place_of_birth": {
                "locality": "The Hague",
                "country": {
                    "name": "The Netherlands",
                    "area_code": 33
                }
            }
        });
        let payload: CredentialPayload = serde_json::from_value(json).unwrap();
        assert_eq!(
            vec![
                vec!["birthdate"],
                vec!["place_of_birth", "locality"],
                vec!["place_of_birth", "country", "name"],
                vec!["place_of_birth", "country", "area_code"],
            ],
            payload
                .collect_keys()
                .iter()
                .map(|keys| keys.iter().map(String::as_str).collect::<Vec<_>>())
                .collect::<Vec<_>>()
        );
    }
}
