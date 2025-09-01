use derive_more::IntoIterator;
use itertools::Either;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;

use error_category::ErrorCategory;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;

use crate::ClaimPath;
use crate::ClaimsQuery;
use crate::ClaimsSelection;
use crate::CredentialFormat;
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
    pub format_request: FormatCredentialRequest,
}

impl MayHaveUniqueId for NormalizedCredentialRequest {
    fn id(&self) -> Option<&str> {
        Some(self.id.as_ref())
    }
}

/// Format specific information for a credential request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormatCredentialRequest {
    MsoMdoc {
        doctype_value: String,
        claims: VecNonEmpty<MdocAttributeRequest>,
    },
    #[serde(rename = "dc+sd-jwt")]
    SdJwt {
        vct_values: VecNonEmpty<String>,
        claims: VecNonEmpty<SdJwtAttributeRequest>,
    },
}

impl FormatCredentialRequest {
    pub fn format(&self) -> CredentialFormat {
        match self {
            FormatCredentialRequest::MsoMdoc { .. } => CredentialFormat::MsoMdoc,
            FormatCredentialRequest::SdJwt { .. } => CredentialFormat::SdJwt,
        }
    }

    pub fn credential_types(&self) -> impl Iterator<Item = &str> {
        match self {
            FormatCredentialRequest::MsoMdoc { doctype_value, .. } => std::slice::from_ref(doctype_value),
            FormatCredentialRequest::SdJwt { vct_values, .. } => vct_values.as_slice(),
        }
        .iter()
        .map(String::as_str)
    }

    pub fn claim_paths(&self) -> impl Iterator<Item = &VecNonEmpty<ClaimPath>> {
        match self {
            FormatCredentialRequest::MsoMdoc { claims, .. } => Either::Left(claims.iter().map(|claim| &claim.path)),
            FormatCredentialRequest::SdJwt { claims, .. } => Either::Right(claims.iter().map(|claim| &claim.path)),
        }
    }
}

/// Request for a single mdoc attribute with the given [path].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MdocAttributeRequest {
    pub path: VecNonEmpty<ClaimPath>,
    pub intent_to_retain: Option<bool>,
}

/// Request for a single SD-JWT attribute with the given [path].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SdJwtAttributeRequest {
    pub path: VecNonEmpty<ClaimPath>,
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
    ClaimSets,
    #[error("claim query with 'values' is not supported")]
    ClaimValues,
    #[error("'trusted_authorities' is not suported")]
    TrustedAuthorities,
    #[error("requests that do not require a cryptographic holder binding proof are not supported")]
    CryptographicHolderBindingNotRequired,
    #[error("unsupported ClaimPath variant, only SelectByKey is supported")]
    UnsupportedClaimPathVariant,
    #[error("'intent_to_retain' is not allowed for dc+sd-jwt format")]
    IntentToRetainPresent,
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

        let claims = match source.claims_selection {
            ClaimsSelection::NoSelectivelyDisclosable => {
                return Err(UnsupportedDcqlFeatures::NoClaims);
            }
            ClaimsSelection::Combinations { .. } => {
                return Err(UnsupportedDcqlFeatures::ClaimSets);
            }
            ClaimsSelection::All { claims } => claims,
        };

        let format_request = match source.format {
            CredentialQueryFormat::MsoMdoc { doctype_value } => {
                let claims = claims
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, _>>()?
                    .try_into()
                    // This unwrap is safe, as the source is guaranteed not to be empty.
                    .unwrap();

                FormatCredentialRequest::MsoMdoc { doctype_value, claims }
            }
            CredentialQueryFormat::SdJwt { vct_values } => {
                let claims = claims
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, _>>()?
                    .try_into()
                    // This unwrap is safe, as the source is guaranteed not to be empty.
                    .unwrap();

                FormatCredentialRequest::SdJwt { vct_values, claims }
            }
        };

        let request = Self {
            id: source.id,
            format_request,
        };
        Ok(request)
    }
}

impl From<NormalizedCredentialRequest> for CredentialQuery {
    fn from(value: NormalizedCredentialRequest) -> Self {
        let (format, claims) = match value.format_request {
            FormatCredentialRequest::MsoMdoc { doctype_value, claims } => (
                CredentialQueryFormat::MsoMdoc { doctype_value },
                claims.into_iter().map(ClaimsQuery::from).collect_vec(),
            ),
            FormatCredentialRequest::SdJwt { vct_values, claims } => (
                CredentialQueryFormat::SdJwt { vct_values },
                claims.into_iter().map(ClaimsQuery::from).collect_vec(),
            ),
        };

        Self {
            id: value.id,
            format,
            multiple: false,
            trusted_authorities: vec![],
            require_cryptographic_holder_binding: true,
            claims_selection: ClaimsSelection::All {
                claims: claims
                    .try_into()
                    // This unwrap is safe as the source is guaranteed not to be empty and no identifiers are used.
                    .unwrap(),
            },
        }
    }
}

fn check_claims_query(claims_query: &ClaimsQuery) -> Result<(), UnsupportedDcqlFeatures> {
    if !claims_query.values.is_empty() {
        return Err(UnsupportedDcqlFeatures::ClaimValues);
    }

    if claims_query
        .path
        .nonempty_iter()
        .any(|p| !matches!(p, ClaimPath::SelectByKey(_)))
    {
        return Err(UnsupportedDcqlFeatures::UnsupportedClaimPathVariant);
    }

    Ok(())
}

impl TryFrom<ClaimsQuery> for MdocAttributeRequest {
    type Error = UnsupportedDcqlFeatures;

    fn try_from(source: ClaimsQuery) -> Result<Self, Self::Error> {
        check_claims_query(&source)?;

        let request = Self {
            path: source.path,
            intent_to_retain: source.intent_to_retain,
        };

        Ok(request)
    }
}

impl From<MdocAttributeRequest> for ClaimsQuery {
    fn from(value: MdocAttributeRequest) -> Self {
        Self {
            // Just set the identifier to `None`, as it is only useful for `claim_sets`, which we do not support.
            id: None,
            path: value.path,
            values: vec![],
            intent_to_retain: value.intent_to_retain,
        }
    }
}

impl TryFrom<ClaimsQuery> for SdJwtAttributeRequest {
    type Error = UnsupportedDcqlFeatures;

    fn try_from(source: ClaimsQuery) -> Result<Self, Self::Error> {
        check_claims_query(&source)?;

        if source.intent_to_retain.is_some() {
            return Err(UnsupportedDcqlFeatures::IntentToRetainPresent);
        }

        let request = Self { path: source.path };

        Ok(request)
    }
}

impl From<SdJwtAttributeRequest> for ClaimsQuery {
    fn from(value: SdJwtAttributeRequest) -> Self {
        Self {
            // Just set the identifier to `None`, as it is only useful for `claim_sets`, which we do not support.
            id: None,
            path: value.path,
            values: vec![],
            intent_to_retain: None,
        }
    }
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use itertools::Itertools;

    use mdoc::examples::EXAMPLE_ATTRIBUTES;
    use mdoc::examples::EXAMPLE_DOC_TYPE;
    use mdoc::examples::EXAMPLE_NAMESPACE;
    use mdoc::test::data::PID;
    use utils::vec_at_least::VecNonEmpty;
    use utils::vec_nonempty;

    use crate::ClaimPath;
    use crate::ClaimsQuery;
    use crate::ClaimsSelection;
    use crate::CredentialFormat;
    use crate::CredentialQuery;
    use crate::CredentialQueryFormat;
    use crate::Query;

    use super::FormatCredentialRequest;
    use super::MdocAttributeRequest;
    use super::NormalizedCredentialRequest;
    use super::NormalizedCredentialRequests;
    use super::SdJwtAttributeRequest;

    impl Query {
        pub fn new_mock_single(credential_query: CredentialQuery) -> Self {
            Self {
                credentials: vec![credential_query].try_into().unwrap(),
                credential_sets: vec![],
            }
        }

        pub fn new_mock_mdoc_iso_example() -> Self {
            Self::new_mock_single(CredentialQuery::new_mock_mdoc_iso_example())
        }

        pub fn new_mock_mdoc_pid_example() -> Self {
            Self::new_mock_single(CredentialQuery::new_mock_mdoc_pid_example())
        }
    }

    impl CredentialQuery {
        pub fn new_mock_mdoc(id: &str, doc_type: &str, name_space: &str, attributes: &[&str]) -> Self {
            CredentialQuery {
                id: id.try_into().expect("identifier should be valid"),
                format: CredentialQueryFormat::MsoMdoc {
                    doctype_value: doc_type.to_string(),
                },
                multiple: false,
                trusted_authorities: vec![],
                require_cryptographic_holder_binding: true,
                claims_selection: ClaimsSelection::All {
                    claims: attributes
                        .iter()
                        .map(|attribute| ClaimsQuery {
                            id: None,
                            path: vec_nonempty![
                                ClaimPath::SelectByKey(name_space.to_string()),
                                ClaimPath::SelectByKey(attribute.to_string()),
                            ],
                            values: vec![],
                            intent_to_retain: None,
                        })
                        .collect_vec()
                        .try_into()
                        .expect("should contain at least one attribute"),
                },
            }
        }

        pub fn new_mock_sd_jwt(id: &str, vcts: &[&str], attribute_paths: &[&[&str]]) -> Self {
            CredentialQuery {
                id: id.try_into().expect("identifier should be valid"),
                format: CredentialQueryFormat::SdJwt {
                    vct_values: vcts
                        .iter()
                        .copied()
                        .map(str::to_string)
                        .collect_vec()
                        .try_into()
                        .expect("should contain at least one vct"),
                },
                multiple: false,
                trusted_authorities: vec![],
                require_cryptographic_holder_binding: true,
                claims_selection: ClaimsSelection::All {
                    claims: attribute_paths
                        .iter()
                        .map(|attributes| ClaimsQuery {
                            id: None,
                            path: attributes
                                .iter()
                                .map(|attribute| ClaimPath::SelectByKey(attribute.to_string()))
                                .collect_vec()
                                .try_into()
                                .expect("should contain at least one path element"),
                            values: vec![],
                            intent_to_retain: None,
                        })
                        .collect_vec()
                        .try_into()
                        .expect("should contain at least one attribute"),
                },
            }
        }

        pub fn new_mock_mdoc_iso_example() -> Self {
            Self::new_mock_mdoc(
                "mdoc_iso_example",
                EXAMPLE_DOC_TYPE,
                EXAMPLE_NAMESPACE,
                &EXAMPLE_ATTRIBUTES,
            )
        }

        pub fn new_mock_mdoc_pid_example() -> Self {
            Self::new_mock_mdoc("mdoc_pid_example", PID, PID, &["bsn", "given_name", "family_name"])
        }
    }

    impl NormalizedCredentialRequests {
        pub fn new_mock_mdoc_from_slices(input: &[(&str, &[&[&str]])]) -> Self {
            input
                .iter()
                .copied()
                .enumerate()
                .map(|(index, (doctype, paths))| {
                    NormalizedCredentialRequest::new_mock_from_slices(
                        &format!("mdoc_{index}"),
                        CredentialFormat::MsoMdoc,
                        &[doctype],
                        paths,
                    )
                })
                .collect_vec()
                .try_into()
                .expect("should contain at least one request")
        }

        pub fn new_mock_sd_jwt_from_slices(input: &[(&[&str], &[&[&str]])]) -> Self {
            input
                .iter()
                .copied()
                .enumerate()
                .map(|(index, (credential_types, paths))| {
                    NormalizedCredentialRequest::new_mock_from_slices(
                        &format!("sd_jwt_{index}"),
                        CredentialFormat::SdJwt,
                        credential_types,
                        paths,
                    )
                })
                .collect_vec()
                .try_into()
                .expect("should contain at least one request")
        }

        pub fn new_mock_mdoc_iso_example() -> Self {
            vec![NormalizedCredentialRequest::new_mock_mdoc_iso_example()]
                .try_into()
                .unwrap()
        }

        pub fn new_mock_mdoc_pid_example() -> Self {
            vec![NormalizedCredentialRequest::new_mock_mdoc_pid_example()]
                .try_into()
                .unwrap()
        }

        pub fn example_with_single_credential() -> Self {
            vec![NormalizedCredentialRequest::new_mock_from_slices(
                "my_credential",
                CredentialFormat::MsoMdoc,
                &["org.iso.7367.1.mVRC"],
                &[
                    &["org.iso.7367.1", "vehicle_holder"],
                    &["org.iso.18013.5.1", "first_name"],
                ],
            )]
            .try_into()
            .unwrap()
        }

        pub fn example_with_multiple_credentials() -> Self {
            vec![
                NormalizedCredentialRequest::new_mock_from_slices(
                    "pid",
                    CredentialFormat::SdJwt,
                    &["https://credentials.example.com/identity_credential"],
                    &[&["given_name"], &["family_name"], &["address", "street_address"]],
                ),
                NormalizedCredentialRequest::new_mock_from_slices(
                    "mdl",
                    CredentialFormat::MsoMdoc,
                    &["org.iso.7367.1.mVRC"],
                    &[
                        &["org.iso.7367.1", "vehicle_holder"],
                        &["org.iso.18013.5.1", "first_name"],
                    ],
                ),
            ]
            .try_into()
            .unwrap()
        }
    }

    impl NormalizedCredentialRequest {
        pub fn new_mock_from_slices(
            id: &str,
            format: CredentialFormat,
            credential_types: &[&str],
            paths: &[&[&str]],
        ) -> Self {
            let id = id.try_into().expect("identifier should be valid");

            let credential_types_iter = credential_types.iter().copied().map(str::to_string);
            let paths_iter = paths.iter().copied().map(|path| {
                let claim_path: Vec<_> = path
                    .iter()
                    .copied()
                    .map(|key| ClaimPath::SelectByKey(key.to_string()))
                    .collect();

                VecNonEmpty::try_from(claim_path).expect("empy path not allowed")
            });

            let format_request = match format {
                CredentialFormat::MsoMdoc => FormatCredentialRequest::MsoMdoc {
                    doctype_value: credential_types_iter
                        .exactly_one()
                        .expect("should have exactly one credential type for mdoc"),
                    claims: paths_iter
                        .map(|path| MdocAttributeRequest {
                            path,
                            intent_to_retain: None,
                        })
                        .collect_vec()
                        .try_into()
                        .expect("should contain at least one claim"),
                },
                CredentialFormat::SdJwt => FormatCredentialRequest::SdJwt {
                    vct_values: credential_types_iter
                        .collect_vec()
                        .try_into()
                        .expect("should have at least one credential type for SD-JWT"),
                    claims: paths_iter
                        .map(|path| SdJwtAttributeRequest { path })
                        .collect_vec()
                        .try_into()
                        .expect("should contain at least one claim"),
                },
            };

            Self { id, format_request }
        }

        pub fn new_mock_mdoc_iso_example() -> Self {
            let attributes = EXAMPLE_ATTRIBUTES
                .iter()
                .map(|attribute| vec![EXAMPLE_NAMESPACE, attribute])
                .collect_vec();

            Self::new_mock_from_slices(
                "mdoc_iso_example",
                CredentialFormat::MsoMdoc,
                &[EXAMPLE_DOC_TYPE],
                &attributes.iter().map(Vec::as_slice).collect_vec(),
            )
        }

        pub fn new_mock_mdoc_pid_example() -> Self {
            Self::new_mock_from_slices(
                "mdoc_pid_example",
                CredentialFormat::MsoMdoc,
                &[PID],
                &[&[PID, "bsn"], &[PID, "given_name"], &[PID, "family_name"]],
            )
        }
    }
}

#[cfg(test)]
mod test {
    use rstest::rstest;

    use utils::vec_nonempty;

    use crate::ClaimPath;
    use crate::ClaimsQuery;
    use crate::ClaimsSelection;
    use crate::CredentialQuery;
    use crate::Query;
    use crate::TrustedAuthoritiesQuery;

    use super::NormalizedCredentialRequests;
    use super::UnsupportedDcqlFeatures;

    #[rstest]
    #[case(
        Query::example_with_single_credential(),
        Ok(NormalizedCredentialRequests::example_with_single_credential())
    )]
    #[case(
        Query::example_with_multiple_credentials(),
        Ok(NormalizedCredentialRequests::example_with_multiple_credentials())
    )]
    #[case(Query::example_with_credential_sets(), Err(UnsupportedDcqlFeatures::CredentialSets))]
    #[case(Query::example_with_claim_sets(), Err(UnsupportedDcqlFeatures::ClaimSets))]
    #[case(Query::example_with_values(), Err(UnsupportedDcqlFeatures::ClaimValues))]
    #[case(
        Query::new_mock_mdoc_iso_example(),
        Ok(NormalizedCredentialRequests::new_mock_mdoc_iso_example())
    )]
    #[case(
        Query::new_mock_mdoc_pid_example(),
        Ok(NormalizedCredentialRequests::new_mock_mdoc_pid_example())
    )]
    #[case(
        mdoc_query_multiple_queries(),
        Err(UnsupportedDcqlFeatures::MultipleCredentialQueries)
    )]
    #[case(
        mdoc_query_with_trusted_authorities(),
        Err(UnsupportedDcqlFeatures::TrustedAuthorities)
    )]
    #[case(mdoc_query_without_claims(), Err(UnsupportedDcqlFeatures::NoClaims))]
    #[case(mdoc_query_with_claim_sets(), Err(UnsupportedDcqlFeatures::ClaimSets))]
    #[case(
        mdoc_query_with_invalid_claim_path_variant_all(),
        Err(UnsupportedDcqlFeatures::UnsupportedClaimPathVariant)
    )]
    #[case(
        mdoc_query_with_invalid_claim_path_variant_by_index(),
        Err(UnsupportedDcqlFeatures::UnsupportedClaimPathVariant)
    )]
    #[case(mdoc_query_with_values(), Err(UnsupportedDcqlFeatures::ClaimValues))]
    #[case(
        mdoc_query_without_cryptographic_holder_binding_requirement(),
        Err(UnsupportedDcqlFeatures::CryptographicHolderBindingNotRequired)
    )]
    #[case(sd_jwt_single_query(), Ok(sd_jwt_single_request()))]
    #[case(sd_jwt_values_query(), Err(UnsupportedDcqlFeatures::ClaimValues))]
    #[case(sd_jwt_no_selectively_disclosable_query(), Err(UnsupportedDcqlFeatures::NoClaims))]
    #[case(sd_jwt_intent_to_retain_query(), Err(UnsupportedDcqlFeatures::IntentToRetainPresent))]
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
        let mut query = Query::new_mock_mdoc_iso_example();
        query.credentials = vec![mutate(query.credentials.into_iter().next().unwrap())]
            .try_into()
            .unwrap();
        query
    }

    fn mdoc_query_multiple_queries() -> Query {
        mdoc_example_query_mutate_first_credential_query(|mut c| {
            c.multiple = true;
            c
        })
    }

    fn mdoc_query_with_trusted_authorities() -> Query {
        mdoc_example_query_mutate_first_credential_query(|mut c| {
            c.trusted_authorities
                .push(TrustedAuthoritiesQuery::Other("placeholder".to_string()));
            c
        })
    }

    fn mdoc_query_without_claims() -> Query {
        mdoc_example_query_mutate_first_credential_query(|mut c| {
            c.claims_selection = ClaimsSelection::NoSelectivelyDisclosable;
            c
        })
    }

    fn mdoc_query_with_claim_sets() -> Query {
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

    fn mdoc_query_with_invalid_claim_path_variant_all() -> Query {
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

    fn mdoc_query_with_invalid_claim_path_variant_by_index() -> Query {
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

    fn mdoc_query_with_values() -> Query {
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

    fn mdoc_query_without_cryptographic_holder_binding_requirement() -> Query {
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

    fn sd_jwt_single_query() -> Query {
        Query::new_mock_single(CredentialQuery::new_mock_sd_jwt(
            "sd_jwt_0",
            &["pid", "another_pid"],
            &[&["given_name"], &["family_name"], &["address", "street_address"]],
        ))
    }

    fn sd_jwt_single_request() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_sd_jwt_from_slices(&[(
            &["pid", "another_pid"],
            &[&["given_name"], &["family_name"], &["address", "street_address"]],
        )])
    }

    fn sd_jwt_values_query() -> Query {
        let mut credential_query = CredentialQuery::new_mock_sd_jwt("intent_to_retain", &["pid"], &[&["family_name"]]);

        credential_query.claims_selection = ClaimsSelection::All {
            claims: vec![ClaimsQuery {
                id: None,
                path: vec_nonempty![ClaimPath::SelectByKey("family_name".to_string())],
                values: vec![serde_json::Value::String("Name".to_string())],
                intent_to_retain: None,
            }]
            .try_into()
            .unwrap(),
        };

        Query {
            credentials: vec![credential_query].try_into().unwrap(),
            credential_sets: vec![],
        }
    }

    fn sd_jwt_no_selectively_disclosable_query() -> Query {
        let mut credential_query = CredentialQuery::new_mock_sd_jwt("intent_to_retain", &["pid"], &[&["family_name"]]);

        credential_query.claims_selection = ClaimsSelection::NoSelectivelyDisclosable;

        Query {
            credentials: vec![credential_query].try_into().unwrap(),
            credential_sets: vec![],
        }
    }

    fn sd_jwt_intent_to_retain_query() -> Query {
        let mut credential_query = CredentialQuery::new_mock_sd_jwt("intent_to_retain", &["pid"], &[&["family_name"]]);

        credential_query.claims_selection = ClaimsSelection::All {
            claims: vec![ClaimsQuery {
                id: None,
                path: vec_nonempty![ClaimPath::SelectByKey("family_name".to_string())],
                values: vec![],
                intent_to_retain: Some(true),
            }]
            .try_into()
            .unwrap(),
        };

        Query {
            credentials: vec![credential_query].try_into().unwrap(),
            credential_sets: vec![],
        }
    }
}
