use std::collections::HashMap;
use std::collections::HashSet;

use derive_more::AsRef;
use itertools::Itertools;

use utils::vec_at_least::VecNonEmpty;

#[derive(Debug, thiserror::Error)]
pub enum RequestedAttributePathsError {
    #[error("no attribute paths present in request")]
    EmptyRequest,
    #[error("no attribute paths for attestation type(s): {}", .0.join(", "))]
    EmptyAttributes(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq, AsRef)]
pub struct RequestedAttributePaths(HashMap<String, HashSet<VecNonEmpty<String>>>);

impl RequestedAttributePaths {
    pub fn try_new(paths: HashMap<String, HashSet<VecNonEmpty<String>>>) -> Result<Self, RequestedAttributePathsError> {
        if paths.is_empty() {
            return Err(RequestedAttributePathsError::EmptyRequest);
        }

        let empty_attestation_types = paths
            .iter()
            .filter(|(_, paths)| paths.is_empty())
            .map(|(attestation_type, _)| attestation_type.clone())
            .collect_vec();

        if !empty_attestation_types.is_empty() {
            return Err(RequestedAttributePathsError::EmptyAttributes(empty_attestation_types));
        }

        Ok(Self(paths))
    }

    pub fn into_inner(self) -> HashMap<String, HashSet<VecNonEmpty<String>>> {
        let Self(inner) = self;

        inner
    }

    pub fn as_mdoc_paths(&self, doc_type: &str) -> HashSet<(&str, &str)> {
        let Self(paths) = self;

        paths
            .get(doc_type)
            .map(|paths| {
                paths
                    .iter()
                    .filter_map(|path| path.iter().map(String::as_str).collect_tuple())
                    .collect()
            })
            .unwrap_or_default()
    }
}
