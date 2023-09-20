use std::collections::HashMap;

use chrono::NaiveDate;
use ciborium::value::Integer;
use indexmap::IndexMap;
use nl_wallet_mdoc::{
    basic_sa_ext::{Entry, UnsignedMdoc},
    DataElementIdentifier, DataElementValue, NameSpace,
};
use once_cell::sync::Lazy;

use super::{
    Attribute, AttributeKey, AttributeLabel, AttributeLabelLanguage, AttributeValue, Document, GenderAttributeValue,
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
        value: DataElementValue,
    },
}

#[derive(Debug, Clone)]
struct DataElementValueMapping {
    key: AttributeKey,
    is_mandatory: bool,
    key_labels: HashMap<AttributeLabelLanguage, AttributeLabel>,
    value_type: AttributeValueType,
}

#[derive(Debug, Clone, Copy)]
pub enum AttributeValueType {
    String,
    Bool,
    Date,
    Gender,
}

type MappingNameSpace = &'static str;
type MappingDataElementIdentifier = &'static str;
type AttributeMapping = IndexMap<(MappingNameSpace, MappingDataElementIdentifier), DataElementValueMapping>;

type MappingDocType = &'static str;
type MdocDocumentMapping = HashMap<MappingDocType, AttributeMapping>;

static MDOC_DOCUMENT_MAPPING: Lazy<MdocDocumentMapping> = Lazy::new(|| {
    HashMap::from([
        (
            "com.example.pid",
            IndexMap::from([
                (
                    ("com.example.pid", "unique_id"),
                    DataElementValueMapping {
                        key: "unique_id",
                        is_mandatory: true,
                        key_labels: HashMap::from([("en", "Unique identifier"), ("nl", "Unieke identificatiecode")]),
                        value_type: AttributeValueType::String,
                    },
                ),
                (
                    ("com.example.pid", "given_name"),
                    DataElementValueMapping {
                        key: "given_name",
                        is_mandatory: true,
                        key_labels: HashMap::from([("en", "First names"), ("nl", "Voornamen")]),
                        value_type: AttributeValueType::String,
                    },
                ),
                (
                    ("com.example.pid", "family_name"),
                    DataElementValueMapping {
                        key: "family_name",
                        is_mandatory: true,
                        key_labels: HashMap::from([("en", "Surname"), ("nl", "Achternaam")]),
                        value_type: AttributeValueType::String,
                    },
                ),
                (
                    ("com.example.pid", "given_name_birth"),
                    DataElementValueMapping {
                        key: "given_name_birth",
                        is_mandatory: false,
                        key_labels: HashMap::from([("en", "First names at birth"), ("nl", "Voornamen bij geboorte")]),
                        value_type: AttributeValueType::String,
                    },
                ),
                (
                    ("com.example.pid", "family_name_birth"),
                    DataElementValueMapping {
                        key: "family_name_birth",
                        is_mandatory: false,
                        key_labels: HashMap::from([("en", "Birth name"), ("nl", "Geboortenaam")]),
                        value_type: AttributeValueType::String,
                    },
                ),
                (
                    ("com.example.pid", "gender"),
                    DataElementValueMapping {
                        key: "gender",
                        is_mandatory: false,
                        key_labels: HashMap::from([("en", "Gender"), ("nl", "Geslacht")]),
                        value_type: AttributeValueType::Gender,
                    },
                ),
                (
                    ("com.example.pid", "birth_date"),
                    DataElementValueMapping {
                        key: "birth_date",
                        is_mandatory: true,
                        key_labels: HashMap::from([("en", "Birth date"), ("nl", "Geboortedatum")]),
                        value_type: AttributeValueType::Date,
                    },
                ),
                (
                    ("com.example.pid", "age_over_18"),
                    DataElementValueMapping {
                        key: "age_over_18",
                        is_mandatory: true,
                        key_labels: HashMap::from([("en", "Older than 18"), ("nl", "Ouder dan 18")]),
                        value_type: AttributeValueType::Bool,
                    },
                ),
                (
                    ("com.example.pid", "birth_place"),
                    DataElementValueMapping {
                        key: "birth_place",
                        is_mandatory: false,
                        key_labels: HashMap::from([("en", "Place of birth"), ("nl", "Geboorteplaats")]),
                        value_type: AttributeValueType::String,
                    },
                ),
                (
                    ("com.example.pid", "birth_city"),
                    DataElementValueMapping {
                        key: "birth_city",
                        is_mandatory: false,
                        key_labels: HashMap::from([("en", "City, town or village of birth"), ("nl", "Geboortestad")]),
                        value_type: AttributeValueType::String,
                    },
                ),
                (
                    ("com.example.pid", "birth_state"),
                    DataElementValueMapping {
                        key: "birth_state",
                        is_mandatory: false,
                        key_labels: HashMap::from([
                            ("en", "State or province of birth"),
                            ("nl", "Geboortestaat of -provincie"),
                        ]),
                        value_type: AttributeValueType::String,
                    },
                ),
                (
                    ("com.example.pid", "birth_country"),
                    DataElementValueMapping {
                        key: "birth_country",
                        is_mandatory: false,
                        key_labels: HashMap::from([("en", "Country of birth"), ("nl", "Geboorteland")]),
                        value_type: AttributeValueType::String,
                    },
                ),
                (
                    ("com.example.pid", "bsn"),
                    DataElementValueMapping {
                        key: "bsn",
                        is_mandatory: true,
                        key_labels: HashMap::from([("en", "BSN"), ("nl", "BSN")]),
                        value_type: AttributeValueType::String,
                    },
                ),
                (
                    ("com.example.pid", "nationality"),
                    DataElementValueMapping {
                        key: "nationality",
                        is_mandatory: false,
                        key_labels: HashMap::from([("en", "Nationality"), ("nl", "Nationaliteit")]),
                        value_type: AttributeValueType::String,
                    },
                ),
            ]),
        ),
        (
            "com.example.address",
            IndexMap::from([
                (
                    ("com.example.address", "resident_address"),
                    DataElementValueMapping {
                        key: "resident_address",
                        is_mandatory: false,
                        key_labels: HashMap::from([("en", "Address"), ("nl", "Adres")]),
                        value_type: AttributeValueType::String,
                    },
                ),
                (
                    ("com.example.address", "resident_street"),
                    DataElementValueMapping {
                        key: "resident_street",
                        is_mandatory: false,
                        key_labels: HashMap::from([("en", "Street"), ("nl", "Straatnaam")]),
                        value_type: AttributeValueType::String,
                    },
                ),
                (
                    ("com.example.address", "resident_house_number"),
                    DataElementValueMapping {
                        key: "resident_house_number",
                        is_mandatory: false,
                        key_labels: HashMap::from([("en", "House number"), ("nl", "Huisnummer")]),
                        value_type: AttributeValueType::String,
                    },
                ),
                (
                    ("com.example.address", "resident_postal_code"),
                    DataElementValueMapping {
                        key: "resident_postal_code",
                        is_mandatory: false,
                        key_labels: HashMap::from([("en", "Postal code"), ("nl", "Postcode")]),
                        value_type: AttributeValueType::String,
                    },
                ),
                (
                    ("com.example.address", "resident_city"),
                    DataElementValueMapping {
                        key: "resident_city",
                        is_mandatory: false,
                        key_labels: HashMap::from([("en", "City, town or village"), ("nl", "Woonplaats")]),
                        value_type: AttributeValueType::String,
                    },
                ),
                (
                    ("com.example.address", "resident_state"),
                    DataElementValueMapping {
                        key: "resident_state",
                        is_mandatory: false,
                        key_labels: HashMap::from([("en", "State or province"), ("nl", "Staat of provincie")]),
                        value_type: AttributeValueType::String,
                    },
                ),
                (
                    ("com.example.address", "resident_country"),
                    DataElementValueMapping {
                        key: "resident_country",
                        is_mandatory: false,
                        key_labels: HashMap::from([("en", "Country"), ("nl", "Land")]),
                        value_type: AttributeValueType::String,
                    },
                ),
            ]),
        ),
    ])
});

impl TryFrom<UnsignedMdoc> for Document {
    type Error = DocumentMdocError;

    fn try_from(value: UnsignedMdoc) -> Result<Self, Self::Error> {
        Document::from_mdoc_attributes(None, &value.doc_type, value.attributes)
    }
}

impl Document {
    fn from_mdoc_attributes(
        id: Option<String>,
        doc_type: &str,
        mut attributes: IndexMap<NameSpace, Vec<Entry>>,
    ) -> Result<Self, DocumentMdocError> {
        let (doc_type, attribute_mapping) =
            MDOC_DOCUMENT_MAPPING
                .get_key_value(doc_type)
                .ok_or_else(|| DocumentMdocError::UnknownDocType {
                    doc_type: doc_type.to_string(),
                })?;

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
                    value: entry.value,
                })
            })
            .next();

        // Return the error if at least one mdoc attributes still remained.
        if let Some(missing_error) = unknown_error {
            return Err(missing_error);
        }

        let document = Document {
            id,
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
