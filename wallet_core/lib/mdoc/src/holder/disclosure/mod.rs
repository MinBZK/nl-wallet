use std::collections::HashSet;
use std::error::Error;

use sd_jwt_vc_metadata::NormalizedTypeMetadata;

use super::Mdoc;

pub use disclosure_request_match::DisclosureRequestMatch;
pub use proposed_document::ProposedAttributes;
pub use proposed_document::ProposedDocument;
pub use proposed_document::ProposedDocumentAttributes;

mod device_signed;
mod disclosure_request_match;
mod issuer_signed;
mod proposed_document;
mod request;
mod response;

#[cfg(test)]
mod iso_tests;
#[cfg(any(test, feature = "mock"))]
pub mod mock;

#[derive(Debug, Clone)]
pub struct StoredMdoc<I> {
    pub id: I,
    pub mdoc: Mdoc,
    pub normalized_metadata: NormalizedTypeMetadata,
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
