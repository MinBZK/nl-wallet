use std::collections::HashSet;

use itertools::Itertools;

use attestation_types::claim_path::ClaimPath;
use utils::vec_at_least::VecNonEmpty;

mod device_response;
mod device_signed;
mod document;
mod issuer_signed;
mod mdoc;

#[cfg(test)]
mod device_retrieval;
#[cfg(test)]
mod iso_tests;

#[derive(Debug, thiserror::Error)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum ResponseMatchingError {
    #[error("attestation count in response does not match request: expected {expected}, found {found}")]
    AttestationCountMismatch { expected: usize, found: usize },
    #[error("at least one request was not for mdoc format")]
    FormatNotMdoc,
    #[error("received incorrect doc type: expected \"{expected}\", found \"{found}\"")]
    DocTypeMismatch { expected: String, found: String },
    #[error("requested attributes are missing for doc type(s): {}", .0.iter().map(|(attestation_type, paths)| {
        format!("({}): {}", attestation_type, paths.iter().map(|path| {
            format!("[{}]", path.iter().join(", "))
        }).join(", "))
    }).join(" / "))]
    MissingAttributes(Vec<(String, HashSet<VecNonEmpty<ClaimPath>>)>),
}

#[derive(Debug, thiserror::Error)]
pub enum IssuerSignedMatchingError {
    #[error("requested attributes are missing: {}", .0.iter().map(|path| {
        format!("[{}]", path.iter().join(", "))
    }).join(", "))]
    MissingAttributes(HashSet<VecNonEmpty<ClaimPath>>),
}
