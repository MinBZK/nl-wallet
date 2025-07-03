use std::collections::HashSet;

use itertools::Itertools;

use attestation_types::attribute_paths::AttestationAttributePaths;

use crate::identifiers::AttributeIdentifier;

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

#[derive(Debug, thiserror::Error)]
pub enum ResponseValidationError {
    #[error("attributes mismatch: {0:?}")]
    MissingAttributes(Vec<AttributeIdentifier>),
    #[error("expected an mdoc")]
    ExpectedMdoc,
}

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
    // TODO: Implement test for attribute_paths_to_mdoc_paths().
}
