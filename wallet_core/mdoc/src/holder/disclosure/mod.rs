use std::collections::HashSet;

use async_trait::async_trait;

use super::Mdoc;

pub use session::{DisclosureMissingAttributes, DisclosureProposal, DisclosureSession, ProposedAttributes};

mod device_signed;
mod engagement;
mod issuer_signed;
mod proposed_document;
mod request;
mod session;

#[cfg(test)]
mod tests;

/// This trait needs to be implemented by an entity that stores mdocs.
#[async_trait]
pub trait MdocDataSource {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Return all `Mdoc` entries from storage that match a set of doc types.
    /// The result is a `Vec` of `Vec<Mdoc>` with the same `doc_type`. The order
    /// of the result is determined by the implementor.
    async fn mdoc_by_doc_types(&self, doc_types: &HashSet<&str>) -> std::result::Result<Vec<Vec<Mdoc>>, Self::Error>;
}
