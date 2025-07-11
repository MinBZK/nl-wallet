use std::num::NonZero;

use nutype::nutype;
use serde::Deserialize;
use serde::Serialize;

use dcql::ClaimPath;
use dcql::CredentialQueryFormat;
use utils::vec_at_least::VecNonEmpty;

#[nutype(
    derive(Debug, Clone, PartialEq, Eq, AsRef, TryFrom, Into, IntoIterator, Serialize, Deserialize),
    validate(predicate = |items| !items.is_empty()),
)]
pub struct NormalizedCredentialRequests(Vec<NormalizedCredentialRequest>);

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
    #[error("unexpected amount of claim paths: {0}")]
    UnexpectedClaimsPathAmount(NonZero<usize>),
    #[error("unexpected claim path type, expected key string")]
    UnexpectedClaimsPathType,
    #[error("sd-jwt is not supported here")]
    SdJwtNotSupported,
}

impl AttributeRequest {
    pub fn to_namespace_and_attribute(&self) -> Result<(&str, &str), MdocCredentialRequestError> {
        if self.path.len() != 2.try_into().unwrap() {
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

#[cfg(test)]
mod test {
    use rstest::rstest;

    use dcql::ClaimPath;
    use utils::vec_at_least::VecNonEmpty;

    use super::{AttributeRequest, MdocCredentialRequestError};

    #[rstest]
    #[case(
        vec![
            ClaimPath::SelectByKey("namespace".to_string()),
            ClaimPath::SelectByKey("attr".to_string())].try_into().unwrap(),
        Ok(("namespace", "attr"))
    )]
    #[case(
        vec![ClaimPath::SelectByKey("namespace".to_string())].try_into().unwrap(),
        Err(MdocCredentialRequestError::UnexpectedClaimsPathAmount(1.try_into().unwrap()))
    )]
    #[case(
        vec![
            ClaimPath::SelectByKey("namespace".to_string()),
            ClaimPath::SelectByKey("addr".to_string()),
            ClaimPath::SelectByKey("street".to_string())
        ].try_into().unwrap(),
        Err(MdocCredentialRequestError::UnexpectedClaimsPathAmount(3.try_into().unwrap()))
    )]
    #[case(
        vec![
            ClaimPath::SelectByKey("namespace".to_string()),
            ClaimPath::SelectByIndex(1)].try_into().unwrap(),
        Err(MdocCredentialRequestError::UnexpectedClaimsPathType)
    )]
    #[case(
        vec![
            ClaimPath::SelectAll,
            ClaimPath::SelectByKey("attr".to_string())].try_into().unwrap(),
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

#[cfg(any(test, feature = "mock"))]
mod mock {
    use dcql::ClaimPath;
    use dcql::CredentialQueryFormat;
    use utils::vec_at_least::VecNonEmpty;

    use super::AttributeRequest;
    use super::NormalizedCredentialRequest;
    use super::NormalizedCredentialRequests;

    pub const EXAMPLE_DOC_TYPE: &str = "org.iso.18013.5.1.mDL";
    pub const EXAMPLE_NAMESPACE: &str = "org.iso.18013.5.1";
    pub const EXAMPLE_ATTR_NAME: &str = "family_name";

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
                    vec![EXAMPLE_NAMESPACE.to_string(), EXAMPLE_ATTR_NAME.to_string()],
                    true,
                )],
            }
        }

        pub fn new_pid_example() -> Self {
            // unwraps below are safe because claims path is not empty
            Self {
                format: CredentialQueryFormat::MsoMdoc {
                    doctype_value: PID.to_string(),
                },
                claims: vec![
                    AttributeRequest::new_with_keys(vec![PID.to_string(), "bsn".to_string()], false),
                    AttributeRequest::new_with_keys(vec![PID.to_string(), "given_name".to_string()], false),
                    AttributeRequest::new_with_keys(vec![PID.to_string(), "family_name".to_string()], false),
                ],
            }
        }

        pub fn pid_full_name() -> Self {
            // unwraps below are safe because claims path is not empty
            Self {
                format: CredentialQueryFormat::MsoMdoc {
                    doctype_value: PID.to_string(),
                },
                claims: vec![
                    AttributeRequest::new_with_keys(vec![PID.to_string(), "family_name".to_string()], true),
                    AttributeRequest::new_with_keys(vec![PID.to_string(), "given_name".to_string()], true),
                ],
            }
        }

        pub fn addr_street() -> Self {
            // unwraps below are safe because claims path is not empty
            Self {
                format: CredentialQueryFormat::MsoMdoc {
                    doctype_value: ADDR.to_string(),
                },
                claims: vec![AttributeRequest::new_with_keys(
                    vec![ADDR_NS.to_string(), "street_address".to_string()],
                    true,
                )],
            }
        }
    }

    impl NormalizedCredentialRequests {
        pub fn new_pid_example() -> Self {
            vec![NormalizedCredentialRequest::new_pid_example()].try_into().unwrap()
        }

        pub fn mock_from_vecs(input: Vec<(String, Vec<VecNonEmpty<String>>)>) -> Self {
            let requests = input
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
            Self::try_new(requests).expect("should contain at least 1 request")
        }

        pub fn example() -> Self {
            vec![NormalizedCredentialRequest {
                format: CredentialQueryFormat::MsoMdoc {
                    doctype_value: EXAMPLE_DOC_TYPE.to_string(),
                },
                claims: vec![
                    AttributeRequest::new_with_keys(
                        vec![EXAMPLE_NAMESPACE.to_string(), "family_name".to_string()],
                        false,
                    ),
                    AttributeRequest::new_with_keys(
                        vec![EXAMPLE_NAMESPACE.to_string(), "issue_date".to_string()],
                        false,
                    ),
                    AttributeRequest::new_with_keys(
                        vec![EXAMPLE_NAMESPACE.to_string(), "expiry_date".to_string()],
                        false,
                    ),
                    AttributeRequest::new_with_keys(
                        vec![EXAMPLE_NAMESPACE.to_string(), "document_number".to_string()],
                        false,
                    ),
                    AttributeRequest::new_with_keys(vec![EXAMPLE_NAMESPACE.to_string(), "portrait".to_string()], false),
                    AttributeRequest::new_with_keys(
                        vec![EXAMPLE_NAMESPACE.to_string(), "driving_privileges".to_string()],
                        false,
                    ),
                ],
            }]
            .try_into()
            .unwrap()
        }
    }
}
