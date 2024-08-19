use std::collections::HashSet;

use crate::holder::Mdoc;

use super::{MdocDataSource, StoredMdoc};

/// A type that implements `MdocDataSource` and simply returns
/// the [`Mdoc`] contained in `DeviceResponse::example()`, if its
/// `doc_type` is requested.
#[derive(Debug)]
pub struct MockMdocDataSource {
    pub mdocs: Vec<Mdoc>,
    pub has_error: bool,
}
impl MockMdocDataSource {
    pub fn new() -> Self {
        Self {
            mdocs: vec![],
            has_error: false,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MdocDataSourceError {
    #[error("failed")]
    Failed,
}

#[cfg(any(test, feature = "examples"))]
impl Default for MockMdocDataSource {
    fn default() -> Self {
        MockMdocDataSource {
            mdocs: vec![Mdoc::new_example_mock()],
            has_error: false,
        }
    }
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
            .filter(|mdoc| doc_types.contains(mdoc.doc_type.as_str()))
            .cloned()
            .enumerate()
            .map(|(index, mdoc)| StoredMdoc {
                id: format!("id_{}", index + 1),
                mdoc,
            })
            .collect();

        Ok(vec![stored_mdocs])
    }
}
