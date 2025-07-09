use std::collections::HashMap;
use std::collections::HashSet;

use derive_more::AsRef;
use itertools::Itertools;

use utils::vec_at_least::VecNonEmpty;

#[derive(Debug, thiserror::Error)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum AttestationAttributePathsError {
    #[error("no attestation(s) provided")]
    EmptyAttestations,
    #[error("no attribute path(s) provided for attestation type(s): {}", .0.iter().join(", "))]
    EmptyAttributes(HashSet<String>),
}

/// Represents a collection of attribute paths, keyed per attestation type. The constructor of this type guarantees
/// that paths for at least one attestation are present and that each attestation has at least one path specified.
#[derive(Debug, Clone, PartialEq, Eq, AsRef)]
pub struct AttestationAttributePaths(HashMap<String, HashSet<VecNonEmpty<String>>>);

impl AttestationAttributePaths {
    pub fn try_new(
        paths: HashMap<String, HashSet<VecNonEmpty<String>>>,
    ) -> Result<Self, AttestationAttributePathsError> {
        if paths.is_empty() {
            return Err(AttestationAttributePathsError::EmptyAttestations);
        }

        let empty_attestation_types = paths
            .iter()
            .filter(|(_, paths)| paths.is_empty())
            .map(|(attestation_type, _)| attestation_type.clone())
            .collect::<HashSet<_>>();

        if !empty_attestation_types.is_empty() {
            return Err(AttestationAttributePathsError::EmptyAttributes(empty_attestation_types));
        }

        Ok(Self(paths))
    }

    pub fn into_inner(self) -> HashMap<String, HashSet<VecNonEmpty<String>>> {
        let Self(inner) = self;

        inner
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::collections::HashSet;

    use rstest::rstest;

    use utils::vec_at_least::VecNonEmpty;

    use super::AttestationAttributePaths;
    use super::AttestationAttributePathsError;

    #[rstest]
    #[case(attribute_paths(), None)]
    #[case(emtpy_attestations(), Some(AttestationAttributePathsError::EmptyAttestations))]
    #[case(empty_attributes1(), Some(AttestationAttributePathsError::EmptyAttributes(HashSet::from(["att_2".to_string()]))))]
    #[case(
        empty_attributes2(),
        Some(AttestationAttributePathsError::EmptyAttributes(HashSet::from(["att_1".to_string(), "att_3".to_string()])))
    )]
    fn test_attestation_attribute_paths_try_new(
        #[case] paths: HashMap<String, HashSet<VecNonEmpty<String>>>,
        #[case] expected_error: Option<AttestationAttributePathsError>,
    ) {
        let result = AttestationAttributePaths::try_new(paths.clone());

        match expected_error {
            None => {
                let attribute_paths = result.expect("creating AttestationAttributePaths should succeed");

                assert_eq!(attribute_paths.as_ref(), &paths);
            }
            Some(expected_error) => {
                let error = result.expect_err("creating AttestationAttributePaths should not succeed");

                assert_eq!(error, expected_error);
            }
        }
    }

    fn attribute_paths() -> HashMap<String, HashSet<VecNonEmpty<String>>> {
        HashMap::from([
            (
                "att_1".to_string(),
                HashSet::from([vec!["path1".to_string(), "path2".to_string()].try_into().unwrap()]),
            ),
            (
                "att_2".to_string(),
                HashSet::from([
                    vec!["path3".to_string(), "path4".to_string()].try_into().unwrap(),
                    vec!["path5".to_string(), "path6".to_string()].try_into().unwrap(),
                ]),
            ),
        ])
    }

    fn emtpy_attestations() -> HashMap<String, HashSet<VecNonEmpty<String>>> {
        HashMap::new()
    }

    fn empty_attributes1() -> HashMap<String, HashSet<VecNonEmpty<String>>> {
        HashMap::from([
            (
                "att_1".to_string(),
                HashSet::from([vec!["path1".to_string(), "path2".to_string()].try_into().unwrap()]),
            ),
            ("att_2".to_string(), HashSet::new()),
        ])
    }

    fn empty_attributes2() -> HashMap<String, HashSet<VecNonEmpty<String>>> {
        HashMap::from([
            ("att_1".to_string(), HashSet::new()),
            (
                "att_2".to_string(),
                HashSet::from([vec!["path1".to_string(), "path2".to_string()].try_into().unwrap()]),
            ),
            ("att_3".to_string(), HashSet::new()),
        ])
    }
}
