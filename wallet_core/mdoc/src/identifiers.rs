use indexmap::IndexSet;

use crate::{
    holder::Mdoc,
    iso::{
        device_retrieval::{DeviceRequest, ItemsRequest},
        disclosure::IssuerSigned,
        mdocs::{DataElementIdentifier, NameSpace},
    },
    utils::serialization::TaggedBytes,
    Document,
};

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct AttributeIdentifier {
    pub credential_type: String,
    pub namespace: NameSpace,
    pub attribute: DataElementIdentifier,
}

impl std::fmt::Debug for AttributeIdentifier {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        fmt.write_fmt(format_args!(
            "{}/{}/{}",
            self.credential_type, self.namespace, self.attribute
        ))
    }
}

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
        self.issuer_signed.attribute_identifiers(&self.doc_type)
    }
}

impl Document {
    pub fn issuer_signed_attribute_identifiers(&self) -> IndexSet<AttributeIdentifier> {
        self.issuer_signed.attribute_identifiers(&self.doc_type)
    }
}

pub trait AttributeIdentifierHolder {
    fn attribute_identifiers(&self) -> IndexSet<AttributeIdentifier>;

    fn difference(&self, other: &impl AttributeIdentifierHolder) -> IndexSet<AttributeIdentifier> {
        let other_attributes = other.attribute_identifiers();
        self.attribute_identifiers()
            .into_iter()
            .filter(|attribute| !other_attributes.contains(attribute))
            .collect()
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

#[cfg(any(test, feature = "examples"))]
mod examples {
    use indexmap::IndexSet;

    use crate::examples::{EXAMPLE_DOC_TYPE, EXAMPLE_NAMESPACE};

    use super::AttributeIdentifier;

    impl AttributeIdentifier {
        pub fn new_example_index_set_from_attributes(
            attributes: impl IntoIterator<Item = impl Into<String>>,
        ) -> IndexSet<Self> {
            attributes
                .into_iter()
                .map(|attribute| AttributeIdentifier {
                    credential_type: EXAMPLE_DOC_TYPE.to_string(),
                    namespace: EXAMPLE_NAMESPACE.to_string(),
                    attribute: attribute.into(),
                })
                .collect()
        }
    }
}

#[cfg(any(test, feature = "test"))]
mod tests {
    use super::*;

    #[derive(Debug, thiserror::Error, PartialEq, Eq)]
    pub enum AttributeIdParsingError {
        #[error("Expected string with 3 parts separated by '/', got {0} parts")]
        InvalidPartsCount(usize),
    }

    // This implementation is solely intended for unit testing purposes to easily construct AttributeIdentifiers.
    // This implementation should never end up in production code, because the use of '/' is officially allowed in the
    // various parts.
    impl std::str::FromStr for AttributeIdentifier {
        type Err = AttributeIdParsingError;

        fn from_str(source: &str) -> Result<Self, Self::Err> {
            let parts = source.split('/').collect::<Vec<&str>>();
            if parts.len() != 3 {
                return Err(AttributeIdParsingError::InvalidPartsCount(parts.len()));
            }
            let result = Self {
                credential_type: parts[0].to_owned(),
                namespace: parts[1].to_owned(),
                attribute: parts[2].to_owned(),
            };
            Ok(result)
        }
    }
}
