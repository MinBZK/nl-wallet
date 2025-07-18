use std::collections::HashSet;

use attestation_types::request::AttributeRequest;
use attestation_types::request::NormalizedCredentialRequest;
use dcql::CredentialQueryFormat;
use utils::vec_at_least::VecNonEmpty;

use crate::identifiers::AttributeIdentifier;
use crate::identifiers::AttributeIdentifierError;

mod device_response;
mod device_signed;
mod document;
mod issuer_signed;
mod mdoc;

#[cfg(test)]
mod doc_request;
#[cfg(test)]
mod iso_tests;

#[derive(Debug, thiserror::Error)]
pub enum ResponseValidationError {
    #[error("attributes mismatch: {0:?}")]
    MissingAttributes(Vec<AttributeIdentifier>),
    #[error("expected an mdoc")]
    ExpectedMdoc,
    #[error("invalid attribute identifiers: {0}")]
    AttributeIdentifier(#[from] AttributeIdentifierError),
}

/// Return the mdoc-specific paths for a particular attestation type in [`VecNonEmpty<NormalizedCredentialRequest>`],
/// which is always a pair of namespace and element (i.e. attribute) identifier. Note that this may return an empty set,
/// either when the attestation type is not present or when none of the paths can be represented as a 2-tuple.
pub fn credential_requests_to_mdoc_paths<'a>(
    credential_requests: &'a VecNonEmpty<NormalizedCredentialRequest>,
    attestation_type: &str,
) -> HashSet<(&'a str, &'a str)> {
    credential_requests
        .as_ref()
        .iter()
        .filter(|request| {
            request.format
                == CredentialQueryFormat::MsoMdoc {
                    doctype_value: attestation_type.to_string(),
                }
        })
        .flat_map(|request| {
            request
                .claims
                .iter()
                .flat_map(AttributeRequest::to_namespace_and_attribute)
                .collect::<HashSet<_>>()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use rstest::rstest;

    use attestation_types::request;
    use attestation_types::request::NormalizedCredentialRequest;
    use utils::vec_at_least::VecNonEmpty;

    use crate::holder::disclosure::credential_requests_to_mdoc_paths;

    #[rstest]
    #[case("att_1", HashSet::from([("path2", "path3"), ("path7", "path8")]))]
    #[case("att_2", HashSet::new())]
    #[case("att_3", HashSet::new())]
    fn test_attribute_paths_to_mdoc_paths(
        #[case] attestation_type: &str,
        #[case] expected_mdoc_mpaths: HashSet<(&str, &str)>,
    ) {
        let credential_requests = credential_requests();
        let actual = credential_requests_to_mdoc_paths(&credential_requests, attestation_type);
        assert_eq!(actual, expected_mdoc_mpaths);
    }

    fn credential_requests() -> VecNonEmpty<NormalizedCredentialRequest> {
        request::mock::mock_from_vecs(vec![
            (
                "att_1".to_string(),
                vec![
                    vec!["path1".to_string()].try_into().unwrap(),
                    vec!["path2".to_string(), "path3".to_string()].try_into().unwrap(),
                    vec!["path4".to_string(), "path5".to_string(), "path6".to_string()]
                        .try_into()
                        .unwrap(),
                    vec!["path7".to_string(), "path8".to_string()].try_into().unwrap(),
                ],
            ),
            (
                "att_2".to_string(),
                vec![
                    vec!["path1".to_string(), "path2".to_string(), "path3".to_string()]
                        .try_into()
                        .unwrap(),
                    vec!["path4".to_string()].try_into().unwrap(),
                ],
            ),
        ])
    }
}
