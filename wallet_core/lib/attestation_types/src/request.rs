use std::num::NonZero;

use serde::Deserialize;
use serde::Serialize;

use dcql::ClaimPath;
use dcql::CredentialQueryFormat;
use utils::vec_at_least::VecNonEmpty;

/// Request for a credential.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedCredentialRequest {
    pub format: CredentialQueryFormat,
    pub claims: Vec<AttributeRequest>,
}

/// Request for a single attribute with the given [path].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttributeRequest {
    pub path: VecNonEmpty<ClaimPath>,
    pub intent_to_retain: bool,
}

#[derive(Debug, thiserror::Error)]
#[cfg_attr(any(test, feature = "mock"), derive(PartialEq, Eq))]
pub enum MdocCredentialRequestError {
    #[error("unexpected amount of claim paths: expected 2, found {0}")]
    UnexpectedClaimsPathAmount(NonZero<usize>),
    #[error("unexpected claim path type, expected key string")]
    UnexpectedClaimsPathType,
    #[error("sd-jwt is not supported here")]
    SdJwtNotSupported,
}

impl AttributeRequest {
    pub fn to_namespace_and_attribute(&self) -> Result<(&str, &str), MdocCredentialRequestError> {
        if self.path.len().get() != 2 {
            return Err(MdocCredentialRequestError::UnexpectedClaimsPathAmount(self.path.len()));
        }
        let ClaimPath::SelectByKey(namespace) = &self.path[0] else {
            return Err(MdocCredentialRequestError::UnexpectedClaimsPathType);
        };
        let ClaimPath::SelectByKey(attribute) = &self.path[1] else {
            return Err(MdocCredentialRequestError::UnexpectedClaimsPathType);
        };
        Ok((namespace, attribute))
    }
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use dcql::ClaimPath;
    use dcql::CredentialQueryFormat;
    use utils::vec_at_least::VecNonEmpty;

    use super::AttributeRequest;
    use super::NormalizedCredentialRequest;

    pub const EXAMPLE_DOC_TYPE: &str = "org.iso.18013.5.1.mDL";
    pub const EXAMPLE_NAMESPACE: &str = "org.iso.18013.5.1";
    pub const ATTR_BSN: &str = "bsn";
    pub const ATTR_FAMILY_NAME: &str = "family_name";
    pub const ATTR_GIVEN_NAME: &str = "given_name";
    pub const ATTR_STREET_ADDRESS: &str = "street_address";
    pub const ATTR_ISSUE_DATE: &str = "issue_date";
    pub const ATTR_EXPIRY_DATE: &str = "expiry_date";
    pub const ATTR_DOCUMENT_NUMBER: &str = "document_number";
    pub const ATTR_PORTRAIT: &str = "portrait";
    pub const ATTR_DRIVING_PRIVILEGES: &str = "driving_privileges";

    pub const PID: &str = "urn:eudi:pid:nl:1";
    pub const ADDR: &str = "urn:eudi:pid-address:nl:1";
    pub const ADDR_NS: &str = "urn:eudi:pid-address:nl:1.address";

    impl AttributeRequest {
        pub fn new_with_keys(keys: Vec<String>, intent_to_retain: bool) -> Self {
            Self {
                path: VecNonEmpty::try_from(keys.into_iter().map(ClaimPath::SelectByKey).collect::<Vec<_>>()).unwrap(),
                intent_to_retain,
            }
        }
    }

    impl NormalizedCredentialRequest {
        pub fn new_example() -> Self {
            Self {
                format: CredentialQueryFormat::MsoMdoc {
                    doctype_value: EXAMPLE_DOC_TYPE.to_string(),
                },
                claims: vec![AttributeRequest::new_with_keys(
                    vec![EXAMPLE_NAMESPACE.to_string(), ATTR_FAMILY_NAME.to_string()],
                    true,
                )],
            }
        }

        pub fn new_pid_example() -> Self {
            Self {
                format: CredentialQueryFormat::MsoMdoc {
                    doctype_value: PID.to_string(),
                },
                claims: vec![
                    AttributeRequest::new_with_keys(vec![PID.to_string(), ATTR_BSN.to_string()], false),
                    AttributeRequest::new_with_keys(vec![PID.to_string(), ATTR_GIVEN_NAME.to_string()], false),
                    AttributeRequest::new_with_keys(vec![PID.to_string(), ATTR_FAMILY_NAME.to_string()], false),
                ],
            }
        }

        pub fn pid_full_name() -> Self {
            Self {
                format: CredentialQueryFormat::MsoMdoc {
                    doctype_value: PID.to_string(),
                },
                claims: vec![
                    AttributeRequest::new_with_keys(vec![PID.to_string(), ATTR_FAMILY_NAME.to_string()], true),
                    AttributeRequest::new_with_keys(vec![PID.to_string(), ATTR_GIVEN_NAME.to_string()], true),
                ],
            }
        }

        pub fn addr_street() -> Self {
            Self {
                format: CredentialQueryFormat::MsoMdoc {
                    doctype_value: ADDR.to_string(),
                },
                claims: vec![AttributeRequest::new_with_keys(
                    vec![ADDR_NS.to_string(), ATTR_STREET_ADDRESS.to_string()],
                    true,
                )],
            }
        }
    }

    pub fn new_pid_example() -> VecNonEmpty<NormalizedCredentialRequest> {
        vec![NormalizedCredentialRequest::new_pid_example()].try_into().unwrap()
    }

    pub fn mock_from_vecs(input: Vec<(String, Vec<VecNonEmpty<String>>)>) -> VecNonEmpty<NormalizedCredentialRequest> {
        let requests: Vec<_> = input
            .into_iter()
            .map(|(doc_type, paths)| {
                let format = CredentialQueryFormat::MsoMdoc {
                    doctype_value: doc_type.to_string(),
                };
                let claims = paths
                    .into_iter()
                    .map(|path| {
                        let claim_path: Vec<_> = path
                            .into_iter()
                            .map(|element| ClaimPath::SelectByKey(element.to_string()))
                            .collect();
                        AttributeRequest {
                            path: VecNonEmpty::try_from(claim_path).expect("empy path not allowed"),
                            intent_to_retain: false,
                        }
                    })
                    .collect();
                NormalizedCredentialRequest { format, claims }
            })
            .collect();
        requests.try_into().expect("should contain at least 1 request")
    }

    pub fn example() -> VecNonEmpty<NormalizedCredentialRequest> {
        vec![NormalizedCredentialRequest {
            format: CredentialQueryFormat::MsoMdoc {
                doctype_value: EXAMPLE_DOC_TYPE.to_string(),
            },
            claims: vec![
                AttributeRequest::new_with_keys(
                    vec![EXAMPLE_NAMESPACE.to_string(), ATTR_FAMILY_NAME.to_string()],
                    false,
                ),
                AttributeRequest::new_with_keys(
                    vec![EXAMPLE_NAMESPACE.to_string(), ATTR_ISSUE_DATE.to_string()],
                    false,
                ),
                AttributeRequest::new_with_keys(
                    vec![EXAMPLE_NAMESPACE.to_string(), ATTR_EXPIRY_DATE.to_string()],
                    false,
                ),
                AttributeRequest::new_with_keys(
                    vec![EXAMPLE_NAMESPACE.to_string(), ATTR_DOCUMENT_NUMBER.to_string()],
                    false,
                ),
                AttributeRequest::new_with_keys(vec![EXAMPLE_NAMESPACE.to_string(), ATTR_PORTRAIT.to_string()], false),
                AttributeRequest::new_with_keys(
                    vec![EXAMPLE_NAMESPACE.to_string(), ATTR_DRIVING_PRIVILEGES.to_string()],
                    false,
                ),
            ],
        }]
        .try_into()
        .unwrap()
    }
}

#[cfg(test)]
mod test {
    use rstest::rstest;

    use dcql::ClaimPath;
    use utils::vec_at_least::VecNonEmpty;

    use super::mock::ATTR_FAMILY_NAME;
    use super::mock::ATTR_GIVEN_NAME;
    use super::mock::EXAMPLE_NAMESPACE;
    use super::AttributeRequest;
    use super::MdocCredentialRequestError;

    #[rstest]
    #[case(
        vec![
            ClaimPath::SelectByKey(EXAMPLE_NAMESPACE.to_string()),
            ClaimPath::SelectByKey(ATTR_FAMILY_NAME.to_string())].try_into().unwrap(),
        Ok((EXAMPLE_NAMESPACE, ATTR_FAMILY_NAME))
    )]
    #[case(
        vec![ClaimPath::SelectByKey(EXAMPLE_NAMESPACE.to_string())].try_into().unwrap(),
        Err(MdocCredentialRequestError::UnexpectedClaimsPathAmount(1.try_into().unwrap()))
    )]
    #[case(
        vec![
            ClaimPath::SelectByKey(EXAMPLE_NAMESPACE.to_string()),
            ClaimPath::SelectByKey(ATTR_FAMILY_NAME.to_string()),
            ClaimPath::SelectByKey(ATTR_GIVEN_NAME.to_string())
        ].try_into().unwrap(),
        Err(MdocCredentialRequestError::UnexpectedClaimsPathAmount(3.try_into().unwrap()))
    )]
    #[case(
        vec![
            ClaimPath::SelectByKey(EXAMPLE_NAMESPACE.to_string()),
            ClaimPath::SelectByIndex(1)].try_into().unwrap(),
        Err(MdocCredentialRequestError::UnexpectedClaimsPathType)
    )]
    #[case(
        vec![
            ClaimPath::SelectAll,
            ClaimPath::SelectByKey(ATTR_FAMILY_NAME.to_string())].try_into().unwrap(),
        Err(MdocCredentialRequestError::UnexpectedClaimsPathType)
    )]
    fn test_to_namespace_and_attribute(
        #[case] claim_paths: VecNonEmpty<ClaimPath>,
        #[case] expected: Result<(&str, &str), MdocCredentialRequestError>,
    ) {
        let test_subject = AttributeRequest {
            path: claim_paths,
            intent_to_retain: false,
        };
        let actual = test_subject.to_namespace_and_attribute();
        assert_eq!(actual, expected);
    }
}
