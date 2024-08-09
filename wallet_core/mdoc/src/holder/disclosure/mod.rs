use std::{collections::HashSet, error::Error};

use super::Mdoc;

pub use disclosure_request_match::DisclosureRequestMatch;
pub use proposed_document::{ProposedAttributes, ProposedDocument, ProposedDocumentAttributes};

mod device_signed;
mod disclosure_request_match;
mod engagement;
mod issuer_signed;
mod proposed_document;
mod request;
mod response;
mod session;

#[cfg(test)]
mod iso_tests;
#[cfg(any(test, feature = "test"))]
pub mod test;

#[derive(Debug, Clone)]
pub struct StoredMdoc<I> {
    pub id: I,
    pub mdoc: Mdoc,
}

/// This trait needs to be implemented by an entity that stores mdocs.
pub trait MdocDataSource {
    type MdocIdentifier;
    type Error: Error + Send + Sync + 'static;

    /// Return all `Mdoc` entries from storage that match a set of doc types.
    /// The result is a `Vec` of `Vec<Mdoc>` with the same `doc_type`. The order
    /// of the result is determined by the implementor.
    async fn mdoc_by_doc_types(
        &self,
        doc_types: &HashSet<&str>,
    ) -> Result<Vec<Vec<StoredMdoc<Self::MdocIdentifier>>>, Self::Error>;
}
