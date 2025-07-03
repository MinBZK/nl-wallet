use std::collections::HashSet;

use dcql::CredentialQueryFormat;

use attestation_types::request::AttributeRequest;
use attestation_types::request::NormalizedCredentialRequests;

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

/// Return the mdoc-specific paths for a particular attestation type in [`NormalizedCredentialRequests`], which is
/// always a pair of namespace and element (i.e. attribute) identifier. Note that this may return an empty set, either
/// when the attestation type is not present or when none of the paths can be represented as a 2-tuple.
pub fn credential_request_to_mdoc_paths<'a>(
    credential_requests: &'a NormalizedCredentialRequests,
    attestation_type: &str,
) -> HashSet<(&'a str, &'a str)> {
    credential_requests
        .as_ref()
        .iter()
        .find(|request| {
            request.format
                == CredentialQueryFormat::MsoMdoc {
                    doctype_value: attestation_type.to_string(),
                }
        })
        .map(|request| {
            request
                .claims
                .iter()
                .map(AttributeRequest::to_namespace_and_attribute)
                .collect::<Result<HashSet<_>, _>>()
                .unwrap_or_default()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    // TODO: Implement test for attribute_paths_to_mdoc_paths().
}
