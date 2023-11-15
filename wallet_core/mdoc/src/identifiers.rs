use indexmap::IndexSet;

use crate::{
    holder::Mdoc,
    iso::{
        device_retrieval::{DeviceRequest, ItemsRequest},
        disclosure::IssuerSigned,
        mdocs::{Attributes, DataElementIdentifier, DocType, NameSpace},
    },
    utils::serialization::TaggedBytes,
    Document,
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
    fn attribute_identifiers(&self, doc_type: &str) -> IndexSet<AttributeIdentifier> {
        self.name_spaces
            .as_ref()
            .map(|name_spaces| {
                name_spaces
                    .into_iter()
                    .flat_map(|(namespace, Attributes(attributes))| {
                        attributes.iter().map(|TaggedBytes(attribute)| AttributeIdentifier {
                            doc_type: doc_type.to_owned(),
                            namespace: namespace.to_owned(),
                            attribute: attribute.element_identifier.to_owned(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Mdoc {
    pub(crate) fn issuer_signed_attribute_identifiers(&self) -> IndexSet<AttributeIdentifier> {
        self.issuer_signed.attribute_identifiers(&self.doc_type)
    }
}

impl Document {
    pub(crate) fn issuer_signed_attribute_identifiers(&self) -> IndexSet<AttributeIdentifier> {
        self.issuer_signed.attribute_identifiers(&self.doc_type)
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
            .flat_map(|(namespace, attributes)| {
                attributes.into_iter().map(|(attribute, _)| AttributeIdentifier {
                    doc_type: self.doc_type.to_owned(),
                    namespace: namespace.to_owned(),
                    attribute: attribute.to_owned(),
                })
            })
            .collect()
    }
}
