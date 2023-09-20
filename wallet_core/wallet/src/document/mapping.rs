use std::collections::HashMap;

use indexmap::IndexMap;
use nl_wallet_mdoc::{
    basic_sa_ext::{Entry, UnsignedMdoc},
    DataElementIdentifier, DataElementValue, NameSpace,
};
use once_cell::sync::Lazy;

use super::{Attribute, AttributeKey, AttributeLabel, AttributeLabelLanguage, AttributeValue, Document};

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
}

type MappingNameSpace = &'static str;
type MappingDataElementIdentifier = &'static str;
type AttributeMapping = IndexMap<(MappingNameSpace, MappingDataElementIdentifier), DataElementValueMapping>;

type MappingDocType = &'static str;
type MdocDocumentMapping = HashMap<MappingDocType, AttributeMapping>;

static MDOC_DOCUMENT_MAPPING: Lazy<MdocDocumentMapping> = Lazy::new(|| {
    HashMap::from([(
        "com.example.pid",
        IndexMap::from([(
            ("com.example.pid", "bsn"),
            DataElementValueMapping {
                key: "bsn",
                is_mandatory: true,
                key_labels: HashMap::from([("en", "BSN"), ("nl", "BSN")]),
                value_type: AttributeValueType::String,
            },
        )]),
    )])
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
        let value = (value, value_mapping.value_type).try_into()?;

        let attribute = Attribute {
            key_labels: value_mapping.key_labels.clone(),
            value,
        };

        Ok(attribute)
    }
}

impl TryFrom<(DataElementValue, AttributeValueType)> for AttributeValue {
    type Error = DataElementValue;

    fn try_from(value: (DataElementValue, AttributeValueType)) -> Result<Self, Self::Error> {
        match value {
            (DataElementValue::Text(s), AttributeValueType::String) => Ok(Self::String(s)),
            _ => Err(value.0),
        }
    }
}
