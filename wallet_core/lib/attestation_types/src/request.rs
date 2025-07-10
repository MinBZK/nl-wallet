use std::num::NonZero;

use nutype::nutype;
use serde::Deserialize;
use serde::Serialize;

use dcql::ClaimPath;
use dcql::ClaimsQuery;
use dcql::ClaimsSelection;
use dcql::CredentialQuery;
use dcql::CredentialQueryFormat;
use dcql::Query;
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

#[derive(Debug, thiserror::Error)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum UnsupportedDcqlFeatures {
    #[error("'credential_sets' are not supported")]
    CredentialSets,
    #[error("multiple credential querys are not supported")]
    MultipleCredentialQueries,
    #[error("'claim_sets' are not supported")]
    MultipleClaimSets,
    #[error("claim query with 'values' is not supported")]
    ClaimValues,
    #[error("'trusted_authorities' not suported")]
    TrustedAuthorities,
    // TODO: PVW-4139 support SdJwt
    #[error("format 'dc+sd-jwt' not supported")]
    SdJwt,
    #[error("empty query not supported")]
    EmptyQuery,
    #[error("invalid claim path length ({0}), mdoc requires 2")]
    InvalidClaimPathLength(NonZero<usize>),
}

impl TryFrom<Query> for NormalizedCredentialRequests {
    type Error = UnsupportedDcqlFeatures;

    fn try_from(source: Query) -> Result<Self, Self::Error> {
        if !source.credential_sets.is_empty() {
            return Err(UnsupportedDcqlFeatures::CredentialSets);
        }
        let requests = source
            .credentials
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()?;
        requests.try_into().map_err(|_| UnsupportedDcqlFeatures::EmptyQuery)
    }
}

impl TryFrom<CredentialQuery> for NormalizedCredentialRequest {
    type Error = UnsupportedDcqlFeatures;

    fn try_from(source: CredentialQuery) -> Result<Self, Self::Error> {
        if source.multiple {
            return Err(UnsupportedDcqlFeatures::MultipleCredentialQueries);
        }
        if !source.trusted_authorities.is_empty() {
            return Err(UnsupportedDcqlFeatures::TrustedAuthorities);
        }
        if !source.require_cryptographic_holder_binding {
            todo!()
        }

        let CredentialQueryFormat::MsoMdoc { doctype_value } = source.format else {
            return Err(UnsupportedDcqlFeatures::SdJwt);
        };
        let claims = match source.claims_selection {
            ClaimsSelection::NoSelectivelyDisclosable => {
                vec![]
            }
            ClaimsSelection::Combinations { .. } => {
                return Err(UnsupportedDcqlFeatures::MultipleClaimSets);
            }
            ClaimsSelection::All { claims } => claims.into_iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
        };

        let request = Self {
            format: CredentialQueryFormat::MsoMdoc { doctype_value },
            claims,
        };
        Ok(request)
    }
}

impl TryFrom<ClaimsQuery> for AttributeRequest {
    type Error = UnsupportedDcqlFeatures;

    fn try_from(source: ClaimsQuery) -> Result<Self, Self::Error> {
        if !source.values.is_empty() {
            return Err(UnsupportedDcqlFeatures::ClaimValues);
        }
        if source.path.len().get() != 2 {
            return Err(UnsupportedDcqlFeatures::InvalidClaimPathLength(source.path.len()));
        }

        let request = AttributeRequest {
            path: source.path,
            intent_to_retain: source.intent_to_retain.unwrap_or_default(),
        };
        Ok(request)
    }
}

#[cfg(test)]
mod test {
    use rstest::rstest;

    use dcql::{ClaimPath, ClaimsQuery, ClaimsSelection, CredentialQuery, CredentialQueryFormat, Query};
    use utils::vec_at_least::VecNonEmpty;

    use crate::request::{NormalizedCredentialRequest, UnsupportedDcqlFeatures};

    use super::{
        mock::{EXAMPLE_ATTR_NAME, EXAMPLE_DOC_TYPE, EXAMPLE_NAMESPACE},
        AttributeRequest, MdocCredentialRequestError, NormalizedCredentialRequests,
    };

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

    #[rstest]
    #[case(Query::example_with_multiple_credentials(), Err(UnsupportedDcqlFeatures::SdJwt))]
    #[case(Query::example_with_credential_sets(), Err(UnsupportedDcqlFeatures::CredentialSets))]
    #[case(Query::example_with_claim_sets(), Err(UnsupportedDcqlFeatures::SdJwt))]
    #[case(Query::example_with_values(), Err(UnsupportedDcqlFeatures::SdJwt))]
    #[case(mdoc_example_query(), Ok(vec![NormalizedCredentialRequest::new_example()].try_into().unwrap()))]
    fn test_conversion(
        #[case] query: Query,
        #[case] expected: Result<NormalizedCredentialRequests, UnsupportedDcqlFeatures>,
    ) {
        let result: Result<NormalizedCredentialRequests, _> = query.try_into();
        assert_eq!(result, expected);
    }

    fn mdoc_example_query() -> Query {
        Query {
            credentials: vec![CredentialQuery {
                id: "my_credential".to_string(),
                format: CredentialQueryFormat::MsoMdoc {
                    doctype_value: EXAMPLE_DOC_TYPE.to_string(),
                },
                multiple: false,
                trusted_authorities: vec![],
                require_cryptographic_holder_binding: true,
                claims_selection: ClaimsSelection::All {
                    claims: vec![ClaimsQuery {
                        id: None,
                        path: vec![
                            ClaimPath::SelectByKey(EXAMPLE_NAMESPACE.to_string()),
                            ClaimPath::SelectByKey(EXAMPLE_ATTR_NAME.to_string()),
                        ]
                        .try_into()
                        .unwrap(),
                        values: vec![],
                        intent_to_retain: Some(true),
                    }]
                    .try_into()
                    .unwrap(),
                },
            }]
            .try_into()
            .unwrap(),
            credential_sets: vec![],
        }
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
