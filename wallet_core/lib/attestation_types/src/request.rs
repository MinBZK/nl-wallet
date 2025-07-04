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

#[cfg(any(test, feature = "mock"))]
mod mock {
    use std::collections::HashMap;
    use std::collections::HashSet;

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

    impl NormalizedCredentialRequest {
        pub fn new_example() -> Self {
            Self {
                format: CredentialQueryFormat::MsoMdoc {
                    doctype_value: EXAMPLE_DOC_TYPE.to_string(),
                },
                claims: vec![AttributeRequest {
                    // unwrap below is safe because claims path is not empty
                    path: vec![
                        ClaimPath::SelectByKey(EXAMPLE_NAMESPACE.to_string()),
                        ClaimPath::SelectByKey(EXAMPLE_ATTR_NAME.to_string()),
                    ]
                    .try_into()
                    .unwrap(),
                    intent_to_retain: true,
                }],
            }
        }

        pub fn new_empty() -> Self {
            Self {
                format: CredentialQueryFormat::MsoMdoc {
                    doctype_value: EXAMPLE_DOC_TYPE.to_string(),
                },
                claims: vec![],
            }
        }

        pub fn pid_full_name() -> Self {
            // unwraps below are safe because claims path is not empty
            Self {
                format: CredentialQueryFormat::MsoMdoc {
                    doctype_value: PID.to_string(),
                },
                claims: vec![
                    AttributeRequest {
                        path: vec![
                            ClaimPath::SelectByKey(PID.to_string()),
                            ClaimPath::SelectByKey("family_name".to_string()),
                        ]
                        .try_into()
                        .unwrap(),
                        intent_to_retain: true,
                    },
                    AttributeRequest {
                        path: vec![
                            ClaimPath::SelectByKey(PID.to_string()),
                            ClaimPath::SelectByKey("given_name".to_string()),
                        ]
                        .try_into()
                        .unwrap(),
                        intent_to_retain: true,
                    },
                ],
            }
        }

        pub fn addr_street() -> Self {
            // unwraps below are safe because claims path is not empty
            Self {
                format: CredentialQueryFormat::MsoMdoc {
                    doctype_value: ADDR.to_string(),
                },
                claims: vec![AttributeRequest {
                    path: vec![
                        ClaimPath::SelectByKey(ADDR_NS.to_string()),
                        ClaimPath::SelectByKey("street_address".to_string()),
                    ]
                    .try_into()
                    .unwrap(),
                    intent_to_retain: true,
                }],
            }
        }
    }

    impl NormalizedCredentialRequests {
        pub fn mock_from_hashmap(input: HashMap<String, HashSet<VecNonEmpty<String>>>) -> Self {
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
                    AttributeRequest {
                        path: vec![
                            ClaimPath::SelectByKey(EXAMPLE_NAMESPACE.to_string()),
                            ClaimPath::SelectByKey("family_name".to_string()),
                        ]
                        .try_into()
                        .unwrap(),
                        intent_to_retain: false,
                    },
                    AttributeRequest {
                        path: vec![
                            ClaimPath::SelectByKey(EXAMPLE_NAMESPACE.to_string()),
                            ClaimPath::SelectByKey("issue_date".to_string()),
                        ]
                        .try_into()
                        .unwrap(),
                        intent_to_retain: false,
                    },
                    AttributeRequest {
                        path: vec![
                            ClaimPath::SelectByKey(EXAMPLE_NAMESPACE.to_string()),
                            ClaimPath::SelectByKey("expiry_date".to_string()),
                        ]
                        .try_into()
                        .unwrap(),
                        intent_to_retain: false,
                    },
                    AttributeRequest {
                        path: vec![
                            ClaimPath::SelectByKey(EXAMPLE_NAMESPACE.to_string()),
                            ClaimPath::SelectByKey("document_number".to_string()),
                        ]
                        .try_into()
                        .unwrap(),
                        intent_to_retain: false,
                    },
                    AttributeRequest {
                        path: vec![
                            ClaimPath::SelectByKey(EXAMPLE_NAMESPACE.to_string()),
                            ClaimPath::SelectByKey("portrait".to_string()),
                        ]
                        .try_into()
                        .unwrap(),
                        intent_to_retain: false,
                    },
                    AttributeRequest {
                        path: vec![
                            ClaimPath::SelectByKey(EXAMPLE_NAMESPACE.to_string()),
                            ClaimPath::SelectByKey("driving_privileges".to_string()),
                        ]
                        .try_into()
                        .unwrap(),
                        intent_to_retain: false,
                    },
                ],
            }]
            .try_into()
            .unwrap()
        }
    }
}
