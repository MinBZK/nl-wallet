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
pub enum IssuerSignedMatchingError {
    #[error("requested attributes are missing: {}", .0.iter().map(|path| {
        format!("[{}]", path.iter().join(", "))
    }).join(", "))]
    MissingAttributes(HashSet<VecNonEmpty<ClaimPath>>),
}
