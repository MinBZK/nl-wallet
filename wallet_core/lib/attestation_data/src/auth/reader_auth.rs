use std::collections::HashMap;

use indexmap::IndexMap;
use indexmap::IndexSet;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use serde_with::skip_serializing_none;
use url::Url;
use x509_parser::der_parser::Oid;
use x509_parser::der_parser::asn1_rs::oid;

use crypto::x509::BorrowingCertificateExtension;
use dcql::ClaimPath;
use error_category::ErrorCategory;
use mdoc::identifiers::AttributeIdentifier;
use mdoc::identifiers::AttributeIdentifierError;
use mdoc::identifiers::AttributeIdentifierHolder;
use utils::vec_at_least::VecNonEmpty;

use crate::auth::LocalizedStrings;
use crate::auth::Organization;
use crate::x509::CertificateType;

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum ValidationError {
    #[error("requested unregistered attributes: {0:?}")]
    #[category(critical)] // RP data, no user data
    UnregisteredAttributes(Vec<AttributeIdentifier>),

    #[error("Unable to verify the requested attributes: {0}")]
    #[category(critical)]
    RequestedAttributeVerification(#[from] AttributeIdentifierError),
}

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
    pub authorized_attributes: HashMap<String, Vec<VecNonEmpty<ClaimPath>>>,
}

impl ReaderRegistration {
    /// Verify whether all requested attributes exist in the registration.
    pub fn verify_requested_attributes(
        &self,
        requested_attributes: &impl AttributeIdentifierHolder,
    ) -> Result<(), ValidationError> {
        let difference: Vec<AttributeIdentifier> = requested_attributes.difference(self)?.into_iter().collect();

        if !difference.is_empty() {
            return Err(ValidationError::UnregisteredAttributes(difference));
        }

        Ok(())
    }
}

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
    fn mdoc_attribute_identifiers(&self) -> Result<IndexSet<AttributeIdentifier>, AttributeIdentifierError> {
        self.authorized_attributes
            .iter()
            .flat_map(|(doc_type, paths)| {
                paths.iter().map(|paths| {
                    if paths.len().get() != 2 {
                        return Err(AttributeIdentifierError::ExtractFromReaderRegistration {
                            authorized_attributes: paths.clone(),
                        });
                    }

                    let namespace = paths.first().to_string();
                    let attribute = paths.last().to_string();
                    Ok(AttributeIdentifier {
                        credential_type: doc_type.to_owned(),
                        namespace,
                        attribute,
                    })
                })
            })
            .try_collect()
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

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizedMdoc(pub IndexMap<String, AuthorizedNamespace>);

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizedNamespace(pub IndexMap<String, AuthorizedAttribute>);

// This struct could be extended in the future for attribute specific policies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizedAttribute {}

impl BorrowingCertificateExtension for ReaderRegistration {
    /// oid: 2.1.123.1
    /// root: {joint-iso-itu-t(2) asn1(1) examples(123)}
    /// suffix: 1, unofficial id for Reader Authentication
    #[rustfmt::skip]
    const OID: Oid<'static> = oid!(2.1.123.1);
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use itertools::Itertools;

    use dcql::ClaimsSelection;
    use dcql::CredentialQueryFormat;
    use dcql::Query;
    use utils::vec_at_least::VecNonEmpty;

    use super::*;

    pub type Attributes<'a> = Vec<&'a str>;
    pub type Namespaces<'a> = Vec<(&'a str, Attributes<'a>)>;
    pub type DocTypes<'a> = Vec<(&'a str, Namespaces<'a>)>;

    impl ReaderRegistration {
        /// Build attributes for [`ReaderRegistration`] from a list of attributes.
        pub fn create_attributes(
            doc_type: String,
            namespace: &str,
            attributes: impl IntoIterator<Item = impl Into<String>>,
        ) -> HashMap<String, Vec<VecNonEmpty<ClaimPath>>> {
            [(
                doc_type,
                attributes
                    .into_iter()
                    .map(|attribute| {
                        vec![
                            ClaimPath::SelectByKey(String::from(namespace)),
                            ClaimPath::SelectByKey(attribute.into()),
                        ]
                        .try_into()
                        .unwrap()
                    })
                    .collect_vec(),
            )]
            .into()
        }

        pub fn mock_from_dcql_query(dcql_query: &Query) -> Self {
            let authorized_attributes = dcql_query.credentials.iter().fold(
                HashMap::new(),
                |mut acc: HashMap<String, Vec<VecNonEmpty<ClaimPath>>>, credential_query| {
                    match credential_query.format {
                        CredentialQueryFormat::MsoMdoc { ref doctype_value } => {
                            let claim_paths = match &credential_query.claims_selection {
                                ClaimsSelection::All { claims } => {
                                    claims.iter().map(|c| c.path.clone()).collect::<Vec<_>>()
                                }
                                ClaimsSelection::NoSelectivelyDisclosable | ClaimsSelection::Combinations { .. } => {
                                    unimplemented!()
                                }
                            };
                            acc.entry(doctype_value.to_string()).or_default().extend(claim_paths);
                        }
                        CredentialQueryFormat::SdJwt { .. } => todo!("PVW-4139 support SdJwt"),
                    };
                    acc
                },
            );

            Self {
                authorized_attributes,
                ..Self::new_mock()
            }
        }

        pub fn new_mock() -> Self {
            let organization = Organization::new_mock();

            ReaderRegistration {
                purpose_statement: vec![("nl", "Beschrijving van mijn dienst"), ("en", "My Service Description")]
                    .into(),
                retention_policy: RetentionPolicy {
                    intent_to_retain: true,
                    max_duration_in_minutes: Some(60 * 24 * 365),
                },
                sharing_policy: SharingPolicy { intent_to_share: false },
                deletion_policy: DeletionPolicy { deleteable: true },
                organization,
                request_origin_base_url: "https://example.com/".parse().unwrap(),
                authorized_attributes: Default::default(),
            }
        }
    }

    pub fn create_some_registration() -> ReaderRegistration {
        create_registration(vec![
            (
                "some_doctype",
                vec![
                    vec!["some_namespace", "another_attribute"],
                    vec!["some_namespace", "some_attribute"],
                    vec!["another_namespace", "some_attribute"],
                    vec!["another_namespace", "another_attribute"],
                ],
            ),
            (
                "another_doctype",
                vec![
                    vec!["some_namespace", "some_attribute"],
                    vec!["some_namespace", "another_attribute"],
                    vec!["another_namespace", "some_attribute"],
                    vec!["another_namespace", "another_attribute"],
                ],
            ),
        ])
    }

    // Utility function to easily create [`ReaderRegistration`]
    pub fn create_registration(authorized_attributes: Vec<(&str, Vec<Vec<&str>>)>) -> ReaderRegistration {
        let authorized_attributes = HashMap::from_iter(authorized_attributes.into_iter().map(|(name, names)| {
            let paths = names
                .into_iter()
                .map(|paths| {
                    paths
                        .into_iter()
                        .map(|path| ClaimPath::SelectByKey(String::from(path)))
                        .collect_vec()
                        .try_into()
                        .unwrap()
                })
                .collect_vec();
            (name.to_string(), paths)
        }));

        ReaderRegistration {
            authorized_attributes,
            ..ReaderRegistration::new_mock()
        }
    }
}

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;

    use mdoc::identifiers::mock::MockAttributeIdentifierHolder;

    use super::mock::*;
    use super::*;

    #[test]
    fn verify_requested_attributes_in_device_request() {
        let device_request: MockAttributeIdentifierHolder = vec![
            "some_doctype/some_namespace/some_attribute".parse().unwrap(),
            "some_doctype/some_namespace/another_attribute".parse().unwrap(),
            "some_doctype/another_namespace/some_attribute".parse().unwrap(),
            "some_doctype/another_namespace/another_attribute".parse().unwrap(),
            "another_doctype/some_namespace/some_attribute".parse().unwrap(),
            "another_doctype/some_namespace/another_attribute".parse().unwrap(),
        ]
        .into();
        let registration = create_some_registration();
        registration.verify_requested_attributes(&device_request).unwrap();
    }

    #[test]
    fn verify_requested_attributes_in_device_request_missing() {
        let device_request: MockAttributeIdentifierHolder = vec![
            "some_doctype/some_namespace/some_attribute".parse().unwrap(),
            "some_doctype/some_namespace/missing_attribute".parse().unwrap(),
            "some_doctype/missing_namespace/some_attribute".parse().unwrap(),
            "some_doctype/missing_namespace/another_attribute".parse().unwrap(),
            "missing_doctype/some_namespace/some_attribute".parse().unwrap(),
            "missing_doctype/some_namespace/another_attribute".parse().unwrap(),
        ]
        .into();
        let registration = create_some_registration();
        let result = registration.verify_requested_attributes(&device_request);
        assert_matches!(
            result,
            Err(ValidationError::UnregisteredAttributes(attrs)) if attrs == vec![
                "some_doctype/some_namespace/missing_attribute".parse().unwrap(),
                "some_doctype/missing_namespace/some_attribute".parse().unwrap(),
                "some_doctype/missing_namespace/another_attribute".parse().unwrap(),
                "missing_doctype/some_namespace/some_attribute".parse().unwrap(),
                "missing_doctype/some_namespace/another_attribute".parse().unwrap(),
            ]
        );
    }

    #[test]
    fn attribute_identifiers_for_single_claimpath_should_err_for_mdoc() {
        let registration = create_registration(vec![("some_doctype", vec![vec!["some_attribute"]])]);
        assert!(registration.mdoc_attribute_identifiers().is_err());
    }

    #[test]
    fn validate_items_request() {
        let request: MockAttributeIdentifierHolder = vec![
            "some_doctype/some_namespace/some_attribute".parse().unwrap(),
            "some_doctype/some_namespace/another_attribute".parse().unwrap(),
        ]
        .into();
        let registration = create_some_registration();
        registration.verify_requested_attributes(&request).unwrap();
    }

    #[test]
    fn validate_items_request_missing_attribute() {
        let request: MockAttributeIdentifierHolder = vec![
            "some_doctype/some_namespace/missing_attribute".parse().unwrap(),
            "some_doctype/some_namespace/another_attribute".parse().unwrap(),
        ]
        .into();
        let registration = create_registration(vec![(
            "some_doctype",
            vec![
                vec!["some_namespace", "some_attribute"],
                vec!["some_namespace", "another_attribute"],
            ],
        )]);

        let result = registration.verify_requested_attributes(&request);
        assert_matches!(result, Err(ValidationError::UnregisteredAttributes(attrs)) if attrs == vec![
            "some_doctype/some_namespace/missing_attribute".parse().unwrap(),
        ]);
    }

    #[test]
    fn validate_items_request_missing_namespace() {
        let request: MockAttributeIdentifierHolder = vec![
            "some_doctype/missing_namespace/some_attribute".parse().unwrap(),
            "some_doctype/missing_namespace/another_attribute".parse().unwrap(),
        ]
        .into();
        let registration = create_registration(vec![(
            "some_doctype",
            vec![
                vec!["some_namespace", "some_attribute"],
                vec!["some_namespace", "another_attribute"],
            ],
        )]);

        let result = registration.verify_requested_attributes(&request);
        assert_matches!(result, Err(ValidationError::UnregisteredAttributes(attrs)) if attrs == vec![
            "some_doctype/missing_namespace/some_attribute".parse().unwrap(),
            "some_doctype/missing_namespace/another_attribute".parse().unwrap(),
        ]);
    }

    #[test]
    fn validate_items_request_missing_doctype() {
        let request: MockAttributeIdentifierHolder = vec![
            "missing_doctype/some_namespace/some_attribute".parse().unwrap(),
            "missing_doctype/some_namespace/another_attribute".parse().unwrap(),
        ]
        .into();
        let registration = create_registration(vec![(
            "some_doctype",
            vec![
                vec!["some_namespace", "some_attribute"],
                vec!["some_namespace", "another_attribute"],
            ],
        )]);

        let result = registration.verify_requested_attributes(&request);
        assert_matches!(result, Err(ValidationError::UnregisteredAttributes(attrs)) if attrs == vec![
            "missing_doctype/some_namespace/some_attribute".parse().unwrap(),
            "missing_doctype/some_namespace/another_attribute".parse().unwrap(),
        ]);
    }
}
