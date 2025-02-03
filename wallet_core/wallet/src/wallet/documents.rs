use tracing::info;

use error_category::sentry_capture_error;
use error_category::ErrorCategory;
use nl_wallet_mdoc::utils::cose::CoseError;
use nl_wallet_mdoc::utils::issuer_auth::IssuerRegistration;
use nl_wallet_mdoc::utils::x509::CertificateError;
use nl_wallet_mdoc::utils::x509::MdocCertificateExtension;
use platform_support::attested_key::AttestedKeyHolder;

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

impl<CR, UR, S, AKH, APC, DS, IS, MDS, WIC> Wallet<CR, UR, S, AKH, APC, DS, IS, MDS, WIC>
where
    S: Storage,
    AKH: AttestedKeyHolder,
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
                    mdoc.doc_type(),
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

        if self.registration.is_registered() {
            self.emit_documents().await?;
        }

        Ok(previous_callback)
    }

    pub fn clear_documents_callback(&mut self) -> Option<DocumentsCallback> {
        self.documents_callback.take()
    }
}
