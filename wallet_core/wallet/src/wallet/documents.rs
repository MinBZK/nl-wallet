use tracing::info;

use error_category::sentry_capture_error;
use error_category::ErrorCategory;
use nl_wallet_mdoc::utils::cose::CoseError;
use nl_wallet_mdoc::utils::issuer_auth::IssuerRegistration;
use nl_wallet_mdoc::utils::x509::CertificateError;
use nl_wallet_mdoc::utils::x509::MdocCertificateExtension;

use crate::document::Document;
use crate::document::DocumentPersistence;
use crate::storage::Storage;
use crate::storage::StorageError;
use crate::storage::StoredMdocCopy;

use super::Wallet;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum DocumentsError {
    #[error("could not fetch documents from database storage: {0}")]
    Storage(#[from] StorageError),
    #[error("could not extract Mdl extension from X.509 certificate: {0}")]
    Certificate(#[from] CertificateError),
    #[error("could not interpret X.509 certificate: {0}")]
    Cose(#[from] CoseError),
    #[error("X.509 certificate does not contain IssuerRegistration")]
    #[category(critical)]
    MissingIssuerRegistration,
}

pub type DocumentsCallback = Box<dyn FnMut(Vec<Document>) + Send + Sync>;

impl<CR, S, PEK, APC, DS, IS, MDS, WIC, UR> Wallet<CR, S, PEK, APC, DS, IS, MDS, WIC, UR>
where
    S: Storage,
{
    pub(super) async fn emit_documents(&mut self) -> Result<(), DocumentsError> {
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
                let issuer_certificate = mdoc.issuer_certificate()?;
                let issuer_registration = IssuerRegistration::from_certificate(&issuer_certificate)?
                    .ok_or(DocumentsError::MissingIssuerRegistration)?;
                let document = Document::from_mdoc_attributes(
                    DocumentPersistence::Stored(mdoc_id.to_string()),
                    &mdoc.doc_type,
                    mdoc.attributes(),
                    issuer_registration,
                )
                .expect("Could not interpret stored mdoc attributes");
                Ok(document)
            })
            .collect::<Result<Vec<_>, DocumentsError>>()?;

        documents.sort_by_key(Document::priority);

        if let Some(ref mut callback) = self.documents_callback {
            callback(documents);
        }

        Ok(())
    }

    #[sentry_capture_error]
    pub async fn set_documents_callback(
        &mut self,
        callback: DocumentsCallback,
    ) -> Result<Option<DocumentsCallback>, DocumentsError> {
        let previous_callback = self.documents_callback.replace(callback);

        if self.registration.is_some() {
            self.emit_documents().await?;
        }

        Ok(previous_callback)
    }

    pub fn clear_documents_callback(&mut self) -> Option<DocumentsCallback> {
        self.documents_callback.take()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use assert_matches::assert_matches;

    use super::super::test::WalletWithMocks;
    use super::super::test::{self};
    use super::*;

    // Tests both setting and clearing the documents callback on an unregistered `Wallet`.
    #[tokio::test]
    async fn test_wallet_set_clear_documents_callback() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithMocks::new_unregistered().await;

        // Register mock document_callback
        let documents = test::setup_mock_documents_callback(&mut wallet)
            .await
            .expect("Failed to set mock documents callback");

        // Infer that the closure is still alive by counting the `Arc` references.
        assert_eq!(Arc::strong_count(&documents), 2);

        // Confirm that the callback was not called.
        {
            let documents = documents.lock();

            assert!(documents.is_empty());
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
        wallet
            .storage
            .get_mut()
            .mdocs
            .insert(mdoc.doc_type.clone(), vec![vec![mdoc].try_into().unwrap()]);

        // Register mock document_callback
        let documents = test::setup_mock_documents_callback(&mut wallet)
            .await
            .expect("Failed to set mock documents callback");

        // Infer that the closure is still alive by counting the `Arc` references.
        assert_eq!(Arc::strong_count(&documents), 2);

        // Confirm that we received a single `Document` on the callback.
        {
            let documents = documents.lock();

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

    // Tests that setting the documents callback on a registered `Wallet`, with invalid issuer certificate raises
    // a `MissingIssuerRegistration` error.
    #[tokio::test]
    async fn test_wallet_set_clear_documents_callback_registered_no_issuer_registration() {
        let mut wallet = Wallet::new_registered_and_unlocked().await;

        // The database contains a single `Mdoc`, without Issuer registration.
        let mdoc = test::create_full_pid_mdoc_unauthenticated().await;
        wallet
            .storage
            .get_mut()
            .mdocs
            .insert(mdoc.doc_type.clone(), vec![vec![mdoc].try_into().unwrap()]);

        // Register mock document_callback
        let (documents, error) = test::setup_mock_documents_callback(&mut wallet)
            .await
            .map(|_| ())
            .expect_err("Expected error");

        assert_matches!(error, DocumentsError::MissingIssuerRegistration);

        // Infer that the closure is still alive by counting the `Arc` references.
        assert_eq!(Arc::strong_count(&documents), 2);
    }

    #[tokio::test]
    async fn test_wallet_set_documents_callback_error() {
        let mut wallet = Wallet::new_registered_and_unlocked().await;

        // Have the database return an error on query.
        wallet.storage.get_mut().has_query_error = true;

        // Confirm that setting the callback returns an error.
        let error = wallet
            .set_documents_callback(Box::new(|_| {}))
            .await
            .map(|_| ())
            .expect_err("Setting documents callback should have resulted in an error");

        assert_matches!(error, DocumentsError::Storage(_));
    }
}
