use std::collections::HashMap;
use std::collections::HashSet;

use derive_more::AsRef;
use itertools::Itertools;

use utils::vec_at_least::VecNonEmpty;

#[derive(Debug, thiserror::Error)]
pub enum AttestationAttributePathsError {
    #[error("no attestation(s) provided")]
    EmptyAttestations,
    #[error("no attribute path(s) provided for attestation type(s): {}", .0.join(", "))]
    EmptyAttributes(Vec<String>),
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
            .collect_vec();

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
    // TODO: Implement test for AttestationAttributePaths::try_new().
}
