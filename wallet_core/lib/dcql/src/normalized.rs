use std::num::NonZero;

use derive_more::IntoIterator;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;

use error_category::ErrorCategory;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;

use crate::ClaimPath;
use crate::ClaimsQuery;
use crate::ClaimsSelection;
use crate::CredentialQuery;
use crate::CredentialQueryFormat;
use crate::CredentialQueryIdentifier;
use crate::MayHaveUniqueId;
use crate::Query;
use crate::UniqueIdVec;
use crate::UniqueIdVecError;

#[derive(Debug, Clone, PartialEq, Eq, IntoIterator, Serialize, Deserialize)]
pub struct NormalizedCredentialRequests(UniqueIdVec<NormalizedCredentialRequest>);

impl AsRef<[NormalizedCredentialRequest]> for NormalizedCredentialRequests {
    fn as_ref(&self) -> &[NormalizedCredentialRequest] {
        let Self(unique_vec) = self;

        unique_vec.as_ref()
    }
}

impl TryFrom<Vec<NormalizedCredentialRequest>> for NormalizedCredentialRequests {
    type Error = UniqueIdVecError;

    fn try_from(value: Vec<NormalizedCredentialRequest>) -> Result<Self, Self::Error> {
        let unique_vec = UniqueIdVec::try_from(value)?;

        Ok(Self(unique_vec))
    }
}

/// Request for a credential.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedCredentialRequest {
    pub id: CredentialQueryIdentifier,
    pub format: CredentialQueryFormat,
    pub claims: VecNonEmpty<AttributeRequest>,
}

impl MayHaveUniqueId for NormalizedCredentialRequest {
    fn id(&self) -> Option<&str> {
        Some(self.id.as_ref())
    }
}

/// Request for a single attribute with the given [path].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttributeRequest {
    pub path: VecNonEmpty<ClaimPath>,
    pub intent_to_retain: bool,
}

impl NormalizedCredentialRequest {
    pub fn claim_paths(&self) -> impl Iterator<Item = &VecNonEmpty<ClaimPath>> {
        self.claims.iter().map(|claim| &claim.path)
    }
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[category(critical)]
pub enum UnsupportedDcqlFeatures {
    #[error("'credential_sets' are not supported")]
    CredentialSets,
    #[error("multiple credential queries are not supported")]
    MultipleCredentialQueries,
    #[error("disclosing only non-selectively disclosable claims is not supported")]
    NoClaims,
    #[error("'claim_sets' are not supported")]
    MultipleClaimSets,
    #[error("claim query with 'values' is not supported")]
    ClaimValues,
    #[error("'trusted_authorities' is not suported")]
    TrustedAuthorities,
    #[error("requests that do not require a cryptographic holder binding proof are not supported")]
    CryptographicHolderBindingNotRequired,
    // TODO: PVW-4139 support SdJwt
    #[error("format 'dc+sd-jwt' is not supported")]
    SdJwt,
    #[error("invalid claim path length ({0}), mdoc requires 2")]
    InvalidClaimPathLength(NonZero<usize>),
    #[error("unsupported ClaimPath variant, only SelectByKey is supported")]
    UnsupportedClaimPathVariant,
    #[error("'intent_to_retain' is mandatory for mso_mdoc format")]
    MissingIntentToRetain,
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
        // This unwrap is safe, because source.credentials is also `UniqueIdVec`.
        Ok(requests.try_into().unwrap())
    }
}

impl From<NormalizedCredentialRequests> for Query {
    fn from(value: NormalizedCredentialRequests) -> Self {
        Self {
            credentials: value
                .into_iter()
                .map(CredentialQuery::from)
                .collect_vec()
                .try_into()
                // This unwrap is safe, the source is also a `UniqueIdVec`.
                .unwrap(),
            credential_sets: vec![],
        }
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
            return Err(UnsupportedDcqlFeatures::CryptographicHolderBindingNotRequired);
        }

        let CredentialQueryFormat::MsoMdoc { doctype_value } = source.format else {
            return Err(UnsupportedDcqlFeatures::SdJwt);
        };
        let claims = match source.claims_selection {
            ClaimsSelection::NoSelectivelyDisclosable => {
                return Err(UnsupportedDcqlFeatures::NoClaims);
            }
            ClaimsSelection::Combinations { .. } => {
                return Err(UnsupportedDcqlFeatures::MultipleClaimSets);
            }
            ClaimsSelection::All { claims } => claims
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?
                .try_into()
                // This unwrap is safe, as the source is guaranteed not to be empty.
                .unwrap(),
        };

        let request = Self {
            id: source.id,
            format: CredentialQueryFormat::MsoMdoc { doctype_value },
            claims,
        };
        Ok(request)
    }
}

impl From<NormalizedCredentialRequest> for CredentialQuery {
    fn from(value: NormalizedCredentialRequest) -> Self {
        Self {
            id: value.id,
            format: value.format,
            multiple: false,
            trusted_authorities: vec![],
            require_cryptographic_holder_binding: true,
            claims_selection: ClaimsSelection::All {
                claims: value
                    .claims
                    .into_iter()
                    .map(ClaimsQuery::from)
                    .collect_vec()
                    .try_into()
                    // This unwrap is safe as the source is guaranteed not to be empty and no identifiers are used.
                    .unwrap(),
            },
        }
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
        if source
            .path
            .nonempty_iter()
            .any(|p| !matches!(p, ClaimPath::SelectByKey(_)))
        {
            return Err(UnsupportedDcqlFeatures::UnsupportedClaimPathVariant);
        }
        let Some(intent_to_retain) = source.intent_to_retain else {
            return Err(UnsupportedDcqlFeatures::MissingIntentToRetain);
        };

        let request = AttributeRequest {
            path: source.path,
            intent_to_retain,
        };
        Ok(request)
    }
}

impl From<AttributeRequest> for ClaimsQuery {
    fn from(value: AttributeRequest) -> Self {
        Self {
            // Just set the identifier to `None`, as it is only useful for `claim_sets`, which we do not support.
            id: None,
            path: value.path,
            values: vec![],
            intent_to_retain: Some(value.intent_to_retain),
        }
    }
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use itertools::Itertools;

    use utils::vec_at_least::VecNonEmpty;
    use utils::vec_nonempty;

    use crate::ClaimPath;
    use crate::ClaimsQuery;
    use crate::ClaimsSelection;
    use crate::CredentialQuery;
    use crate::CredentialQueryFormat;
    use crate::CredentialQueryIdentifier;
    use crate::Query;

    use super::AttributeRequest;
    use super::NormalizedCredentialRequest;
    use super::NormalizedCredentialRequests;

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

    impl Query {
        pub fn new_mdoc_example() -> Self {
            Self {
                credentials: vec![CredentialQuery {
                    id: "new_mdoc_example".to_string().try_into().unwrap(),
                    format: CredentialQueryFormat::MsoMdoc {
                        doctype_value: EXAMPLE_DOC_TYPE.to_string(),
                    },
                    multiple: false,
                    trusted_authorities: vec![],
                    require_cryptographic_holder_binding: true,
                    claims_selection: ClaimsSelection::All {
                        claims: vec![ClaimsQuery {
                            id: None,
                            path: vec_nonempty![
                                ClaimPath::SelectByKey(EXAMPLE_NAMESPACE.to_string()),
                                ClaimPath::SelectByKey(ATTR_FAMILY_NAME.to_string()),
                            ],
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

        pub fn new_pid_example() -> Self {
            Self {
                credentials: vec![CredentialQuery {
                    id: "new_pid_example".to_string().try_into().unwrap(),
                    format: CredentialQueryFormat::MsoMdoc {
                        doctype_value: PID.to_string(),
                    },
                    multiple: false,
                    trusted_authorities: vec![],
                    require_cryptographic_holder_binding: true,
                    claims_selection: ClaimsSelection::All {
                        claims: vec![
                            ClaimsQuery {
                                id: None,
                                path: vec_nonempty![
                                    ClaimPath::SelectByKey(PID.to_string()),
                                    ClaimPath::SelectByKey(ATTR_BSN.to_string()),
                                ],
                                values: vec![],
                                intent_to_retain: Some(true),
                            },
                            ClaimsQuery {
                                id: None,
                                path: vec_nonempty![
                                    ClaimPath::SelectByKey(PID.to_string()),
                                    ClaimPath::SelectByKey(ATTR_GIVEN_NAME.to_string()),
                                ],
                                values: vec![],
                                intent_to_retain: Some(true),
                            },
                            ClaimsQuery {
                                id: None,
                                path: vec_nonempty![
                                    ClaimPath::SelectByKey(PID.to_string()),
                                    ClaimPath::SelectByKey(ATTR_FAMILY_NAME.to_string()),
                                ],
                                values: vec![],
                                intent_to_retain: Some(true),
                            },
                        ]
                        .try_into()
                        .unwrap(),
                    },
                }]
                .try_into()
                .unwrap(),
                credential_sets: vec![],
            }
        }

        pub fn pid_full_name() -> Self {
            Self {
                credentials: vec![CredentialQuery {
                    id: "pid_full_name".to_string().try_into().unwrap(),
                    format: CredentialQueryFormat::MsoMdoc {
                        doctype_value: PID.to_string(),
                    },
                    multiple: false,
                    trusted_authorities: vec![],
                    require_cryptographic_holder_binding: true,
                    claims_selection: ClaimsSelection::All {
                        claims: vec![
                            ClaimsQuery {
                                id: None,
                                path: vec_nonempty![
                                    ClaimPath::SelectByKey(PID.to_string()),
                                    ClaimPath::SelectByKey(ATTR_GIVEN_NAME.to_string()),
                                ],
                                values: vec![],
                                intent_to_retain: Some(true),
                            },
                            ClaimsQuery {
                                id: None,
                                path: vec_nonempty![
                                    ClaimPath::SelectByKey(PID.to_string()),
                                    ClaimPath::SelectByKey(ATTR_FAMILY_NAME.to_string()),
                                ],
                                values: vec![],
                                intent_to_retain: Some(true),
                            },
                        ]
                        .try_into()
                        .unwrap(),
                    },
                }]
                .try_into()
                .unwrap(),
                credential_sets: vec![],
            }
        }

        pub fn pid_family_name() -> Self {
            Self {
                credentials: vec![CredentialQuery {
                    id: "my_credential".to_string().try_into().unwrap(),
                    format: CredentialQueryFormat::MsoMdoc {
                        doctype_value: PID.to_string(),
                    },
                    multiple: false,
                    trusted_authorities: vec![],
                    require_cryptographic_holder_binding: true,
                    claims_selection: ClaimsSelection::All {
                        claims: vec![ClaimsQuery {
                            id: None,
                            path: vec_nonempty![
                                ClaimPath::SelectByKey(PID.to_string()),
                                ClaimPath::SelectByKey(ATTR_FAMILY_NAME.to_string()),
                            ],
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

    impl AttributeRequest {
        pub fn new_with_keys(keys: Vec<String>, intent_to_retain: bool) -> Self {
            Self {
                path: VecNonEmpty::try_from(keys.into_iter().map(ClaimPath::SelectByKey).collect::<Vec<_>>()).unwrap(),
                intent_to_retain,
            }
        }
    }

    impl NormalizedCredentialRequests {
        pub fn new_pid_example() -> Self {
            vec![NormalizedCredentialRequest::new_pid_example()].try_into().unwrap()
        }

        pub fn new_example() -> Self {
            vec![NormalizedCredentialRequest::new_mdoc_example()]
                .try_into()
                .unwrap()
        }

        pub fn new_big_example() -> Self {
            vec![NormalizedCredentialRequest {
                id: "my_credential".to_string().try_into().unwrap(),
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
                    AttributeRequest::new_with_keys(
                        vec![EXAMPLE_NAMESPACE.to_string(), ATTR_PORTRAIT.to_string()],
                        false,
                    ),
                    AttributeRequest::new_with_keys(
                        vec![EXAMPLE_NAMESPACE.to_string(), ATTR_DRIVING_PRIVILEGES.to_string()],
                        false,
                    ),
                ]
                .try_into()
                .unwrap(),
            }]
            .try_into()
            .unwrap()
        }

        fn new_mock_from_slices<'a>(
            input: impl IntoIterator<Item = (CredentialQueryIdentifier, CredentialQueryFormat, &'a [&'a [&'a str]])>,
        ) -> Self {
            let requests: Vec<_> = input
                .into_iter()
                .map(|(id, format, paths)| {
                    let claims = paths
                        .iter()
                        .copied()
                        .map(mock_attribute_request_from_slice)
                        .collect_vec()
                        .try_into()
                        .expect("should contain at least 1 claim");
                    NormalizedCredentialRequest { id, format, claims }
                })
                .collect();
            requests.try_into().expect("should contain at least 1 request")
        }

        pub fn new_mock_mdoc_from_slices(input: &[(&str, &[&[&str]])]) -> Self {
            let input_iter = input.iter().copied().enumerate().map(|(index, (doctype, paths))| {
                let id = format!("mdoc_{index}").try_into().unwrap();
                let format = CredentialQueryFormat::MsoMdoc {
                    doctype_value: doctype.to_string(),
                };

                (id, format, paths)
            });

            Self::new_mock_from_slices(input_iter)
        }

        pub fn new_mock_sd_jwt_from_slices(input: &[(&[&str], &[&[&str]])]) -> Self {
            let input_iter = input
                .iter()
                .copied()
                .enumerate()
                .map(|(index, (credential_types, paths))| {
                    let id = format!("sd_jwt_{index}").try_into().unwrap();
                    let format = CredentialQueryFormat::SdJwt {
                        vct_values: credential_types
                            .iter()
                            .copied()
                            .map(str::to_string)
                            .collect::<Vec<_>>()
                            .try_into()
                            .expect("should contain at least one credential type"),
                    };

                    (id, format, paths)
                });

            Self::new_mock_from_slices(input_iter)
        }
    }

    impl NormalizedCredentialRequest {
        pub fn new_mdoc_example() -> Self {
            Self {
                id: "new_mdoc_example".to_string().try_into().unwrap(),
                format: CredentialQueryFormat::MsoMdoc {
                    doctype_value: EXAMPLE_DOC_TYPE.to_string(),
                },
                claims: vec![AttributeRequest::new_with_keys(
                    vec![EXAMPLE_NAMESPACE.to_string(), ATTR_FAMILY_NAME.to_string()],
                    true,
                )]
                .try_into()
                .unwrap(),
            }
        }

        pub fn new_pid_example() -> Self {
            Self {
                id: "pid".to_string().try_into().unwrap(),
                format: CredentialQueryFormat::MsoMdoc {
                    doctype_value: PID.to_string(),
                },
                claims: vec![
                    AttributeRequest::new_with_keys(vec![PID.to_string(), ATTR_BSN.to_string()], false),
                    AttributeRequest::new_with_keys(vec![PID.to_string(), ATTR_GIVEN_NAME.to_string()], false),
                    AttributeRequest::new_with_keys(vec![PID.to_string(), ATTR_FAMILY_NAME.to_string()], false),
                ]
                .try_into()
                .unwrap(),
            }
        }

        pub fn pid_full_name() -> Self {
            Self {
                id: "pid".to_string().try_into().unwrap(),
                format: CredentialQueryFormat::MsoMdoc {
                    doctype_value: PID.to_string(),
                },
                claims: vec![
                    AttributeRequest::new_with_keys(vec![PID.to_string(), ATTR_FAMILY_NAME.to_string()], true),
                    AttributeRequest::new_with_keys(vec![PID.to_string(), ATTR_GIVEN_NAME.to_string()], true),
                ]
                .try_into()
                .unwrap(),
            }
        }

        pub fn addr_street() -> Self {
            Self {
                id: "pid".to_string().try_into().unwrap(),
                format: CredentialQueryFormat::MsoMdoc {
                    doctype_value: ADDR.to_string(),
                },
                claims: vec![AttributeRequest::new_with_keys(
                    vec![ADDR_NS.to_string(), ATTR_STREET_ADDRESS.to_string()],
                    true,
                )]
                .try_into()
                .unwrap(),
            }
        }
    }

    pub fn mock_attribute_request_from_slice(path: &[&str]) -> AttributeRequest {
        let claim_path: Vec<_> = path
            .iter()
            .copied()
            .map(|key| ClaimPath::SelectByKey(key.to_string()))
            .collect();

        AttributeRequest {
            path: VecNonEmpty::try_from(claim_path).expect("empy path not allowed"),
            intent_to_retain: false,
        }
    }
}

#[cfg(test)]
mod test {
    use rstest::rstest;

    use crate::ClaimPath;
    use crate::ClaimsQuery;
    use crate::ClaimsSelection;
    use crate::CredentialQuery;
    use crate::Query;
    use crate::TrustedAuthoritiesQuery;

    use super::NormalizedCredentialRequests;
    use super::UnsupportedDcqlFeatures;

    #[rstest]
    #[case(Query::example_with_multiple_credentials(), Err(UnsupportedDcqlFeatures::SdJwt))]
    #[case(Query::example_with_credential_sets(), Err(UnsupportedDcqlFeatures::CredentialSets))]
    #[case(Query::example_with_claim_sets(), Err(UnsupportedDcqlFeatures::SdJwt))]
    #[case(Query::example_with_values(), Err(UnsupportedDcqlFeatures::SdJwt))]
    #[case(Query::new_mdoc_example(), Ok(NormalizedCredentialRequests::new_example()))]
    #[case(query_multiple_queries(), Err(UnsupportedDcqlFeatures::MultipleCredentialQueries))]
    #[case(query_with_trusted_authorities(), Err(UnsupportedDcqlFeatures::TrustedAuthorities))]
    #[case(query_without_claims(), Err(UnsupportedDcqlFeatures::NoClaims))]
    #[case(query_with_claim_sets(), Err(UnsupportedDcqlFeatures::MultipleClaimSets))]
    #[case(
        query_with_invalid_claim_path_length(),
        Err(UnsupportedDcqlFeatures::InvalidClaimPathLength(1.try_into().unwrap())),
    )]
    #[case(
        query_with_invalid_claim_path_variant_all(),
        Err(UnsupportedDcqlFeatures::UnsupportedClaimPathVariant)
    )]
    #[case(
        query_with_invalid_claim_path_variant_by_index(),
        Err(UnsupportedDcqlFeatures::UnsupportedClaimPathVariant)
    )]
    #[case(
        query_with_missing_intent_to_retain(),
        Err(UnsupportedDcqlFeatures::MissingIntentToRetain)
    )]
    #[case(query_with_values(), Err(UnsupportedDcqlFeatures::ClaimValues))]
    #[case(
        query_without_cryptographic_holder_binding_requirement(),
        Err(UnsupportedDcqlFeatures::CryptographicHolderBindingNotRequired)
    )]
    fn test_conversion(
        #[case] query: Query,
        #[case] expected: Result<NormalizedCredentialRequests, UnsupportedDcqlFeatures>,
    ) {
        let result = NormalizedCredentialRequests::try_from(query.clone());

        assert_eq!(result, expected);

        // If the conversion succeeds, test that the conversion back matches the input.
        // Note that this requires that the input does not use `ClaimsQuery` identifiers.
        if let Ok(normalized) = result {
            assert_eq!(Query::from(normalized), query);
        }
    }

    fn mdoc_example_query_mutate_first_credential_query<F>(mutate: F) -> Query
    where
        F: FnOnce(CredentialQuery) -> CredentialQuery,
    {
        let mut query = Query::pid_family_name();
        query.credentials = vec![mutate(query.credentials.into_iter().next().unwrap())]
            .try_into()
            .unwrap();
        query
    }

    fn query_multiple_queries() -> Query {
        mdoc_example_query_mutate_first_credential_query(|mut c| {
            c.multiple = true;
            c
        })
    }

    fn query_with_trusted_authorities() -> Query {
        mdoc_example_query_mutate_first_credential_query(|mut c| {
            c.trusted_authorities
                .push(TrustedAuthoritiesQuery::Other("placeholder".to_string()));
            c
        })
    }

    fn query_with_missing_intent_to_retain() -> Query {
        mdoc_example_query_mutate_first_credential_query(|mut c| {
            c.claims_selection = ClaimsSelection::All {
                claims: vec![mdoc_claims_query_missing_intent_to_retain()].try_into().unwrap(),
            };
            c
        })
    }

    fn query_without_claims() -> Query {
        mdoc_example_query_mutate_first_credential_query(|mut c| {
            c.claims_selection = ClaimsSelection::NoSelectivelyDisclosable;
            c
        })
    }

    fn query_with_claim_sets() -> Query {
        mdoc_example_query_mutate_first_credential_query(|mut c| {
            c.claims_selection = ClaimsSelection::Combinations {
                claims: vec![mdoc_claims_query()].try_into().unwrap(),
                claim_sets: vec![vec!["1".to_string().try_into().unwrap()].try_into().unwrap()]
                    .try_into()
                    .unwrap(),
            };
            c
        })
    }

    fn query_with_invalid_claim_path_length() -> Query {
        let claims_query = {
            let mut claims_query = mdoc_claims_query();
            let mut path = claims_query.path.into_inner();
            path.swap_remove(0);
            claims_query.path = path.try_into().unwrap();
            claims_query
        };
        mdoc_example_query_mutate_first_credential_query(move |mut c| {
            c.claims_selection = ClaimsSelection::All {
                claims: vec![claims_query].try_into().unwrap(),
            };
            c
        })
    }

    fn query_with_invalid_claim_path_variant_all() -> Query {
        let claims_query = {
            let mut claims_query = mdoc_claims_query();
            claims_query.path = vec![ClaimPath::SelectByKey("ns".to_string()), ClaimPath::SelectAll]
                .try_into()
                .unwrap();
            claims_query
        };
        mdoc_example_query_mutate_first_credential_query(move |mut c| {
            c.claims_selection = ClaimsSelection::All {
                claims: vec![claims_query].try_into().unwrap(),
            };
            c
        })
    }

    fn query_with_invalid_claim_path_variant_by_index() -> Query {
        let claims_query = {
            let mut claims_query = mdoc_claims_query();
            claims_query.path = vec![ClaimPath::SelectByKey("ns".to_string()), ClaimPath::SelectByIndex(1)]
                .try_into()
                .unwrap();
            claims_query
        };
        mdoc_example_query_mutate_first_credential_query(move |mut c| {
            c.claims_selection = ClaimsSelection::All {
                claims: vec![claims_query].try_into().unwrap(),
            };
            c
        })
    }

    fn query_with_values() -> Query {
        let claims_query = {
            let mut claims_query = mdoc_claims_query();
            claims_query.values = vec![serde_json::Value::Bool(true)];
            claims_query
        };
        mdoc_example_query_mutate_first_credential_query(move |mut c| {
            c.claims_selection = ClaimsSelection::All {
                claims: vec![claims_query].try_into().unwrap(),
            };
            c
        })
    }

    fn query_without_cryptographic_holder_binding_requirement() -> Query {
        mdoc_example_query_mutate_first_credential_query(move |mut c| {
            c.require_cryptographic_holder_binding = false;
            c
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
            intent_to_retain: Some(true),
        }
    }

    fn mdoc_claims_query_missing_intent_to_retain() -> ClaimsQuery {
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
