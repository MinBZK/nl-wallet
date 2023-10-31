use indexmap::IndexSet;

use crate::{
    iso::{
        device_retrieval::{DeviceRequest, ItemsRequest},
        disclosure::IssuerSigned,
        mdocs::{Attributes, DataElementIdentifier, DocType, NameSpace},
    },
    utils::serialization::TaggedBytes,
};

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct AttributeIdentifier {
    pub doc_type: DocType,
    pub namespace: NameSpace,
    pub attribute: DataElementIdentifier,
}

impl std::fmt::Debug for AttributeIdentifier {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        fmt.write_fmt(format_args!("{}/{}/{}", self.doc_type, self.namespace, self.attribute))
    }
}

impl IssuerSigned {
    pub(crate) fn attribute_identifiers(&self, doc_type: &str) -> IndexSet<AttributeIdentifier> {
        self.name_spaces
            .as_ref()
            .map(|name_spaces| {
                name_spaces
                    .iter()
                    .flat_map(|(namespace, Attributes(attrs))| {
                        attrs.iter().map(|TaggedBytes(attr)| AttributeIdentifier {
                            doc_type: doc_type.to_string(),
                            namespace: namespace.clone(),
                            attribute: attr.element_identifier.clone(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl DeviceRequest {
    pub(crate) fn attribute_identifiers(&self) -> IndexSet<AttributeIdentifier> {
        self.doc_requests
            .iter()
            .flat_map(|doc_request| doc_request.items_request.0.attribute_identifiers())
            .collect()
    }
}

impl ItemsRequest {
    pub(crate) fn attribute_identifiers(&self) -> IndexSet<AttributeIdentifier> {
        self.name_spaces
            .iter()
            .flat_map(|(namespace, attrs)| {
                attrs.iter().map(|(attr, _)| AttributeIdentifier {
                    doc_type: self.doc_type.clone(),
                    namespace: namespace.clone(),
                    attribute: attr.clone(),
                })
            })
            .collect()
    }
}
