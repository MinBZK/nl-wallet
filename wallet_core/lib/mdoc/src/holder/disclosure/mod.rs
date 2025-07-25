use std::collections::HashSet;

use dcql::CredentialQueryFormat;
use dcql::normalized::AttributeRequest;
use dcql::normalized::NormalizedCredentialRequest;
use utils::vec_at_least::VecNonEmpty;

use crate::identifiers::AttributeIdentifier;

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

    use dcql::normalized;
    use dcql::normalized::NormalizedCredentialRequest;
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
        normalized::mock::mock_mdoc_from_slices(&[
            (
                "att_1",
                &[
                    &["path1"],
                    &["path2", "path3"],
                    &["path4", "path5", "path6"],
                    &["path7", "path8"],
                ],
            ),
            ("att_2", &[&["path1", "path2", "path3"], &["path4"]]),
        ])
    }
}
