use chrono::NaiveDate;
use ciborium::value::Integer;
use indexmap::IndexMap;
use itertools::Itertools;

use error_category::ErrorCategory;
use nl_wallet_mdoc::{
    holder::{ProposedAttributes, ProposedDocumentAttributes},
    identifiers::AttributeIdentifier,
    unsigned::{Entry, UnsignedMdoc},
    utils::{
        issuer_auth::IssuerRegistration,
        x509::{CertificateError, MdocCertificateExtension},
    },
    DataElementIdentifier, DataElementValue, NameSpace,
};

use super::{
    mapping::{AttributeMapping, DataElementValueMapping, MappingDocType, MDOC_DOCUMENT_MAPPING},
    Attribute, AttributeValue, DisclosureDocument, Document, DocumentAttributes, DocumentPersistence,
    GenderAttributeValue, MissingDisclosureAttributes, PID_DOCTYPE,
};

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(pd)]
pub enum DocumentMdocError {
    #[error("unknown doc type \"{doc_type}\"")]
    #[category(critical)]
    UnknownDocType { doc_type: String },
    #[error("mandatory attributes for \"{doc_type}\" not found at \"{name_space} / {name}\"")]
    #[category(critical)]
    MissingAttribute {
        doc_type: String,
        name_space: NameSpace,
        name: DataElementIdentifier,
    },
    #[error(
        "attribute for \"{doc_type}\" encountered at \"{name_space} / {name}\" does not match expected type \
         {expected_type:?}: {value:?}"
    )]
    AttributeValueTypeMismatch {
        doc_type: String,
        name_space: NameSpace,
        name: DataElementIdentifier,
        expected_type: AttributeValueType,
        value: DataElementValue,
    },
    #[error("unknown attribute for \"{doc_type}\" encounted at \"{name_space} / {name}\": {value:?}")]
    UnknownAttribute {
        doc_type: String,
        name_space: NameSpace,
        name: DataElementIdentifier,
        value: Option<DataElementValue>,
    },
    #[error("certificate error for \"{doc_type}\": {error}")]
    #[category(defer)]
    Certificate {
        #[defer]
        error: CertificateError,
        doc_type: String,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum AttributeValueType {
    String,
    Bool,
    Date,
    Gender,
}

// TODO: Think about refactoring/renaming DisclosureType. We currently have
// DisclosureType here, *and* in disclosure_history_event.rs, EventType, *and*
// in flutter_api's disclosure.rs again as DisclosureType. Things to think about
// when refactoring: why this many and not just one.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisclosureType {
    Login,
    Regular,
}

impl DisclosureType {
    /// Something is a login flow if the `proposed_attributes` map has exactly one element with a
    /// `doc_type` of `PID_DOCTYPE`, with a `doc_attributes` map where `namespace` is `PID_DOCTYPE`
    /// also, with an entry vec of exactly one entry, where the `DataElementIdentifier` string is "bsn".
    pub fn from_proposed_attributes(proposed_attributes: &ProposedAttributes) -> Self {
        if let Ok((doc_type, doc_attributes)) = proposed_attributes.iter().exactly_one() {
            if doc_type == PID_DOCTYPE {
                if let Ok((namespace, entries)) = doc_attributes.attributes.iter().exactly_one() {
                    if namespace == PID_DOCTYPE {
                        if let Ok(entry) = entries.iter().exactly_one() {
                            if entry.name == "bsn" {
                                return Self::Login;
                            }
                        }
                    }
                }
            }
        }
        Self::Regular
    }

    pub fn is_login_flow(&self) -> bool {
        match self {
            Self::Login => true,
            Self::Regular => false,
        }
    }
}

/// Get the correct `AttributeMapping` or return an error if it cannot be found for the `doc_type`.
fn mapping_for_doc_type(doc_type: &str) -> Result<(MappingDocType, &'static AttributeMapping), DocumentMdocError> {
    let (doc_type, attribute_mapping) =
        MDOC_DOCUMENT_MAPPING
            .get_key_value(doc_type)
            .ok_or_else(|| DocumentMdocError::UnknownDocType {
                doc_type: doc_type.to_string(),
            })?;

    Ok((*doc_type, attribute_mapping))
}

fn document_attributes_from_mdoc_attributes(
    doc_type: &str,
    mut attributes: IndexMap<NameSpace, Vec<Entry>>,
    error_on_missing: bool,
) -> Result<(MappingDocType, DocumentAttributes), DocumentMdocError> {
    let (doc_type, attribute_mapping) = mapping_for_doc_type(doc_type)?;

    // Loop through the attributes in the mapping in order and find
    // the corresponding entry in the input attributes, based on the
    // name space and the entry name. If found, move the entry value
    // out of the input attributes and try to convert it to an `Attribute`.
    let document_attributes = attribute_mapping
        .iter()
        // Loop through the all the mapped attributes in order and remove any
        // returned instances of `None` for non-mandatory attributes.
        .flat_map(|((name_space, element_id), value_mapping)| {
            // Get a mutable reference to the `Vec<Entry>` for the name space,
            // then find the index within the vector for the entry that has the
            // matching name. If found, remove the `Entry` at that index so that
            // we have ownership over it.
            let entry = attributes.get_mut(*name_space).and_then(|entries| {
                entries
                    .iter()
                    .position(|entry| entry.name == *element_id)
                    .map(|index| entries.swap_remove(index))
            });

            // If the entry is not found in the mdoc attributes, but it is not
            // mandatory or we are not processing mandatory attributes, we can
            // return `None` early here.
            if entry.is_none() && (!value_mapping.is_mandatory || !error_on_missing) {
                return None;
            }

            // Otherwise, create the `Result<>` and return an error if the entry
            // is not found.
            let attribute_result = entry
                .ok_or_else(|| DocumentMdocError::MissingAttribute {
                    doc_type: doc_type.to_string(),
                    name_space: (*name_space).to_string(),
                    name: (*element_id).to_string(),
                })
                .and_then(|entry| {
                    // If the entry is found, try to to convert it to a document
                    // attribute, which could also result in an error.
                    let Entry { name, value } = entry;

                    Attribute::try_from((value, value_mapping)).map_err(|value| {
                        DocumentMdocError::AttributeValueTypeMismatch {
                            doc_type: doc_type.to_string(),
                            name_space: (*name_space).to_string(),
                            name,
                            expected_type: value_mapping.value_type,
                            value,
                        }
                    })
                })
                // Finally, make sure the attribute is returned with the key,
                // so that we can create an `IndexMap<>` for it.
                .map(|attribute| (value_mapping.key, attribute));

            Some(attribute_result)
        })
        .collect::<Result<_, _>>()?;

    // Find the first remaining mdoc attributes and convert it to an error.
    let unknown_error = attributes
        .into_iter()
        .flat_map(|(name_space, mut entries)| {
            entries.pop().map(|entry| DocumentMdocError::UnknownAttribute {
                doc_type: doc_type.to_string(),
                name_space,
                name: entry.name,
                value: entry.value.into(),
            })
        })
        .next();

    // Return the error if at least one mdoc attributes still remained.
    if let Some(missing_error) = unknown_error {
        return Err(missing_error);
    }

    Ok((doc_type, document_attributes))
}

impl Document {
    pub(crate) fn from_mdoc_attributes(
        persistence: DocumentPersistence,
        doc_type: &str,
        attributes: IndexMap<NameSpace, Vec<Entry>>,
        issuer_registration: IssuerRegistration,
    ) -> Result<Self, DocumentMdocError> {
        let (doc_type, document_attributes) = document_attributes_from_mdoc_attributes(doc_type, attributes, true)?;

        let document = Document {
            persistence,
            doc_type,
            attributes: document_attributes,
            issuer_registration,
        };

        Ok(document)
    }

    pub(crate) fn from_unsigned_mdoc(
        mdoc: UnsignedMdoc,
        issuer_registration: IssuerRegistration,
    ) -> Result<Self, DocumentMdocError> {
        Document::from_mdoc_attributes(
            DocumentPersistence::InMemory,
            &mdoc.doc_type,
            mdoc.attributes.into_inner(),
            issuer_registration,
        )
    }
}

impl TryFrom<(DataElementValue, &DataElementValueMapping)> for Attribute {
    type Error = DataElementValue;

    fn try_from((value, value_mapping): (DataElementValue, &DataElementValueMapping)) -> Result<Self, Self::Error> {
        let value = (value_mapping.value_type, value).try_into()?;

        let attribute = Attribute {
            key_labels: value_mapping.key_labels.clone(),
            value,
        };

        Ok(attribute)
    }
}

impl TryFrom<(AttributeValueType, DataElementValue)> for AttributeValue {
    type Error = DataElementValue;

    fn try_from(value: (AttributeValueType, DataElementValue)) -> Result<Self, Self::Error> {
        match value {
            (AttributeValueType::String, DataElementValue::Text(s)) => Ok(Self::String(s)),
            (AttributeValueType::Bool, DataElementValue::Bool(b)) => Ok(Self::Boolean(b)),
            (AttributeValueType::Date, DataElementValue::Text(ref s)) => {
                let date = NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| value.1)?;

                Ok(Self::Date(date))
            }
            (AttributeValueType::Gender, DataElementValue::Integer(i)) => {
                let gender = GenderAttributeValue::try_from(i).map_err(|_| value.1)?;

                Ok(Self::Gender(gender))
            }
            _ => Err(value.1),
        }
    }
}

impl TryFrom<Integer> for GenderAttributeValue {
    type Error = ();

    fn try_from(value: Integer) -> Result<Self, Self::Error> {
        match value.into() {
            0 => Ok(Self::Unknown),
            1 => Ok(Self::Male),
            2 => Ok(Self::Female),
            9 => Ok(Self::NotApplicable),
            _ => Err(()),
        }
    }
}

impl MissingDisclosureAttributes {
    // Use the Mdoc document mapping to translate a `Vec<AttributeIdentifier>` to
    // a `Vec<MissingDisclosureAttributes>`. If any attribute cannot be found a
    // `DocumentMdocError` is returned.
    pub(crate) fn from_mdoc_missing_attributes(
        missing_attributes: Vec<AttributeIdentifier>,
    ) -> Result<Vec<Self>, DocumentMdocError> {
        // Create an `IndexMap` that contains `IndexMap`s of attributes per doc type.
        let attributes_by_doc_type =
            missing_attributes
                .into_iter()
                .try_fold(IndexMap::<_, IndexMap<_, _>>::new(), {
                    |mut attributes_by_doc_type, missing_attribute| {
                        let (doc_type, attribute_mapping) =
                            mapping_for_doc_type(missing_attribute.credential_type.as_str())?;
                        let value_mapping = attribute_mapping
                            .get(&(
                                missing_attribute.namespace.as_str(),
                                missing_attribute.attribute.as_str(),
                            ))
                            .ok_or_else(|| DocumentMdocError::UnknownAttribute {
                                doc_type: missing_attribute.credential_type,
                                name_space: missing_attribute.namespace.clone(),
                                name: missing_attribute.attribute.clone(),
                                value: None,
                            })?;

                        attributes_by_doc_type
                            .entry(doc_type)
                            .or_default()
                            .insert(value_mapping.key, value_mapping.key_labels.clone());

                        Ok(attributes_by_doc_type)
                    }
                })?;

        // Convert these `IndexMap`s to a `Vec<MissingDisclosureAttributes>`.
        let mut missing_disclosure_attributes = attributes_by_doc_type
            .into_iter()
            .map(|(doc_type, attributes)| MissingDisclosureAttributes { doc_type, attributes })
            .collect::<Vec<_>>();

        // Make sure that the resulting doc types are sorted canonically.
        missing_disclosure_attributes.sort_by_key(|attributes| super::doc_type_priority(attributes.doc_type));

        Ok(missing_disclosure_attributes)
    }
}

impl DisclosureDocument {
    pub(crate) fn from_mdoc_attributes(
        doc_type: &str,
        attributes: ProposedDocumentAttributes,
    ) -> Result<Self, DocumentMdocError> {
        let issuer_registration = IssuerRegistration::from_certificate(&attributes.issuer)
            .map_err(|error| DocumentMdocError::Certificate {
                doc_type: doc_type.to_owned(),
                error,
            })?
            .expect("IssuerRegistration must exist after successful issuance");
        let (doc_type, document_attributes) =
            document_attributes_from_mdoc_attributes(doc_type, attributes.attributes, false)?;

        let document = DisclosureDocument {
            issuer_registration,
            doc_type,
            attributes: document_attributes,
        };

        Ok(document)
    }
}

#[cfg(test)]
pub mod tests {
    use std::{collections::HashMap, mem, num::NonZeroU8, sync::LazyLock};

    use assert_matches::assert_matches;
    use chrono::{Days, Utc};
    use rstest::rstest;

    use nl_wallet_mdoc::{server_keys::KeyPair, Tdate};

    use super::{
        super::{ADDRESS_DOCTYPE, PID_DOCTYPE},
        *,
    };

    static ISSUER_KEY: LazyLock<KeyPair> = LazyLock::new(|| {
        let ca = KeyPair::generate_issuer_mock_ca().unwrap();
        ca.generate_issuer_mock(IssuerRegistration::new_mock().into()).unwrap()
    });

    /// This creates an `UnsignedMdoc` that only contains a bsn entry.
    pub fn create_bsn_only_unsigned_pid_mdoc() -> UnsignedMdoc {
        UnsignedMdoc {
            doc_type: PID_DOCTYPE.to_string(),
            copy_count: NonZeroU8::new(1).unwrap(),
            valid_from: Tdate::now(),
            valid_until: (Utc::now() + Days::new(365)).into(),
            attributes: IndexMap::from([(
                PID_DOCTYPE.to_string(),
                vec![Entry {
                    name: "bsn".to_string(),
                    value: DataElementValue::Text("999999999".to_string()),
                }],
            )])
            .try_into()
            .unwrap(),
        }
    }

    /// This creates a minimal `UnsignedMdoc` that is valid.
    pub fn create_minimal_unsigned_pid_mdoc() -> UnsignedMdoc {
        let mut unsigned_mdoc = create_bsn_only_unsigned_pid_mdoc();
        let mut attributes = unsigned_mdoc.attributes.into_inner();

        attributes.get_mut(PID_DOCTYPE).unwrap().extend(vec![
            Entry {
                name: "family_name".to_string(),
                value: DataElementValue::Text("De Bruijn".to_string()),
            },
            Entry {
                name: "given_name".to_string(),
                value: DataElementValue::Text("Willeke Liselotte".to_string()),
            },
            Entry {
                name: "birth_date".to_string(),
                value: DataElementValue::Text("1997-05-10".to_string()),
            },
            Entry {
                name: "age_over_18".to_string(),
                value: DataElementValue::Bool(true),
            },
        ]);

        unsigned_mdoc.attributes = attributes.try_into().unwrap();

        unsigned_mdoc
    }

    /// This creates a full `UnsignedMdoc` that is valid.
    pub fn create_full_unsigned_pid_mdoc() -> UnsignedMdoc {
        let mut unsigned_mdoc = create_minimal_unsigned_pid_mdoc();
        let mut attributes = unsigned_mdoc.attributes.into_inner();

        attributes.get_mut(PID_DOCTYPE).unwrap().extend(vec![
            Entry {
                name: "family_name_birth".to_string(),
                value: DataElementValue::Text("Molenaar".to_string()),
            },
            Entry {
                name: "given_name_birth".to_string(),
                value: DataElementValue::Text("Liselotte Willeke".to_string()),
            },
            Entry {
                name: "birth_place".to_string(),
                value: DataElementValue::Text("Delft".to_string()),
            },
            Entry {
                name: "birth_country".to_string(),
                value: DataElementValue::Text("NL".to_string()),
            },
            Entry {
                name: "birth_state".to_string(),
                value: DataElementValue::Text("Zuid-Holland".to_string()),
            },
            Entry {
                name: "birth_city".to_string(),
                value: DataElementValue::Text("Delft".to_string()),
            },
            Entry {
                name: "gender".to_string(),
                value: DataElementValue::Integer(2.into()),
            },
        ]);

        unsigned_mdoc.attributes = attributes.try_into().unwrap();

        unsigned_mdoc
    }

    /// This creates a minimal `UnsignedMdoc` that is valid.
    pub fn create_minimal_unsigned_address_mdoc() -> UnsignedMdoc {
        UnsignedMdoc {
            doc_type: ADDRESS_DOCTYPE.to_string(),
            copy_count: NonZeroU8::new(1).unwrap(),
            valid_from: Tdate::now(),
            valid_until: (Utc::now() + Days::new(365)).into(),
            attributes: IndexMap::from([(
                ADDRESS_DOCTYPE.to_string(),
                vec![
                    Entry {
                        name: "resident_street".to_string(),
                        value: DataElementValue::Text("Turfmarkt".to_string()),
                    },
                    Entry {
                        name: "resident_house_number".to_string(),
                        value: DataElementValue::Text("147".to_string()),
                    },
                    Entry {
                        name: "resident_postal_code".to_string(),
                        value: DataElementValue::Text("2511 DP".to_string()),
                    },
                    Entry {
                        name: "resident_city".to_string(),
                        value: DataElementValue::Text("Den Haag".to_string()),
                    },
                ],
            )])
            .try_into()
            .unwrap(),
        }
    }

    /// This creates a full `UnsignedMdoc` that is valid.
    pub fn create_full_unsigned_address_mdoc() -> UnsignedMdoc {
        let mut unsigned_mdoc = create_minimal_unsigned_address_mdoc();
        let mut attributes = unsigned_mdoc.attributes.into_inner();

        attributes.get_mut(ADDRESS_DOCTYPE).unwrap().extend(vec![
            Entry {
                name: "resident_address".to_string(),
                value: DataElementValue::Text("Turfmarkt 147".to_string()),
            },
            Entry {
                name: "resident_state".to_string(),
                value: DataElementValue::Text("Zuid-Holland".to_string()),
            },
            Entry {
                name: "resident_country".to_string(),
                value: DataElementValue::Text("NL".to_string()),
            },
        ]);

        unsigned_mdoc.attributes = attributes.try_into().unwrap();

        unsigned_mdoc
    }

    #[test]
    fn test_minimal_unsigned_mdoc_to_document_mapping() {
        let unsigned_mdoc = create_minimal_unsigned_pid_mdoc();

        let document = Document::from_unsigned_mdoc(unsigned_mdoc, IssuerRegistration::new_mock())
            .expect("Could not convert minimal mdoc to document");

        assert_matches!(document.persistence, DocumentPersistence::InMemory);
        assert_eq!(document.doc_type, PID_DOCTYPE);
        assert_eq!(
            document.attributes.keys().cloned().collect::<Vec<_>>(),
            vec!["given_name", "family_name", "birth_date", "age_over_18", "bsn"]
        );
        assert_matches!(
            document.attributes.get("given_name").unwrap(),
            Attribute {
                key_labels,
                value: AttributeValue::String(given_name),
            } if key_labels == &HashMap::from([("en", "First names"), ("nl", "Voornamen")]) &&
                 given_name == "Willeke Liselotte"
        );
        assert_matches!(
            document.attributes.get("family_name").unwrap(),
            Attribute {
                key_labels: _,
                value: AttributeValue::String(family_name),
            } if family_name == "De Bruijn"
        );
        assert_matches!(
            document.attributes.get("birth_date").unwrap(),
            Attribute {
                key_labels: _,
                value: AttributeValue::Date(birth_date),
            } if birth_date == &NaiveDate::parse_from_str("1997-05-10", "%Y-%m-%d").unwrap()
        );
        assert_matches!(
            document.attributes.get("age_over_18").unwrap(),
            Attribute {
                key_labels: _,
                value: AttributeValue::Boolean(true),
            }
        );
        assert_matches!(
            document.attributes.get("bsn").unwrap(),
            Attribute {
                key_labels: _,
                value: AttributeValue::String(given_name),
            } if given_name == "999999999"
        );
    }

    #[test]
    fn test_full_unsigned_mdoc_to_document_mapping() {
        let unsigned_mdoc = create_full_unsigned_pid_mdoc();

        let document = Document::from_unsigned_mdoc(unsigned_mdoc, IssuerRegistration::new_mock())
            .expect("Could not convert full mdoc to document");

        assert_matches!(
            document.attributes.get("gender").unwrap(),
            Attribute {
                key_labels: _,
                value: AttributeValue::Gender(GenderAttributeValue::Female),
            }
        );
    }

    #[test]
    fn test_unsigned_mdoc_to_document_mapping_doc_type_error() {
        // Test changing the doc_type.
        let mut unsigned_mdoc = create_minimal_unsigned_pid_mdoc();
        unsigned_mdoc.doc_type = "com.example.foobar".to_string();

        let result = Document::from_unsigned_mdoc(unsigned_mdoc, IssuerRegistration::new_mock());

        assert_matches!(
            result,
            Err(DocumentMdocError::UnknownDocType { doc_type }) if doc_type == "com.example.foobar"
        );
    }

    #[test]
    fn test_unsigned_mdoc_to_document_mapping_missing_attribute_error() {
        // Test removing the `age_over_18` attribute.
        let mut unsigned_mdoc = create_minimal_unsigned_pid_mdoc();
        let mut attributes = unsigned_mdoc.attributes.into_inner();
        attributes.get_mut(PID_DOCTYPE).unwrap().pop();
        unsigned_mdoc.attributes = attributes.try_into().unwrap();

        let result = Document::from_unsigned_mdoc(unsigned_mdoc, IssuerRegistration::new_mock());

        assert_matches!(
            result,
            Err(DocumentMdocError::MissingAttribute {
                doc_type,
                name_space,
                name
            }) if doc_type == PID_DOCTYPE && name_space == PID_DOCTYPE && name == "age_over_18"
        );

        // Test removing the "gender" attribute, conversion should still succeed.
        unsigned_mdoc = create_full_unsigned_pid_mdoc();
        let mut attributes = unsigned_mdoc.attributes.into_inner();
        attributes.get_mut(PID_DOCTYPE).unwrap().pop();
        unsigned_mdoc.attributes = attributes.try_into().unwrap();

        _ = Document::from_unsigned_mdoc(unsigned_mdoc, IssuerRegistration::new_mock())
            .expect("Could not convert full mdoc to document");
    }

    #[test]
    fn test_unsigned_mdoc_to_document_mapping_attribute_value_type_mismatch_error() {
        // Test changing the "bsn" attribute to an integer.
        let mut unsigned_mdoc = create_minimal_unsigned_pid_mdoc();
        let mut attributes = unsigned_mdoc.attributes.into_inner();
        _ = mem::replace(
            &mut attributes.get_mut(PID_DOCTYPE).unwrap()[0],
            Entry {
                name: "bsn".to_string(),
                value: DataElementValue::Integer(1234.into()),
            },
        );
        unsigned_mdoc.attributes = attributes.try_into().unwrap();

        let result = Document::from_unsigned_mdoc(unsigned_mdoc, IssuerRegistration::new_mock());

        assert_matches!(
            result,
            Err(DocumentMdocError::AttributeValueTypeMismatch {
                doc_type,
                name_space,
                name,
                expected_type: AttributeValueType::String,
                value,
            }) if doc_type == PID_DOCTYPE && name_space == PID_DOCTYPE &&
                  name == "bsn" && value == DataElementValue::Integer(1234.into())
        );

        // Test changing the "birth_date" attribute to an invalid date.
        let mut unsigned_mdoc = create_minimal_unsigned_pid_mdoc();
        let mut attributes = unsigned_mdoc.attributes.into_inner();
        _ = mem::replace(
            &mut attributes.get_mut(PID_DOCTYPE).unwrap()[3],
            Entry {
                name: "birth_date".to_string(),
                value: DataElementValue::Text("1997-04-31".to_string()),
            },
        );
        unsigned_mdoc.attributes = attributes.try_into().unwrap();

        let result = Document::from_unsigned_mdoc(unsigned_mdoc, IssuerRegistration::new_mock());

        assert_matches!(
            result,
            Err(DocumentMdocError::AttributeValueTypeMismatch {
                doc_type,
                name_space,
                name,
                expected_type: AttributeValueType::Date,
                value,
            }) if doc_type == PID_DOCTYPE && name_space == PID_DOCTYPE &&
                  name == "birth_date" && value == DataElementValue::Text("1997-04-31".to_string())
        );

        // Test changing the "gender" attribute to an invalid value.
        let mut unsigned_mdoc = create_full_unsigned_pid_mdoc();
        let mut attributes = unsigned_mdoc.attributes.into_inner();
        _ = mem::replace(
            attributes.get_mut(PID_DOCTYPE).unwrap().last_mut().unwrap(),
            Entry {
                name: "gender".to_string(),
                value: DataElementValue::Integer(5.into()),
            },
        );
        unsigned_mdoc.attributes = attributes.try_into().unwrap();

        let result = Document::from_unsigned_mdoc(unsigned_mdoc, IssuerRegistration::new_mock());

        assert_matches!(
            result,
            Err(DocumentMdocError::AttributeValueTypeMismatch {
                doc_type,
                name_space,
                name,
                expected_type: AttributeValueType::Gender,
                value,
            }) if doc_type == PID_DOCTYPE && name_space == PID_DOCTYPE &&
                  name == "gender" && value == DataElementValue::Integer(5.into())
        );
    }

    #[test]
    fn test_unsigned_mdoc_to_document_mapping_unknown_attribute_error() {
        // Test adding an unknown entry.
        let mut unsigned_mdoc = create_minimal_unsigned_pid_mdoc();
        let mut attributes = unsigned_mdoc.attributes.into_inner();
        attributes.get_mut(PID_DOCTYPE).unwrap().push(Entry {
            name: "foobar".to_string(),
            value: DataElementValue::Text("Foo Bar".to_string()),
        });
        unsigned_mdoc.attributes = attributes.try_into().unwrap();

        let result = Document::from_unsigned_mdoc(unsigned_mdoc, IssuerRegistration::new_mock());

        assert_matches!(
            result,
            Err(DocumentMdocError::UnknownAttribute {
                doc_type,
                name_space,
                name,
                value,
            }) if doc_type == PID_DOCTYPE && name_space == PID_DOCTYPE &&
                  name == "foobar" && value == Some(DataElementValue::Text("Foo Bar".to_string()))
        );
    }

    #[test]
    fn test_mdoc_to_proposed_disclosure_document_mapping_minimal() {
        let unsigned_mdoc = create_minimal_unsigned_pid_mdoc();

        let disclosure_document = DisclosureDocument::from_mdoc_attributes(
            &unsigned_mdoc.doc_type,
            ProposedDocumentAttributes {
                attributes: unsigned_mdoc.attributes.into_inner(),
                issuer: ISSUER_KEY.certificate().clone(),
            },
        )
        .expect("Could not convert attributes to proposed disclosure document");

        assert_eq!(disclosure_document.doc_type, PID_DOCTYPE);
        assert_eq!(
            disclosure_document.attributes.keys().cloned().collect::<Vec<_>>(),
            vec!["given_name", "family_name", "birth_date", "age_over_18", "bsn"]
        );
        assert_matches!(
            disclosure_document.attributes.get("given_name").unwrap(),
            Attribute {
                key_labels,
                value: AttributeValue::String(given_name),
            } if key_labels == &HashMap::from([("en", "First names"), ("nl", "Voornamen")]) &&
                 given_name == "Willeke Liselotte"
        );
        assert_matches!(
            disclosure_document.attributes.get("family_name").unwrap(),
            Attribute {
                key_labels: _,
                value: AttributeValue::String(family_name),
            } if family_name == "De Bruijn"
        );
        assert_matches!(
            disclosure_document.attributes.get("birth_date").unwrap(),
            Attribute {
                key_labels: _,
                value: AttributeValue::Date(birth_date),
            } if birth_date == &NaiveDate::parse_from_str("1997-05-10", "%Y-%m-%d").unwrap()
        );
        assert_matches!(
            disclosure_document.attributes.get("age_over_18").unwrap(),
            Attribute {
                key_labels: _,
                value: AttributeValue::Boolean(true),
            }
        );
        assert_matches!(
            disclosure_document.attributes.get("bsn").unwrap(),
            Attribute {
                key_labels: _,
                value: AttributeValue::String(given_name),
            } if given_name == "999999999"
        );
    }

    #[test]
    fn test_mdoc_to_proposed_disclosure_document_mapping_age_over_18() {
        let attributes = IndexMap::from([(
            PID_DOCTYPE.to_string(),
            vec![Entry {
                name: "age_over_18".to_string(),
                value: DataElementValue::Bool(true),
            }],
        )]);

        // This should not result in a `DocumentMdocError::MissingAttribute` error.
        let disclosure_document = DisclosureDocument::from_mdoc_attributes(
            PID_DOCTYPE,
            ProposedDocumentAttributes {
                attributes,
                issuer: ISSUER_KEY.certificate().clone(),
            },
        )
        .expect("Could not convert attributes to proposed disclosure document");

        assert_eq!(disclosure_document.doc_type, PID_DOCTYPE);
        assert_eq!(
            disclosure_document.attributes.keys().cloned().collect::<Vec<_>>(),
            vec!["age_over_18"]
        );
        assert_matches!(
            disclosure_document.attributes.get("age_over_18").unwrap(),
            Attribute {
                key_labels: _,
                value: AttributeValue::Boolean(true),
            }
        );
    }

    #[test]
    fn test_mdoc_to_proposed_disclosure_document_mapping_error_unknown_doc_type() {
        let attributes = IndexMap::from([(
            PID_DOCTYPE.to_string(),
            vec![Entry {
                name: "age_over_18".to_string(),
                value: DataElementValue::Bool(true),
            }],
        )]);

        let result = DisclosureDocument::from_mdoc_attributes(
            "com.example.foobar",
            ProposedDocumentAttributes {
                attributes,
                issuer: ISSUER_KEY.certificate().clone(),
            },
        );

        assert_matches!(
            result,
            Err(DocumentMdocError::UnknownDocType { doc_type }) if doc_type == "com.example.foobar"
        );
    }

    #[test]
    fn test_mdoc_to_proposed_disclosure_document_mapping_error_attribute_value_type_mismatch() {
        let attributes = IndexMap::from([(
            PID_DOCTYPE.to_string(),
            vec![Entry {
                name: "age_over_18".to_string(),
                value: DataElementValue::Text("Yes".to_string()),
            }],
        )]);

        let result = DisclosureDocument::from_mdoc_attributes(
            PID_DOCTYPE,
            ProposedDocumentAttributes {
                attributes,
                issuer: ISSUER_KEY.certificate().clone(),
            },
        );

        assert_matches!(
            result,
            Err(DocumentMdocError::AttributeValueTypeMismatch {
                doc_type,
                name_space,
                name,
                expected_type: AttributeValueType::Bool,
                value,
            }) if doc_type == PID_DOCTYPE && name_space == PID_DOCTYPE &&
                  name == "age_over_18" && value == DataElementValue::Text("Yes".to_string())
        );
    }

    #[test]
    fn test_mdoc_to_proposed_disclosure_document_mapping_error_unknown_attribute() {
        let attributes = IndexMap::from([(
            PID_DOCTYPE.to_string(),
            vec![Entry {
                name: "favourite_colour".to_string(),
                value: DataElementValue::Text("Red".to_string()),
            }],
        )]);

        let result = DisclosureDocument::from_mdoc_attributes(
            PID_DOCTYPE,
            ProposedDocumentAttributes {
                attributes,
                issuer: ISSUER_KEY.certificate().clone(),
            },
        );

        assert_matches!(
            result,
            Err(DocumentMdocError::UnknownAttribute {
                doc_type,
                name_space,
                name,
                value,
            }) if doc_type == PID_DOCTYPE && name_space == PID_DOCTYPE &&
                  name == "favourite_colour" && value == Some(DataElementValue::Text("Red".to_string()))
        );
    }

    #[rstest]
    #[case(vec![], vec![].into())]
    #[case(vec!["com.example.pid/com.example.pid/bsn"], vec![("com.example.pid", vec!["bsn"])].into())]
    #[case(
        vec!["com.example.pid/com.example.pid/bsn", "com.example.pid/com.example.pid/age_over_18"],
        vec![("com.example.pid", vec!["bsn", "age_over_18"])].into())
    ]
    #[case(
        vec![
            "com.example.address/com.example.address/resident_country",
            "com.example.pid/com.example.pid/bsn",
            "com.example.address/com.example.address/resident_state",
            "com.example.pid/com.example.pid/gender",
        ],
        vec![
            ("com.example.pid", vec!["bsn", "gender"]),
            ("com.example.address", vec!["resident_country", "resident_state"])
        ].into())
    ]
    #[case(vec!["com.example.foo/com.example.bar/something"], None)] // DocumentMdocError::UnknownDocType
    #[case(vec!["com.example.pid/com.example.pid/favorite_colour"], None)] // DocumentMdocError::UnknownAttribute
    fn test_missing_disclosure_attributes_from_mdoc_missing_attributes(
        #[case] attribute_identifiers: Vec<&str>,
        #[case] expected_result: Option<Vec<(&str, Vec<&str>)>>,
    ) {
        // Convert the input attribute identifier strings to actual `AttributeIdentifier`s.
        let attribute_identifiers: Vec<AttributeIdentifier> = attribute_identifiers
            .into_iter()
            .map(|attribute| attribute.parse().unwrap())
            .collect();

        // Attempt to convert the identifiers to a `Vec<MissingDisclosureAttributes>`.
        let result = MissingDisclosureAttributes::from_mdoc_missing_attributes(attribute_identifiers);

        // If `expected_result` contains a `Vec`, match the expected `doc_type` and keys against the result.
        // Note that the returned order is relevant.
        if let Some(expected_result) = expected_result {
            let missing = result.expect("Could not convert attribute identifiers to missing disclosure attributes");

            assert_eq!(missing.len(), expected_result.len());
            missing.into_iter().zip(expected_result).for_each(
                |(missing_attributes, (expected_doc_type, expected_attributes))| {
                    assert_eq!(missing_attributes.doc_type, expected_doc_type);
                    assert_eq!(
                        missing_attributes.attributes.into_keys().collect::<Vec<_>>(),
                        expected_attributes
                    );
                },
            );

            return;
        }

        // If `expected_result` is None, the result should be an error.
        assert!(result.is_err());
    }

    #[rstest]
    #[case(create_bsn_only_unsigned_pid_mdoc(), DisclosureType::Login)]
    #[case(create_minimal_unsigned_pid_mdoc(), DisclosureType::Regular)]
    #[case(create_full_unsigned_pid_mdoc(), DisclosureType::Regular)]
    fn test_disclosure_type_from_proposed_attributes(#[case] input: UnsignedMdoc, #[case] expected: DisclosureType) {
        let pa = ProposedAttributes::from([(
            PID_DOCTYPE.to_string(),
            ProposedDocumentAttributes {
                attributes: input.attributes.into_inner(),
                issuer: ISSUER_KEY.certificate().clone(),
            },
        )]);

        assert_eq!(DisclosureType::from_proposed_attributes(&pa), expected);
    }
}
