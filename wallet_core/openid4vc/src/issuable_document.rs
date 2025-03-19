use std::num::NonZeroU8;

use indexmap::IndexMap;
use mdoc::AttestationQualification;
use serde::Deserialize;
use serde::Serialize;
use serde_valid::Validate;

use mdoc::unsigned::Entry;
use mdoc::unsigned::UnsignedAttributesError;
use mdoc::unsigned::UnsignedMdoc;
use mdoc::Tdate;
use wallet_common::urls::HttpsUri;
use wallet_common::vec_at_least::VecNonEmpty;

use crate::attributes::Attribute;

/// Generic data model used to pass the attributes to be issued from the issuer backend to the wallet server. This model
/// should be convertable into all documents that are actually issued to the wallet. For now, this will only be
/// `UnsignedMdoc`.
/// ```json
/// {
///     "attestation_type": "com.example.pid",
///     "attributes": {
///         "name": "John",
///         "lastname": "Doe"
///     }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[validate(custom = IssuableDocument::validate)]
pub struct IssuableDocument {
    attestation_type: String,
    attributes: IndexMap<String, Attribute>,
}

impl IssuableDocument {
    pub fn try_new(
        attestation_type: String,
        attributes: IndexMap<String, Attribute>,
    ) -> Result<Self, serde_valid::validation::Error> {
        let document = Self {
            attestation_type,
            attributes,
        };
        document.validate()?;
        Ok(document)
    }

    fn validate(&self) -> Result<(), serde_valid::validation::Error> {
        self.attributes
            .len()
            .ge(&1)
            .then_some(())
            .ok_or_else(|| serde_valid::validation::Error::Custom("must contain at least one attribute".to_string()))?;

        Ok(())
    }

    fn walk_attributes_recursive(
        namespace: String,
        attributes: &IndexMap<String, Attribute>,
        result: &mut IndexMap<String, Vec<Entry>>,
    ) {
        let mut entries = vec![];
        for (key, value) in attributes {
            match value {
                Attribute::Single(single) => {
                    entries.push(Entry {
                        name: key.to_owned(),
                        value: single.into(),
                    });
                }
                Attribute::Nested(nested) => {
                    let key = format!("{}.{}", namespace, key);
                    Self::walk_attributes_recursive(key, nested, result);
                }
            }
        }

        result.insert(namespace, entries);
    }

    /// Convert an issuable document into an `UnsignedMdoc`. This is done by walking down the tree of attributes and
    /// using their keys as namespaces. For example, this issuable document:
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
    /// Turns into an `UnsignedMdoc` with the following structure:
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
    pub fn to_unsigned_mdoc(
        &self,
        valid_from: Tdate,
        valid_until: Tdate,
        copy_count: NonZeroU8,
        issuer_uri: HttpsUri,
        attestation_qualification: AttestationQualification,
    ) -> Result<UnsignedMdoc, UnsignedAttributesError> {
        let mut flattened = IndexMap::new();
        Self::walk_attributes_recursive(self.attestation_type.clone(), &self.attributes, &mut flattened);

        Ok(UnsignedMdoc {
            doc_type: self.attestation_type.clone(),
            attributes: flattened.try_into()?,
            valid_from,
            valid_until,
            copy_count,
            issuer_uri,
            attestation_qualification,
        })
    }

    pub fn attestation_type(&self) -> &str {
        &self.attestation_type
    }
}

pub type IssuableDocuments = VecNonEmpty<IssuableDocument>;

#[cfg(test)]
mod test {
    use std::ops::Add;

    use chrono::Days;
    use chrono::Utc;
    use serde_json::json;

    use mdoc::unsigned::Entry;
    use mdoc::DataElementValue;
    use mdoc::NameSpace;

    use crate::attributes::AttributeValue;

    use super::*;

    fn readable_attrs(attrs: &IndexMap<NameSpace, Vec<Entry>>) -> IndexMap<String, IndexMap<String, DataElementValue>> {
        attrs
            .iter()
            .map(|(ns, entries)| {
                (
                    ns.to_string(),
                    entries
                        .iter()
                        .map(|entry| (entry.name.to_string(), entry.value.clone()))
                        .collect(),
                )
            })
            .collect()
    }

    fn issuable_attrs_to_unsigned_mdocs(
        issuable: &IssuableDocuments,
    ) -> Result<Vec<UnsignedMdoc>, UnsignedAttributesError> {
        issuable
            .as_ref()
            .iter()
            .map(|doc| {
                doc.to_unsigned_mdoc(
                    Tdate::now(),
                    Utc::now().add(Days::new(1)).into(),
                    NonZeroU8::new(1).unwrap(),
                    "https://pid.example.com".parse().unwrap(),
                    Default::default(),
                )
            })
            .collect::<Result<Vec<_>, _>>()
    }

    fn setup_issuable_attributes() -> IssuableDocuments {
        vec![IssuableDocument {
            attestation_type: "com.example.address".to_string(),
            attributes: IndexMap::from_iter(vec![
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
            ]),
        }]
        .try_into()
        .unwrap()
    }

    #[test]
    fn test_serialize_attributes() {
        let attributes = setup_issuable_attributes();
        assert_eq!(
            serde_json::to_value(attributes).unwrap(),
            json!([{
                "attestation_type": "com.example.address",
                "attributes": {
                    "city": "The Capital",
                    "street": "Main St.",
                    "house": {
                        "number": 1,
                        "letter": "A",
                    },
                },
            }])
        );
    }

    #[test]
    fn test_issuable_attributes_to_unsigned_mdoc() {
        let attributes = setup_issuable_attributes();
        let unsigned_mdoc = issuable_attrs_to_unsigned_mdocs(&attributes).unwrap().remove(0);

        assert_eq!(unsigned_mdoc.issuer_uri.to_string(), "https://pid.example.com/");
        assert_eq!(unsigned_mdoc.doc_type, "com.example.address");
        assert_eq!(
            serde_json::to_value(readable_attrs(unsigned_mdoc.attributes.as_ref())).unwrap(),
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
}
