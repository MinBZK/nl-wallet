use tracing::info;

use crate::{
    document::{Document, DocumentPersistence},
    storage::{Storage, StorageError},
};

use super::Wallet;

#[derive(Debug, thiserror::Error)]
pub enum SetDocumentsCallbackError {
    #[error("Could not fetch mdocs from database storage: {0}")]
    Storage(#[from] StorageError),
}

pub type DocumentsCallback = Box<dyn FnMut(Vec<Document>) + Send + Sync>;

impl<C, S, K, A, D, P> Wallet<C, S, K, A, D, P>
where
    S: Storage,
{
    pub(super) async fn emit_documents(&mut self) -> Result<(), StorageError> {
        info!("Emit mdocs from storage");

        let storage = self.storage.read().await;

        // Note that this currently panics whenever conversion from Mdoc to Documents fails,
        // as we assume that the (hardcoded) mapping will always be backwards compatible.
        // This is particularly important when this mapping comes from a trusted registry
        // in the near future!
        let mut documents = storage
            .fetch_unique_mdocs()
            .await?
            .into_iter()
            .map(|(id, mdoc)| {
                Document::from_mdoc_attributes(
                    DocumentPersistence::Stored(id.to_string()),
                    &mdoc.doc_type,
                    mdoc.attributes(),
                )
                .expect("Could not interpret stored mdoc attributes")
            })
            .collect::<Vec<_>>();

        documents.sort_by_key(Document::priority);

        if let Some(ref mut callback) = self.documents_callback {
            callback(documents);
        }

        Ok(())
    }

    pub async fn set_documents_callback<F>(&mut self, callback: F) -> Result<(), SetDocumentsCallbackError>
    where
        F: FnMut(Vec<Document>) + Send + Sync + 'static,
    {
        self.documents_callback.replace(Box::new(callback));

        // If the `Wallet` is not registered, the database will not be open.
        // In that case send an empty vec, so the UI has something to work with.
        //
        // TODO: have the UI not call this until after registration.
        if self.registration.is_some() {
            self.emit_documents().await?;
        } else {
            self.documents_callback.as_mut().unwrap()(Default::default());
        }

        Ok(())
    }

    pub fn clear_documents_callback(&mut self) {
        self.documents_callback.take();
    }
}
