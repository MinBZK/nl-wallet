use std::num::NonZero;

use serde::Deserialize;
use serde::Serialize;

use dcql::ClaimPath;
use dcql::ClaimsQuery;
use dcql::ClaimsSelection;
use dcql::CredentialQuery;
use dcql::CredentialQueryFormat;
use dcql::Query;
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
    #[error("invalid claim path length ({0}), mdoc requires 2")]
    InvalidClaimPathLength(NonZero<usize>),
    #[error("unsupported ClaimPath variant, only SelectByKey is supported")]
    UnsupportedClaimPathVariant,
}

impl NormalizedCredentialRequest {
    pub fn try_from_query(source: Query) -> Result<VecNonEmpty<Self>, UnsupportedDcqlFeatures> {
        if !source.credential_sets.is_empty() {
            return Err(UnsupportedDcqlFeatures::CredentialSets);
        }
        let requests = source
            .credentials
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()?;
        // unwrap is safe, because source.credentials is also [`VecNonEmpty`]
        Ok(requests.try_into().unwrap())
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
        if source.path.iter().any(|p| !matches!(p, ClaimPath::SelectByKey(_))) {
            return Err(UnsupportedDcqlFeatures::UnsupportedClaimPathVariant);
        }

        let request = AttributeRequest {
            path: source.path,
            intent_to_retain: source.intent_to_retain.unwrap_or_default(),
        };
        Ok(request)
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
    use dcql::ClaimsQuery;
    use dcql::ClaimsSelection;
    use dcql::CredentialQuery;
    use dcql::CredentialQueryFormat;
    use dcql::Query;
    use dcql::TrustedAuthoritiesQuery;
    use utils::vec_at_least::VecNonEmpty;

    use super::AttributeRequest;
    use super::MdocCredentialRequestError;
    use super::NormalizedCredentialRequest;

    use super::UnsupportedDcqlFeatures;
    use super::mock::ATTR_FAMILY_NAME;
    use super::mock::ATTR_GIVEN_NAME;
    use super::mock::EXAMPLE_DOC_TYPE;
    use super::mock::EXAMPLE_NAMESPACE;

    #[rstest]
    #[case(
        vec![
            ClaimPath::SelectByKey(EXAMPLE_NAMESPACE.to_string()),
            ClaimPath::SelectByKey(ATTR_FAMILY_NAME.to_string()),
        ].try_into().unwrap(),
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
            ClaimPath::SelectByKey(ATTR_GIVEN_NAME.to_string()),
        ].try_into().unwrap(),
        Err(MdocCredentialRequestError::UnexpectedClaimsPathAmount(3.try_into().unwrap()))
    )]
    #[case(
        vec![
            ClaimPath::SelectByKey(EXAMPLE_NAMESPACE.to_string()),
            ClaimPath::SelectByIndex(1),
        ].try_into().unwrap(),
        Err(MdocCredentialRequestError::UnexpectedClaimsPathType)
    )]
    #[case(
        vec![
            ClaimPath::SelectAll,
            ClaimPath::SelectByKey(ATTR_FAMILY_NAME.to_string()),
        ].try_into().unwrap(),
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
    #[case(query_multiple_queries(), Err(UnsupportedDcqlFeatures::MultipleCredentialQueries))]
    #[case(query_with_trusted_authorities(), Err(UnsupportedDcqlFeatures::TrustedAuthorities))]
    #[case(query_with_claim_sets(), Err(UnsupportedDcqlFeatures::MultipleClaimSets))]
    #[case(
        mdoc_query_with_invalid_claim_path_length(),
        Err(UnsupportedDcqlFeatures::InvalidClaimPathLength(1.try_into().unwrap())),
    )]
    #[case(
        mdoc_query_with_invalid_claim_path_variant_all(),
        Err(UnsupportedDcqlFeatures::UnsupportedClaimPathVariant)
    )]
    #[case(
        mdoc_query_with_invalid_claim_path_variant_by_index(),
        Err(UnsupportedDcqlFeatures::UnsupportedClaimPathVariant)
    )]
    fn test_conversion(
        #[case] query: Query,
        #[case] expected: Result<VecNonEmpty<NormalizedCredentialRequest>, UnsupportedDcqlFeatures>,
    ) {
        let result: Result<VecNonEmpty<NormalizedCredentialRequest>, _> =
            NormalizedCredentialRequest::try_from_query(query);
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
                            ClaimPath::SelectByKey(ATTR_FAMILY_NAME.to_string()),
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

    fn mdoc_example_query_mutate_first_credential_query<F: FnOnce(&mut CredentialQuery)>(mutate: F) -> Query {
        let mut query = mdoc_example_query();
        query.credentials.as_mut().first_mut().map(mutate);
        query
    }

    fn query_multiple_queries() -> Query {
        mdoc_example_query_mutate_first_credential_query(|c| c.multiple = true)
    }

    fn query_with_trusted_authorities() -> Query {
        mdoc_example_query_mutate_first_credential_query(|c| {
            c.trusted_authorities
                .push(TrustedAuthoritiesQuery::Other("placeholder".to_string()))
        })
    }

    fn query_with_claim_sets() -> Query {
        mdoc_example_query_mutate_first_credential_query(|c| {
            c.claims_selection = ClaimsSelection::Combinations {
                claims: vec![mdoc_claims_query()].try_into().unwrap(),
                claim_sets: vec![vec!["1".to_string()].try_into().unwrap()].try_into().unwrap(),
            };
        })
    }

    fn mdoc_query_with_invalid_claim_path_length() -> Query {
        let claims_query = {
            let mut claims_query = mdoc_claims_query();
            let _ = claims_query.path.as_mut().swap_remove(0);
            claims_query
        };
        mdoc_example_query_mutate_first_credential_query(move |c| {
            c.claims_selection = ClaimsSelection::All {
                claims: vec![claims_query].try_into().unwrap(),
            };
        })
    }

    fn mdoc_query_with_invalid_claim_path_variant_all() -> Query {
        let claims_query = {
            let mut claims_query = mdoc_claims_query();
            claims_query.path = vec![ClaimPath::SelectByKey("ns".to_string()), ClaimPath::SelectAll]
                .try_into()
                .unwrap();
            claims_query
        };
        mdoc_example_query_mutate_first_credential_query(move |c| {
            c.claims_selection = ClaimsSelection::All {
                claims: vec![claims_query].try_into().unwrap(),
            };
        })
    }

    fn mdoc_query_with_invalid_claim_path_variant_by_index() -> Query {
        let claims_query = {
            let mut claims_query = mdoc_claims_query();
            claims_query.path = vec![ClaimPath::SelectByKey("ns".to_string()), ClaimPath::SelectByIndex(1)]
                .try_into()
                .unwrap();
            claims_query
        };
        mdoc_example_query_mutate_first_credential_query(move |c| {
            c.claims_selection = ClaimsSelection::All {
                claims: vec![claims_query].try_into().unwrap(),
            };
        })
    }

    fn mdoc_claims_query() -> ClaimsQuery {
        ClaimsQuery {
            id: None,
            path: vec![
                ClaimPath::SelectByKey("ns".to_string()),
                ClaimPath::SelectByKey("attr".to_string()),
            ]
            .try_into()
            .unwrap(),
            values: vec![],
            intent_to_retain: None,
        }
    }
}
