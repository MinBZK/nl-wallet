use indexmap::IndexSet;

use attestation::identifiers::AttributeIdentifier;
use attestation::identifiers::AttributeIdentifierHolder;

use crate::holder::Mdoc;
use crate::iso::device_retrieval::DeviceRequest;
use crate::iso::device_retrieval::ItemsRequest;
use crate::utils::serialization::TaggedBytes;
use crate::Document;
use crate::IssuerSigned;

impl IssuerSigned {
    fn attribute_identifiers(&self, doc_type: &str) -> IndexSet<AttributeIdentifier> {
        self.name_spaces
            .as_ref()
            .map(|name_spaces| {
                name_spaces
                    .as_ref()
                    .iter()
                    .flat_map(|(namespace, attributes)| {
                        attributes
                            .as_ref()
                            .iter()
                            .map(|TaggedBytes(attribute)| AttributeIdentifier {
                                credential_type: doc_type.to_owned(),
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
    pub fn issuer_signed_attribute_identifiers(&self) -> IndexSet<AttributeIdentifier> {
        self.issuer_signed.attribute_identifiers(self.doc_type())
    }
}

impl Document {
    pub fn issuer_signed_attribute_identifiers(&self) -> IndexSet<AttributeIdentifier> {
        self.issuer_signed.attribute_identifiers(&self.doc_type)
    }
}

impl AttributeIdentifierHolder for DeviceRequest {
    fn attribute_identifiers(&self) -> IndexSet<AttributeIdentifier> {
        self.doc_requests
            .iter()
            .flat_map(|doc_request| doc_request.items_request.0.attribute_identifiers())
            .collect()
    }
}

impl AttributeIdentifierHolder for ItemsRequest {
    fn attribute_identifiers(&self) -> IndexSet<AttributeIdentifier> {
        self.name_spaces
            .iter()
            .flat_map(|(namespace, attributes)| {
                attributes.into_iter().map(|(attribute, _)| AttributeIdentifier {
                    credential_type: self.doc_type.to_owned(),
                    namespace: namespace.to_owned(),
                    attribute: attribute.to_owned(),
                })
            })
            .collect()
    }
}
