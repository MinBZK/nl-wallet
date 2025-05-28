use std::collections::HashSet;

use sd_jwt_vc_metadata::NormalizedTypeMetadata;

use crate::holder::Mdoc;

use super::MdocDataSource;
use super::StoredMdoc;

/// A type that implements `MdocDataSource` and simply returns
/// the [`Mdoc`] contained in `DeviceResponse::example()`, if its
/// `doc_type` is requested.
#[derive(Debug, Default)]
pub struct MockMdocDataSource {
    pub mdocs: Vec<(Mdoc, NormalizedTypeMetadata)>,
    pub has_error: bool,
}

impl MockMdocDataSource {
    pub fn new(mdocs: Vec<(Mdoc, NormalizedTypeMetadata)>) -> Self {
        Self {
            mdocs,
            has_error: false,
        }
    }

    #[cfg(any(test, feature = "mock_example_constructors"))]
    pub async fn new_example_resigned(ca: &crypto::server_keys::generate::Ca) -> Self {
        let mdoc = Mdoc::new_example_resigned(ca).await;

        Self::new(vec![(mdoc, NormalizedTypeMetadata::example())])
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MdocDataSourceError {
    #[error("failed")]
    Failed,
}

impl MdocDataSource for MockMdocDataSource {
    type MdocIdentifier = String;
    type Error = MdocDataSourceError;

    async fn mdoc_by_doc_types(
        &self,
        doc_types: &HashSet<&str>,
    ) -> std::result::Result<Vec<Vec<StoredMdoc<Self::MdocIdentifier>>>, Self::Error> {
        if self.has_error {
            return Err(MdocDataSourceError::Failed);
        }

        let stored_mdocs = self
            .mdocs
            .iter()
            .filter(|(mdoc, _normalized_metadata)| doc_types.contains(mdoc.doc_type().as_str()))
            .cloned()
            .enumerate()
            .map(|(index, (mdoc, normalized_metadata))| StoredMdoc {
                id: format!("id_{}", index + 1),
                mdoc,
                normalized_metadata,
            })
            .collect();

        Ok(vec![stored_mdocs])
    }
}
