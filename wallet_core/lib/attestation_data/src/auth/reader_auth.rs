use std::collections::HashMap;
use std::collections::HashSet;

use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use serde_with::skip_serializing_none;
use url::Url;
use x509_parser::der_parser::Oid;
use x509_parser::der_parser::asn1_rs::oid;

use attestation_types::claim_path::ClaimPath;
use crypto::x509::BorrowingCertificateExtension;
use dcql::normalized::NormalizedCredentialRequest;
use error_category::ErrorCategory;
use utils::vec_at_least::VecNonEmpty;

use crate::auth::LocalizedStrings;
use crate::auth::Organization;
use crate::x509::CertificateType;

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum ValidationError {
    #[error("requested unregistered attributes: {}", .0.iter().map(|(attestation_type, paths)| {
        format!("({}): {}", attestation_type, paths.iter().map(|path| {
            format!("[{}]", path.iter().join(", "))
        }).join(", "))
    }).join(" / "))]
    #[category(critical)] // RP data, no user data
    UnregisteredAttributes(HashMap<String, HashSet<VecNonEmpty<ClaimPath>>>),
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
    pub fn verify_requested_attributes<'a>(
        &'a self,
        requests: impl IntoIterator<Item = &'a NormalizedCredentialRequest>,
    ) -> Result<(), ValidationError> {
        let unregistered_attributes = requests
            .into_iter()
            .flat_map(|request| {
                let request_attributes = request.claims.iter().map(|claim| &claim.path).collect::<HashSet<_>>();

                // Check if any of the requested attributes are missing from the
                // authorized attributes for all requested attestation types.
                request.format.credential_types().flat_map(move |attestation_type| {
                    let authorized_attributes = self
                        .authorized_attributes
                        .get(attestation_type)
                        .map(|attributes| attributes.iter().collect::<HashSet<_>>())
                        .unwrap_or_default();

                    let unauthorized_attributes = request_attributes
                        .difference(&authorized_attributes)
                        .copied()
                        .cloned()
                        .collect::<HashSet<_>>();

                    (!unauthorized_attributes.is_empty())
                        .then(|| (attestation_type.to_string(), unauthorized_attributes))
                })
            })
            .collect::<HashMap<_, _>>();

        if !unregistered_attributes.is_empty() {
            return Err(ValidationError::UnregisteredAttributes(unregistered_attributes));
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
            let authorized_attributes = dcql_query.credentials.as_ref().iter().fold(
                HashMap::new(),
                |mut acc: HashMap<String, Vec<VecNonEmpty<ClaimPath>>>, credential_query| {
                    match credential_query.format {
                        CredentialQueryFormat::MsoMdoc { ref doctype_value } => {
                            let claim_paths = match &credential_query.claims_selection {
                                ClaimsSelection::All { claims } => {
                                    claims.as_ref().iter().map(|c| c.path.clone()).collect::<Vec<_>>()
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
    use rstest::rstest;

    use dcql::normalized::NormalizedCredentialRequests;

    use super::mock::*;
    use super::*;

    #[rstest]
    #[case(
        NormalizedCredentialRequests::new_mock_mdoc_from_slices(&[
            (
                "some_doctype",
                &[
                    &["some_namespace", "some_attribute"],
                    &["some_namespace", "another_attribute"],
                    &["another_namespace", "some_attribute"],
                    &["another_namespace", "another_attribute"],
                ]
            ),
            (
                "another_doctype",
                &[
                    &["some_namespace", "some_attribute"],
                    &["some_namespace", "another_attribute"],
                ]
            ),
        ]),
        None
    )]
    #[case(
        NormalizedCredentialRequests::new_mock_sd_jwt_from_slices(&[(
            &["some_doctype", "another_doctype"],
            &[
                &["some_namespace", "some_attribute"],
                &["some_namespace", "another_attribute"],
                &["another_namespace", "some_attribute"],
                &["another_namespace", "another_attribute"],
            ],
        )]),
        None
    )]
    #[case(
        NormalizedCredentialRequests::new_mock_mdoc_from_slices(&[
            (
                "some_doctype",
                &[
                    &["some_namespace", "some_attribute"],
                    &["some_namespace", "missing_attribute"],
                    &["missing_namespace", "some_attribute"],
                    &["missing_namespace", "another_attribute"],
                ]
            ),
            (
                "missing_doctype",
                &[
                    &["some_namespace", "some_attribute"],
                    &["some_namespace", "another_attribute"],
                ]
            ),
        ]),
        Some(HashMap::from([
            (
                "some_doctype".to_string(),
                HashSet::from([
                    vec![
                        ClaimPath::SelectByKey("some_namespace".to_string()),
                        ClaimPath::SelectByKey("missing_attribute".to_string()),
                    ]
                    .try_into()
                    .unwrap(),
                    vec![
                        ClaimPath::SelectByKey("missing_namespace".to_string()),
                        ClaimPath::SelectByKey("some_attribute".to_string()),
                    ]
                    .try_into()
                    .unwrap(),
                    vec![
                        ClaimPath::SelectByKey("missing_namespace".to_string()),
                        ClaimPath::SelectByKey("another_attribute".to_string()),
                    ]
                    .try_into()
                    .unwrap(),
                ]),
            ),
            (
                "missing_doctype".to_string(),
                HashSet::from([
                    vec![
                        ClaimPath::SelectByKey("some_namespace".to_string()),
                        ClaimPath::SelectByKey("some_attribute".to_string()),
                    ]
                    .try_into()
                    .unwrap(),
                    vec![
                        ClaimPath::SelectByKey("some_namespace".to_string()),
                        ClaimPath::SelectByKey("another_attribute".to_string()),
                    ]
                    .try_into()
                    .unwrap(),
                ]),
            ),
        ]))
    )]
    #[case(
        NormalizedCredentialRequests::new_mock_sd_jwt_from_slices(&[(
            &["some_doctype", "missing_doctype"],
            &[
                &["some_namespace", "some_attribute"],
                &["some_namespace", "missing_attribute"],
                &["another_namespace", "some_attribute"],
                &["missing_namespace", "another_attribute"],
            ],
        )]),
        Some(HashMap::from([
            (
                "some_doctype".to_string(),
                HashSet::from([
                    vec![
                        ClaimPath::SelectByKey("some_namespace".to_string()),
                        ClaimPath::SelectByKey("missing_attribute".to_string()),
                    ]
                    .try_into()
                    .unwrap(),
                    vec![
                        ClaimPath::SelectByKey("missing_namespace".to_string()),
                        ClaimPath::SelectByKey("another_attribute".to_string()),
                    ]
                    .try_into()
                    .unwrap(),
                ]),
            ),
            (
                "missing_doctype".to_string(),
                HashSet::from([
                    vec![
                        ClaimPath::SelectByKey("some_namespace".to_string()),
                        ClaimPath::SelectByKey("some_attribute".to_string()),
                    ]
                    .try_into()
                    .unwrap(),
                    vec![
                        ClaimPath::SelectByKey("some_namespace".to_string()),
                        ClaimPath::SelectByKey("missing_attribute".to_string()),
                    ]
                    .try_into()
                    .unwrap(),
                    vec![
                        ClaimPath::SelectByKey("another_namespace".to_string()),
                        ClaimPath::SelectByKey("some_attribute".to_string()),
                    ]
                    .try_into()
                    .unwrap(),
                    vec![
                        ClaimPath::SelectByKey("missing_namespace".to_string()),
                        ClaimPath::SelectByKey("another_attribute".to_string()),
                    ]
                    .try_into()
                    .unwrap(),
                ]),
            ),
        ]))
    )]
    fn test_reader_registration_verify_requested_attributes(
        #[case] requested_attributes: NormalizedCredentialRequests,
        #[case] expected_unregistered: Option<HashMap<String, HashSet<VecNonEmpty<ClaimPath>>>>,
    ) {
        let registration = create_some_registration();
        let result = registration.verify_requested_attributes(requested_attributes.as_ref());

        match expected_unregistered {
            Some(expected_unregistered) => {
                assert_matches!(
                    result.expect_err("verifying requested attributes should fail"),
                    ValidationError::UnregisteredAttributes(unregistered) if unregistered == expected_unregistered
                );
            }
            None => result.expect("verifying requested attributes should succeed"),
        }
    }
}
