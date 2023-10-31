use chrono::NaiveDate;
use ciborium::value::Integer;
use indexmap::IndexMap;

use nl_wallet_mdoc::{
    basic_sa_ext::{Entry, UnsignedMdoc},
    identifiers::AttributeIdentifier,
    DataElementIdentifier, DataElementValue, NameSpace,
};

use super::{
    mapping::{AttributeMapping, DataElementValueMapping, MappingDocType, MDOC_DOCUMENT_MAPPING},
    Attribute, AttributeValue, Document, DocumentPersistence, GenderAttributeValue, MissingDisclosureAttributes,
};

#[derive(Debug, thiserror::Error)]
pub enum DocumentMdocError {
    #[error("unknown doc type \"{doc_type}\"")]
    UnknownDocType { doc_type: String },
    #[error("mandatory attributes for \"{doc_type}\" not found at \"{name_space} / {name}\"")]
    MissingAttribute {
        doc_type: String,
        name_space: NameSpace,
        name: DataElementIdentifier,
    },
    #[error(
        "attribute for \"{doc_type}\" encountered at \"{name_space} / {name}\" \
         does not match expected type {expected_type:?}: {value:?}"
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
}

#[derive(Debug, Clone, Copy)]
pub enum AttributeValueType {
    String,
    Bool,
    Date,
    Gender,
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

impl TryFrom<UnsignedMdoc> for Document {
    type Error = DocumentMdocError;

    fn try_from(value: UnsignedMdoc) -> Result<Self, Self::Error> {
        Document::from_mdoc_attributes(DocumentPersistence::InMemory, &value.doc_type, value.attributes)
    }
}

impl Document {
    pub(crate) fn from_mdoc_attributes(
        persistence: DocumentPersistence,
        doc_type: &str,
        mut attributes: IndexMap<NameSpace, Vec<Entry>>,
    ) -> Result<Self, DocumentMdocError> {
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
                // mandatory, we can return `None` early here.
                if entry.is_none() && !value_mapping.is_mandatory {
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

        let document = Document {
            persistence,
            doc_type,
            attributes: document_attributes,
        };

        Ok(document)
    }
}

impl TryFrom<(DataElementValue, &DataElementValueMapping)> for Attribute {
    type Error = DataElementValue;

    fn try_from(value: (DataElementValue, &DataElementValueMapping)) -> Result<Self, Self::Error> {
        let (value, value_mapping) = value;
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
                        let (doc_type, attribute_mapping) = mapping_for_doc_type(missing_attribute.doc_type.as_str())?;
                        let value_mapping = attribute_mapping
                            .get(&(
                                missing_attribute.namespace.as_str(),
                                missing_attribute.attribute.as_str(),
                            ))
                            .ok_or_else(|| DocumentMdocError::UnknownAttribute {
                                doc_type: missing_attribute.doc_type,
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

#[cfg(test)]
pub mod tests {
    use std::{collections::HashMap, mem};

    use assert_matches::assert_matches;
    use chrono::{Days, Utc};

    use nl_wallet_mdoc::Tdate;

    use super::{super::PID_DOCTYPE, *};

    /// This creates a minimal `UnsignedMdoc` that is valid.
    pub fn create_minimal_unsigned_pid_mdoc() -> UnsignedMdoc {
        UnsignedMdoc {
            doc_type: PID_DOCTYPE.to_string(),
            copy_count: 1,
            valid_from: Tdate::now(),
            valid_until: (Utc::now() + Days::new(365)).into(),
            attributes: IndexMap::from([(
                PID_DOCTYPE.to_string(),
                vec![
                    Entry {
                        name: "bsn".to_string(),
                        value: DataElementValue::Text("999999999".to_string()),
                    },
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
                ],
            )]),
        }
    }

    /// This creates a full `UnsignedMdoc` that is valid.
    pub fn create_full_unsigned_pid_mdoc() -> UnsignedMdoc {
        let mut unsigned_mdoc = create_minimal_unsigned_pid_mdoc();

        unsigned_mdoc.attributes.get_mut(PID_DOCTYPE).unwrap().extend(vec![
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

        unsigned_mdoc
    }

    #[test]
    fn test_minimal_unsigned_mdoc_to_document_mapping() {
        let unsigned_mdoc = create_minimal_unsigned_pid_mdoc();

        let document = Document::try_from(unsigned_mdoc).expect("Could not convert minimal mdoc to document");

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

        let document = Document::try_from(unsigned_mdoc).expect("Could not convert full mdoc to document");

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

        let result = Document::try_from(unsigned_mdoc);

        assert_matches!(
            result,
            Err(DocumentMdocError::UnknownDocType { doc_type }) if doc_type == "com.example.foobar"
        );
    }

    #[test]
    fn test_unsigned_mdoc_to_document_mapping_missing_attribute_error() {
        // Test removing the `age_over_18` attribute.
        let mut unsigned_mdoc = create_minimal_unsigned_pid_mdoc();
        unsigned_mdoc.attributes.get_mut(PID_DOCTYPE).unwrap().pop();

        let result = Document::try_from(unsigned_mdoc);

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
        unsigned_mdoc.attributes.get_mut(PID_DOCTYPE).unwrap().pop();

        _ = Document::try_from(unsigned_mdoc).expect("Could not convert full mdoc to document");
    }

    #[test]
    fn test_unsigned_mdoc_to_document_mapping_attribute_value_type_mismatch_error() {
        // Test changing the "bsn" attribute to an integer.
        let mut unsigned_mdoc = create_minimal_unsigned_pid_mdoc();
        _ = mem::replace(
            &mut unsigned_mdoc.attributes.get_mut(PID_DOCTYPE).unwrap()[0],
            Entry {
                name: "bsn".to_string(),
                value: DataElementValue::Integer(1234.into()),
            },
        );

        let result = Document::try_from(unsigned_mdoc);

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
        _ = mem::replace(
            &mut unsigned_mdoc.attributes.get_mut(PID_DOCTYPE).unwrap()[3],
            Entry {
                name: "birth_date".to_string(),
                value: DataElementValue::Text("1997-04-31".to_string()),
            },
        );

        let result = Document::try_from(unsigned_mdoc);

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
        _ = mem::replace(
            unsigned_mdoc
                .attributes
                .get_mut(PID_DOCTYPE)
                .unwrap()
                .last_mut()
                .unwrap(),
            Entry {
                name: "gender".to_string(),
                value: DataElementValue::Integer(5.into()),
            },
        );

        let result = Document::try_from(unsigned_mdoc);

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
        unsigned_mdoc.attributes.get_mut(PID_DOCTYPE).unwrap().push(Entry {
            name: "foobar".to_string(),
            value: DataElementValue::Text("Foo Bar".to_string()),
        });

        let result = Document::try_from(unsigned_mdoc);

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
}
