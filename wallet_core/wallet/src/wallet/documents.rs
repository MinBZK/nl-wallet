use tracing::info;

use crate::{
    document::{Document, DocumentPersistence},
    storage::{Storage, StorageError, StoredMdocCopy},
};

use super::Wallet;

#[derive(Debug, thiserror::Error)]
pub enum SetDocumentsCallbackError {
    #[error("Could not fetch mdocs from database storage: {0}")]
    Storage(#[from] StorageError),
}

pub type DocumentsCallback = Box<dyn FnMut(Vec<Document>) + Send + Sync>;

impl<CR, S, PEK, APC, DGS, PIC, MDS> Wallet<CR, S, PEK, APC, DGS, PIC, MDS>
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
            .map(|StoredMdocCopy { mdoc_id, mdoc, .. }| {
                Document::from_mdoc_attributes(
                    DocumentPersistence::Stored(mdoc_id.to_string()),
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
#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use assert_matches::assert_matches;

    use super::{
        super::test::{self, WalletWithMocks},
        *,
    };

    // Tests both setting and clearing the documents callback on an unregistered `Wallet`.
    #[tokio::test]
    async fn test_wallet_set_clear_documents_callback() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithMocks::new_unregistered().await;

        // Wrap a `Vec<Document>` in both a `Mutex` and `Arc`,
        // so we can write to it from the closure.
        let documents = Arc::new(Mutex::new(Vec::<Vec<Document>>::with_capacity(1)));
        let callback_documents = Arc::clone(&documents);

        // Set the documents callback on the `Wallet`, which
        // should immediately be called with an empty `Vec`.
        wallet
            .set_documents_callback(move |documents| callback_documents.lock().unwrap().push(documents.clone()))
            .await
            .expect("Could not set documents callback");

        // Infer that the closure is still alive by counting the `Arc` references.
        assert_eq!(Arc::strong_count(&documents), 2);

        // Confirm that we received an empty `Vec` in the callback.
        {
            let documents = documents.lock().unwrap();

            assert_eq!(documents.len(), 1);
            assert!(documents.first().unwrap().is_empty());
        }

        // Clear the documents callback on the `Wallet.`
        wallet.clear_documents_callback();

        // Infer that the closure is now dropped by counting the `Arc` references.
        assert_eq!(Arc::strong_count(&documents), 1);
    }

    // Tests both setting and clearing the documents callback on a registered `Wallet`.
    #[tokio::test]
    async fn test_wallet_set_clear_documents_callback_registered() {
        let mut wallet = Wallet::new_registered_and_unlocked().await;

        // The database contains a single `Mdoc`.
        let mdoc = test::create_full_pid_mdoc().await;
        let mdoc_doc_type = mdoc.doc_type.clone();
        wallet.storage.get_mut().mdocs.add([mdoc].into_iter()).unwrap();

        // Wrap a `Vec<Document>` in both a `Mutex` and `Arc`,
        // so we can write to it from the closure.
        let documents = Arc::new(Mutex::new(Vec::<Vec<Document>>::with_capacity(1)));
        let callback_documents = Arc::clone(&documents);

        // Set the documents callback on the `Wallet`, which should
        // immediately be called with a `Vec` containing a single `Document`
        wallet
            .set_documents_callback(move |documents| callback_documents.lock().unwrap().push(documents.clone()))
            .await
            .expect("Could not set documents callback");

        // Infer that the closure is still alive by counting the `Arc` references.
        assert_eq!(Arc::strong_count(&documents), 2);

        // Confirm that we received a single `Document` on the callback.
        {
            let documents = documents.lock().unwrap();

            let document = documents
                .first()
                .expect("Documents callback should have been called")
                .first()
                .expect("Documents callback should have been provided an Mdoc");
            assert_eq!(document.doc_type, mdoc_doc_type);
        }

        // Clear the documents callback on the `Wallet.`
        wallet.clear_documents_callback();

        // Infer that the closure is now dropped by counting the `Arc` references.
        assert_eq!(Arc::strong_count(&documents), 1);
    }

    #[tokio::test]
    async fn test_wallet_set_documents_callback_error() {
        let mut wallet = Wallet::new_registered_and_unlocked().await;

        // Have the database return an error on query.
        wallet.storage.get_mut().has_query_error = true;

        // Confirm that setting the callback returns an error.
        let error = wallet
            .set_documents_callback(|_| {})
            .await
            .expect_err("Setting documents callback should have resulted in an error");

        assert_matches!(error, SetDocumentsCallbackError::Storage(_));
    }
}
