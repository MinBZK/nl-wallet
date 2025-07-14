use derive_more::Display;
use indexmap::IndexSet;

use attestation_types::request::AttributeRequest;
use attestation_types::request::MdocCredentialRequestError;
use attestation_types::request::NormalizedCredentialRequest;
use dcql::CredentialQueryFormat;
use error_category::ErrorCategory;
use sd_jwt_vc_metadata::ClaimPath;
use utils::vec_at_least::VecNonEmpty;

use crate::DataElementIdentifier;
use crate::Document;
use crate::NameSpace;

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum AttributeIdentifierError {
    #[error("unable to extract attribute identifiers from reader registration: {authorized_attributes:?}")]
    #[category(critical)]
    ExtractFromReaderRegistration {
        authorized_attributes: VecNonEmpty<ClaimPath>,
    },
}

#[derive(Debug, Display, PartialEq, Eq, Hash, Clone)]
#[display("{credential_type}/{namespace}/{attribute}")]
pub struct AttributeIdentifier {
    pub credential_type: String,
    pub namespace: NameSpace,
    pub attribute: DataElementIdentifier,
}

impl AttributeIdentifier {
    pub fn from_attribute_request(
        doc_type: &str,
        attribute_request: &AttributeRequest,
    ) -> Result<Self, MdocCredentialRequestError> {
        let (namespace, attribute) = attribute_request.to_namespace_and_attribute()?;

        let identifier = Self {
            credential_type: doc_type.to_owned(),
            namespace: namespace.to_owned(),
            attribute: attribute.to_owned(),
        };

        Ok(identifier)
    }
}

pub trait AttributeIdentifierHolder {
    fn mdoc_attribute_identifiers(&self) -> Result<IndexSet<AttributeIdentifier>, AttributeIdentifierError>;

    fn difference(
        &self,
        other: &impl AttributeIdentifierHolder,
    ) -> Result<IndexSet<AttributeIdentifier>, AttributeIdentifierError> {
        let other_attributes = other.mdoc_attribute_identifiers()?;
        Ok(self
            .mdoc_attribute_identifiers()?
            .into_iter()
            .filter(|attribute| !other_attributes.contains(attribute))
            .collect())
    }

    /// Returns requested attributes, if any, that are not present in the `issuer_signed`.
    fn match_against_issuer_signed(
        &self,
        document: &Document,
    ) -> Result<Vec<AttributeIdentifier>, AttributeIdentifierError> {
        let document_identifiers = document.issuer_signed_attribute_identifiers();
        Ok(self
            .mdoc_attribute_identifiers()?
            .into_iter()
            .filter(|attribute| !document_identifiers.contains(attribute))
            .collect())
    }
}

impl<T> AttributeIdentifierHolder for &[T]
where
    T: AttributeIdentifierHolder,
{
    fn mdoc_attribute_identifiers(&self) -> Result<IndexSet<AttributeIdentifier>, AttributeIdentifierError> {
        self.iter().try_fold(IndexSet::new(), |mut acc, holder| {
            let mut identifiers = holder.mdoc_attribute_identifiers()?;
            acc.append(&mut identifiers);
            Ok(acc)
        })
    }
}

impl AttributeIdentifierHolder for NormalizedCredentialRequest {
    fn mdoc_attribute_identifiers(&self) -> Result<IndexSet<AttributeIdentifier>, AttributeIdentifierError> {
        let CredentialQueryFormat::MsoMdoc { doctype_value } = &self.format else {
            // This function should never be called for an sd-jwt request, as this is mdoc specific
            panic!("sd-jwt not supported");
        };
        Ok(self
            .claims
            .iter()
            .map(|claim| AttributeIdentifier::from_attribute_request(doctype_value, claim).unwrap())
            .collect())
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
    use super::AttributeIdentifierError;
    use super::AttributeIdentifierHolder;

    pub fn some_attr() -> AttributeIdentifier {
        AttributeIdentifier {
            credential_type: "some_doc".to_string(),
            namespace: "some_ns".to_string(),
            attribute: "some_attr".to_string(),
        }
    }

    pub fn another_attr() -> AttributeIdentifier {
        AttributeIdentifier {
            credential_type: "some_doc".to_string(),
            namespace: "some_ns".to_string(),
            attribute: "another_attr".to_string(),
        }
    }

    pub fn another_namespace() -> AttributeIdentifier {
        AttributeIdentifier {
            credential_type: "some_doc".to_string(),
            namespace: "another_ns".to_string(),
            attribute: "some_attr".to_string(),
        }
    }

    pub fn another_doctype() -> AttributeIdentifier {
        AttributeIdentifier {
            credential_type: "another_doc".to_string(),
            namespace: "some_ns".to_string(),
            attribute: "some_attr".to_string(),
        }
    }

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
        fn mdoc_attribute_identifiers(&self) -> Result<IndexSet<AttributeIdentifier>, AttributeIdentifierError> {
            Ok(self.0.clone())
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

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use attestation_types::request::MdocCredentialRequestError;
    use attestation_types::request::NormalizedCredentialRequest;
    use attestation_types::request::self;
    use dcql::ClaimPath;
    use utils::vec_at_least::VecNonEmpty;

    use super::mock::*;
    use super::AttributeIdentifier;
    use super::AttributeIdentifierHolder;

    #[rstest]
    #[case(
        vec![ClaimPath::SelectByKey("ns".to_string())].try_into().unwrap(),
        Err(MdocCredentialRequestError::UnexpectedClaimsPathAmount(1.try_into().unwrap()))
    )]
    #[case(
        vec![
            ClaimPath::SelectByKey("ns".to_string()),
            ClaimPath::SelectByKey("attr".to_string())
        ].try_into().unwrap(),
        Ok("doc/ns/attr".parse().unwrap())
    )]
    fn test_from_attribute_request(
        #[case] path: VecNonEmpty<ClaimPath>,
        #[case] expected: Result<AttributeIdentifier, MdocCredentialRequestError>,
    ) {
        let actual = AttributeIdentifier::from_attribute_request(
            "doc",
            &request::AttributeRequest {
                path,
                intent_to_retain: false,
            },
        );
        assert_eq!(actual, expected);
    }

    #[rstest]
    #[case(
        vec![].into(),
        vec![some_attr(), another_attr(), another_namespace(), another_doctype()].into(),
        vec![].into(),
    )]
    #[case(
        vec![some_attr(), another_attr(), another_namespace(), another_doctype()].into(),
        vec![].into(),
        vec![some_attr(), another_attr(), another_namespace(), another_doctype()].into(),
    )]
    #[case(
        vec![some_attr(), another_attr(), another_namespace(), another_doctype()].into(),
        vec![some_attr()].into(),
        vec![another_attr(), another_namespace(), another_doctype()].into(),
    )]
    #[case(
        vec![some_attr(), another_attr(), another_namespace(), another_doctype()].into(),
        vec![another_attr()].into(),
        vec![some_attr(), another_namespace(), another_doctype()].into(),
    )]
    #[case(
        vec![some_attr(), another_attr(), another_namespace(), another_doctype()].into(),
        vec![another_namespace()].into(),
        vec![some_attr(), another_attr(), another_doctype()].into(),
    )]
    #[case(
        vec![some_attr(), another_attr(), another_namespace(), another_doctype()].into(),
        vec![another_doctype()].into(),
        vec![some_attr(), another_attr(), another_namespace()].into(),
    )]
    #[case(
        vec![some_attr(), another_attr(), ].into(),
        vec![another_attr(), another_namespace()].into(),
        vec![some_attr()].into(),
    )]
    fn test_attribute_identifier_holder_difference(
        #[case] a: MockAttributeIdentifierHolder,
        #[case] b: MockAttributeIdentifierHolder,
        #[case] expected: MockAttributeIdentifierHolder,
    ) {
        use super::AttributeIdentifierHolder;

        let difference = a.difference(&b);
        assert_eq!(difference.unwrap(), expected.mdoc_attribute_identifiers().unwrap())
    }

    #[rstest]
    #[case(
        NormalizedCredentialRequest::pid_full_name(),
        vec![
            "urn:eudi:pid:nl:1/urn:eudi:pid:nl:1/family_name".parse().unwrap(),
            "urn:eudi:pid:nl:1/urn:eudi:pid:nl:1/given_name".parse().unwrap(),
        ].into(),
    )]
    #[case(
        NormalizedCredentialRequest::addr_street(),
        vec![
            "urn:eudi:pid-address:nl:1/urn:eudi:pid-address:nl:1.address/street_address".parse().unwrap(),
        ].into(),
    )]
    fn test_normalized_credential_request(
        #[case] input: NormalizedCredentialRequest,
        #[case] expected: MockAttributeIdentifierHolder,
    ) {
        let actual = input.mdoc_attribute_identifiers();
        assert_eq!(actual.unwrap(), expected.mdoc_attribute_identifiers().unwrap());
    }
}
