use derive_more::Display;
use indexmap::IndexSet;

use crate::DataElementIdentifier;
use crate::Document;
use crate::NameSpace;

#[derive(Debug, Display, PartialEq, Eq, Hash, Clone)]
#[display("{credential_type}/{namespace}/{attribute}")]
pub struct AttributeIdentifier {
    pub credential_type: String,
    pub namespace: NameSpace,
    pub attribute: DataElementIdentifier,
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

    /// Returns requested attributes, if any, that are not present in the `issuer_signed`.
    fn match_against_issuer_signed(&self, document: &Document) -> Vec<AttributeIdentifier> {
        let document_identifiers = document.issuer_signed_attribute_identifiers();
        self.attribute_identifiers()
            .into_iter()
            .filter(|attribute| !document_identifiers.contains(attribute))
            .collect()
    }
}

impl<'a, I, T> AttributeIdentifierHolder for I
where
    I: IntoIterator<Item = &'a T> + Clone,
    T: AttributeIdentifierHolder + 'a,
{
    fn attribute_identifiers(&self) -> IndexSet<AttributeIdentifier> {
        self.clone()
            .into_iter()
            .flat_map(AttributeIdentifierHolder::attribute_identifiers)
            .collect()
    }
}

#[cfg(any(test, feature = "examples"))]
mod examples {
    use indexmap::IndexSet;

    use crate::identifiers::AttributeIdentifier;

    pub const EXAMPLE_DOC_TYPE: &str = "org.iso.18013.5.1.mDL";
    pub const EXAMPLE_NAMESPACE: &str = "org.iso.18013.5.1";

    impl AttributeIdentifier {
        pub fn new_index_set_from_attributes_doc_type_and_namespace(
            credential_type: &str,
            namespace: &str,
            attributes: impl IntoIterator<Item = impl Into<String>>,
        ) -> IndexSet<Self> {
            attributes
                .into_iter()
                .map(|attribute| AttributeIdentifier {
                    credential_type: credential_type.to_owned(),
                    namespace: namespace.to_owned(),
                    attribute: attribute.into(),
                })
                .collect()
        }

        pub fn new_example_index_set_from_attributes(
            attributes: impl IntoIterator<Item = impl Into<String>>,
        ) -> IndexSet<Self> {
            Self::new_index_set_from_attributes_doc_type_and_namespace(EXAMPLE_DOC_TYPE, EXAMPLE_NAMESPACE, attributes)
        }
    }
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use indexmap::IndexSet;

    use super::AttributeIdentifier;
    use super::AttributeIdentifierHolder;

    #[derive(Debug, thiserror::Error, PartialEq, Eq)]
    pub enum AttributeIdParsingError {
        #[error("Expected string with 3 parts separated by '/', got {0} parts")]
        InvalidPartsCount(usize),
    }

    pub struct MockAttributeIdentifierHolder(IndexSet<AttributeIdentifier>);

    impl From<Vec<AttributeIdentifier>> for MockAttributeIdentifierHolder {
        fn from(value: Vec<AttributeIdentifier>) -> Self {
            Self(value.into_iter().collect())
        }
    }

    impl AttributeIdentifierHolder for MockAttributeIdentifierHolder {
        fn attribute_identifiers(&self) -> IndexSet<AttributeIdentifier> {
            self.0.clone()
        }
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
