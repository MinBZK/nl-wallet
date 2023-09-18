use std::collections::HashMap;

use indexmap::IndexMap;
use nl_wallet_mdoc::{basic_sa_ext::Entry, DataElementValue, NameSpace};
use once_cell::sync::Lazy;

use super::{Attribute, AttributeKey, AttributeLabel, AttributeLabelLanguage, AttributeValue, Document};

struct DataElementValueMapping {
    key: AttributeKey,
    key_labels: HashMap<AttributeLabelLanguage, AttributeLabel>,
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
                key_labels: HashMap::from([("en", "BSN"), ("nl", "BSN")]),
            },
        )]),
    )])
});

impl Document {
    pub(crate) fn from_mdoc_attributes(
        id: Option<String>,
        doc_type: &str,
        mut attributes: IndexMap<NameSpace, Vec<Entry>>,
    ) -> Option<Self> {
        MDOC_DOCUMENT_MAPPING
            .get_key_value(doc_type)
            .map(|(doc_type, attribute_mapping)| {
                // Loop through the attributes in the mapping in order and find
                // the corresponding entry in the input attributes, based on the
                // name space and the entry name. If found, move the entry value
                // out of the input attributes and try to convert it to an `Attribute`.
                let document_attributes = attribute_mapping
                    .iter()
                    .flat_map(|((name_space, element_id), value_mapping)| {
                        attributes.get_mut(*name_space).map(|entries| {
                            entries
                                .iter()
                                .position(|entry| entry.name == *element_id)
                                .map(|index| entries.swap_remove(index).value)
                                .and_then(|value| Attribute::from_data_value(value, value_mapping))
                                .map(|attribute| (value_mapping.key, attribute))
                        })
                    })
                    .flatten()
                    .collect();

                // TODO: Return an error when encountering an unknown key
                // TODO: Make some attributes mandatory in the mapping,
                //       return an error when when they are not present.

                Document {
                    id,
                    doc_type,
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
