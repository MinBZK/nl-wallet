use indexmap::IndexMap;
use indexmap::IndexSet;
use serde::Deserialize;
use serde::Serialize;
use serde_with::skip_serializing_none;
use url::Url;
use x509_parser::der_parser::asn1_rs::oid;
use x509_parser::der_parser::Oid;

use crypto::x509::BorrowingCertificateExtension;

use crate::auth::LocalizedStrings;
use crate::auth::Organization;
use crate::identifiers::AttributeIdentifier;
use crate::identifiers::AttributeIdentifierHolder;
use crate::x509::CertificateType;

#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReaderRegistration {
    pub purpose_statement: LocalizedStrings,
    pub retention_policy: RetentionPolicy,
    pub sharing_policy: SharingPolicy,
    pub deletion_policy: DeletionPolicy,
    pub organization: Organization,
    /// Origin base url, for visual user inspection
    pub request_origin_base_url: Url,
    pub attributes: IndexMap<String, AuthorizedMdoc>,
}

// TODO: ensure that `mdoc` contains a replacement for this
// impl ReaderRegistration {
//     /// Verify whether all requested attributes exist in the registration.
//     pub fn verify_requested_attributes<'a, R>(&self, requested_attributes: &R) -> Result<(), ValidationError>
//     where
//         R: IntoIterator<Item = &'a ItemsRequest> + Clone,
//     {
//         let difference: Vec<AttributeIdentifier> = requested_attributes.difference(self).into_iter().collect();

//         if !difference.is_empty() {
//             return Err(ValidationError::UnregisteredAttributes(difference));
//         }

//         Ok(())
//     }
// }

#[cfg(feature = "generate")]
impl TryFrom<ReaderRegistration> for Vec<rcgen::CustomExtension> {
    type Error = crypto::x509::CertificateError;

    fn try_from(value: ReaderRegistration) -> Result<Self, Self::Error> {
        let certificate_type = CertificateType::from(value);
        let result = certificate_type.try_into()?;
        Ok(result)
    }
}

impl AttributeIdentifierHolder for ReaderRegistration {
    fn attribute_identifiers(&self) -> IndexSet<AttributeIdentifier> {
        self.attributes
            .iter()
            .flat_map(|(doc_type, AuthorizedMdoc(namespaces))| {
                namespaces
                    .into_iter()
                    .flat_map(|(namespace, AuthorizedNamespace(attributes))| {
                        attributes.into_iter().map(|(attribute, _)| AttributeIdentifier {
                            credential_type: doc_type.to_owned(),
                            namespace: namespace.to_owned(),
                            attribute: attribute.to_owned(),
                        })
                    })
            })
            .collect()
    }
}

impl From<ReaderRegistration> for CertificateType {
    fn from(source: ReaderRegistration) -> Self {
        CertificateType::ReaderAuth(Box::new(source).into())
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetentionPolicy {
    pub intent_to_retain: bool,
    pub max_duration_in_minutes: Option<u64>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SharingPolicy {
    pub intent_to_share: bool,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletionPolicy {
    pub deleteable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizedMdoc(pub IndexMap<String, AuthorizedNamespace>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizedNamespace(pub IndexMap<String, AuthorizedAttribute>);

// This struct could be extended in the future for attribute specific policies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizedAttribute {}

impl BorrowingCertificateExtension for ReaderRegistration {
    /// oid: 2.1.123.1
    /// root: {joint-iso-itu-t(2) asn1(1) examples(123)}
    /// suffix: 1, unofficial id for Reader Authentication
    const OID: Oid<'static> = oid!(2.1.123 .1);
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use super::*;

    impl ReaderRegistration {
        pub fn new_mock() -> Self {
            let organization = Organization::new_mock();

            ReaderRegistration {
                purpose_statement: vec![("nl", "Beschrijving van mijn dienst"), ("en", "My Service Description")]
                    .into(),
                retention_policy: RetentionPolicy {
                    intent_to_retain: true,
                    max_duration_in_minutes: Some(60 * 24 * 365),
                },
                sharing_policy: SharingPolicy { intent_to_share: true },
                deletion_policy: DeletionPolicy { deleteable: true },
                organization,
                request_origin_base_url: "https://example.com/".parse().unwrap(),
                attributes: Default::default(),
            }
        }
    }
}

#[cfg(feature = "test")]
pub mod test {
    use indexmap::IndexMap;

    use super::*;

    pub type Attributes<'a> = Vec<&'a str>;
    pub type Namespaces<'a> = Vec<(&'a str, Attributes<'a>)>;
    pub type DocTypes<'a> = Vec<(&'a str, Namespaces<'a>)>;

    impl ReaderRegistration {
        /// Build attributes for [`ReaderRegistration`] from a list of attributes.
        pub fn create_attributes(
            doc_type: String,
            name_space: String,
            attributes: impl Iterator<Item = impl Into<String>>,
        ) -> IndexMap<String, AuthorizedMdoc> {
            [(
                doc_type,
                AuthorizedMdoc(
                    [(
                        name_space,
                        AuthorizedNamespace(
                            attributes
                                .map(|attribute| (attribute.into(), AuthorizedAttribute {}))
                                .collect(),
                        ),
                    )]
                    .into(),
                ),
            )]
            .into()
        }
    }

    pub fn create_some_registration() -> ReaderRegistration {
        create_registration(vec![
            (
                "some_doctype",
                vec![
                    ("some_namespace", vec!["some_attribute", "another_attribute"]),
                    ("another_namespace", vec!["some_attribute", "another_attribute"]),
                ],
            ),
            (
                "another_doctype",
                vec![
                    ("some_namespace", vec!["some_attribute", "another_attribute"]),
                    ("another_namespace", vec!["some_attribute", "another_attribute"]),
                ],
            ),
        ])
    }

    // Utility function to easily create [`ReaderRegistration`]
    pub fn create_registration(registered_doctypes: DocTypes) -> ReaderRegistration {
        let mut attributes = IndexMap::new();
        for (doc_type, namespaces) in registered_doctypes {
            let mut namespace_map = IndexMap::new();
            for (ns, attrs) in namespaces {
                let mut attribute_map = IndexMap::new();
                for attr in attrs {
                    attribute_map.insert(attr.to_owned(), AuthorizedAttribute {});
                }
                namespace_map.insert(ns.to_owned(), AuthorizedNamespace(attribute_map));
            }
            attributes.insert(doc_type.to_owned(), AuthorizedMdoc(namespace_map));
        }

        ReaderRegistration {
            attributes,
            ..ReaderRegistration::new_mock()
        }
    }
}
