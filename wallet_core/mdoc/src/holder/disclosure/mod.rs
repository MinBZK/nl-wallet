use std::collections::HashSet;

use super::Mdoc;

pub use session::{DisclosureMissingAttributes, DisclosureProposal, DisclosureSession, ProposedAttributes};

mod device_signed;
mod engagement;
mod issuer_signed;
mod proposed_document;
mod request;
mod response;
mod session;

#[cfg(test)]
mod iso_tests;
#[cfg(test)]
mod test_utils;

#[derive(Debug, Clone)]
pub struct StoredMdoc<I> {
    pub id: I,
    pub mdoc: Mdoc,
}

/// This trait needs to be implemented by an entity that stores mdocs.
#[allow(async_fn_in_trait)]
pub trait MdocDataSource {
    type MdocIdentifier;
    type Error: std::error::Error + Send + Sync + 'static;

    /// Return all `Mdoc` entries from storage that match a set of doc types.
    /// The result is a `Vec` of `Vec<Mdoc>` with the same `doc_type`. The order
    /// of the result is determined by the implementor.
    async fn mdoc_by_doc_types(
        &self,
        doc_types: &HashSet<&str>,
    ) -> Result<Vec<Vec<StoredMdoc<Self::MdocIdentifier>>>, Self::Error>;
}
