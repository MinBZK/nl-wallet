use std::collections::HashSet;

use itertools::Itertools;

use attestation_types::attribute_paths::AttestationAttributePaths;

mod device_response;
mod device_signed;
mod document;
mod issuer_signed;
mod items_requests;
mod mdoc;

#[cfg(test)]
mod doc_request;
#[cfg(test)]
mod iso_tests;

/// Return the mdoc-specific paths for a particular attestation type in [`AttestationAttributePaths`], which is always
/// a pair of namespace and element (i.e. attribute) identifier. Note that this may return an empty set, either when
/// the attestation type is not present or when none of the paths can be represented as a 2-tuple.
pub fn attribute_paths_to_mdoc_paths<'a>(
    attribute_paths: &'a AttestationAttributePaths,
    attestation_type: &str,
) -> HashSet<(&'a str, &'a str)> {
    attribute_paths
        .as_ref()
        .get(attestation_type)
        .map(|paths| {
            paths
                .iter()
                .filter_map(|path| path.iter().map(String::as_str).collect_tuple())
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::collections::HashSet;

    use rstest::rstest;

    use attestation_types::attribute_paths::AttestationAttributePaths;

    use super::attribute_paths_to_mdoc_paths;

    #[rstest]
    #[case("att_1", HashSet::from([("path2", "path3"), ("path7", "path8")]))]
    #[case("att_2", HashSet::new())]
    #[case("att_3", HashSet::new())]
    fn test_attribute_paths_to_mdoc_paths(
        #[case] attestation_type: &str,
        #[case] expected_mdoc_mpaths: HashSet<(&str, &str)>,
    ) {
        assert_eq!(
            attribute_paths_to_mdoc_paths(&attribute_paths(), attestation_type),
            expected_mdoc_mpaths
        );
    }

    fn attribute_paths() -> AttestationAttributePaths {
        AttestationAttributePaths::try_new(HashMap::from([
            (
                "att_1".to_string(),
                HashSet::from([
                    vec!["path1".to_string()].try_into().unwrap(),
                    vec!["path2".to_string(), "path3".to_string()].try_into().unwrap(),
                    vec!["path4".to_string(), "path5".to_string(), "path6".to_string()]
                        .try_into()
                        .unwrap(),
                    vec!["path7".to_string(), "path8".to_string()].try_into().unwrap(),
                ]),
            ),
            (
                "att_2".to_string(),
                HashSet::from([
                    vec!["path1".to_string(), "path2".to_string(), "path3".to_string()]
                        .try_into()
                        .unwrap(),
                    vec!["path4".to_string()].try_into().unwrap(),
                ]),
            ),
        ]))
        .unwrap()
    }
}
