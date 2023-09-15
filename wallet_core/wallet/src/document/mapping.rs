use std::{borrow::Cow, collections::HashMap};

use indexmap::IndexMap;
use nl_wallet_mdoc::{basic_sa_ext::Entry, DataElementValue, NameSpace};
use once_cell::sync::Lazy;

use super::{Attribute, AttributeKey, AttributeLabel, AttributeLabelLanguage, AttributeValue, Document, DocumentType};

struct DataElementValueMapping {
    key: AttributeKey,
    key_labels: HashMap<AttributeLabelLanguage, AttributeLabel>,
}

type MappingNameSpace = &'static str;
type MappingDataElementIdentifier = &'static str;
type AttributeMapping = IndexMap<(MappingNameSpace, MappingDataElementIdentifier), DataElementValueMapping>;

struct MdocMapping {
    document_type: DocumentType,
    attribute_mapping: AttributeMapping,
}

type MappingDocType = &'static str;
type MdocDocumentMapping = HashMap<MappingDocType, MdocMapping>;

static MDOC_DOCUMENT_MAPPING: Lazy<MdocDocumentMapping> = Lazy::new(|| {
    HashMap::from([(
        "com.example.pid",
        MdocMapping {
            document_type: DocumentType::Identity,
            attribute_mapping: IndexMap::from([(
                ("com.example.pid", "bsn"),
                DataElementValueMapping {
                    key: "bsn",
                    key_labels: HashMap::from([("en", "BSN"), ("nl", "BSN")]),
                },
            )]),
        },
    )])
});

impl Document {
    pub(crate) fn from_mdoc_attributes(
        id: Option<String>,
        doc_type: &str,
        attributes: IndexMap<NameSpace, Vec<Entry>>,
    ) -> Option<Self> {
        MDOC_DOCUMENT_MAPPING.get(doc_type).map(|mdoc_mapping| {
            // Split the attributes `IndexMap` into two owned `Vec`s.
            let (name_spaces, entries): (Vec<_>, Vec<_>) = attributes.into_iter().unzip();

            // Build a `HashMap` from the entry values, keyed by a tuple of the name space
            // and entry name. Note that we have to use a `Cow` for the entry name, which
            // is an owned string, so that it may be looked up using a `&str`.
            let mut attribute_values: HashMap<(_, Cow<_>), _> = name_spaces
                .iter()
                .zip(entries.into_iter())
                .flat_map(|(name_space, entries)| {
                    entries
                        .into_iter()
                        .map(|entry| ((name_space.as_str(), entry.name.into()), entry.value))
                })
                .collect();

            // Finally, loop through the attributes in the mapping in order, look them up in
            // the `HashMap` we just created and build an `IndexMap` of `Attribute`s from them.
            let document_attributes = mdoc_mapping
                .attribute_mapping
                .iter()
                .flat_map(|((name_space, element_id), value_mapping)| {
                    attribute_values
                        .remove(&(*name_space, Cow::from(*element_id)))
                        .and_then(|value| Attribute::from_data_value(value, value_mapping))
                        .map(|attribute| (value_mapping.key, attribute))
                })
                .collect();

            Document {
                id,
                document_type: mdoc_mapping.document_type,
                attributes: document_attributes,
            }
        })
    }
}

impl Attribute {
    fn from_data_value(value: DataElementValue, value_mapping: &DataElementValueMapping) -> Option<Self> {
        // TODO: Maybe check the expected value type.
        AttributeValue::from_data_element_value(value).map(|value| Attribute {
            key_labels: value_mapping.key_labels.clone(),
            value,
        })
    }
}

impl AttributeValue {
    fn from_data_element_value(value: DataElementValue) -> Option<Self> {
        match value {
            DataElementValue::Text(s) => Self::String(s).into(),
            _ => None,
        }
    }
}
