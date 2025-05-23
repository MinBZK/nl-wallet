use tracing::info;

use attestation_data::auth::issuer_auth::IssuerRegistration;
use crypto::x509::BorrowingCertificateExtension;
use crypto::x509::CertificateError;
use error_category::sentry_capture_error;
use error_category::ErrorCategory;
use mdoc::utils::cose::CoseError;
use platform_support::attested_key::AttestedKeyHolder;

use crate::attestation::Attestation;
use crate::attestation::AttestationError;
use crate::attestation::AttestationIdentity;
use crate::storage::Storage;
use crate::storage::StorageError;
use crate::storage::StoredMdocCopy;

use super::Wallet;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum AttestationsError {
    #[error("could not fetch documents from database storage: {0}")]
    Storage(#[from] StorageError),

    #[error("could not extract Mdl extension from X.509 certificate: {0}")]
    Certificate(#[from] CertificateError),

    #[error("could not interpret X.509 certificate: {0}")]
    Cose(#[from] CoseError),

    #[error("X.509 certificate does not contain IssuerRegistration")]
    #[category(critical)]
    MissingIssuerRegistration,

    #[error("could not extract type metadata from mdoc: {0}")]
    #[category(defer)]
    Metadata(#[source] mdoc::Error),

    #[error("error converting credential payload to attestation: {0}")]
    #[category(defer)]
    Attestation(#[from] AttestationError),
}

pub type AttestationsCallback = Box<dyn FnMut(Vec<Attestation>) + Send + Sync>;

impl<CR, UR, S, AKH, APC, DS, IS, MDS, WIC> Wallet<CR, UR, S, AKH, APC, DS, IS, MDS, WIC>
where
    S: Storage,
    AKH: AttestedKeyHolder,
{
    pub(super) async fn emit_attestations(&mut self) -> Result<(), AttestationsError> {
        info!("Emit mdocs from storage");

        let storage = self.storage.read().await;

        let attestations = storage
            .fetch_unique_mdocs()
            .await?
            .into_iter()
            .map(|StoredMdocCopy { mdoc_id, mdoc, .. }| {
                let issuer_certificate = mdoc.issuer_certificate()?;
                let issuer_registration = IssuerRegistration::from_certificate(&issuer_certificate)?
                    .ok_or(AttestationsError::MissingIssuerRegistration)?;

                let metadata = mdoc.type_metadata().map_err(AttestationsError::Metadata)?;
                let attestation = Attestation::create_for_issuance(
                    AttestationIdentity::Fixed {
                        id: mdoc_id.to_string(),
                    },
                    metadata,
                    issuer_registration.organization,
                    mdoc.issuer_signed.into_entries_by_namespace(),
                )?;
                Ok(attestation)
            })
            .collect::<Result<Vec<_>, AttestationsError>>()?;

        if let Some(ref mut callback) = self.attestations_callback {
            callback(attestations);
        }

        Ok(())
    }

    #[sentry_capture_error]
    pub async fn set_attestations_callback(
        &mut self,
        callback: AttestationsCallback,
    ) -> Result<Option<AttestationsCallback>, AttestationsError> {
        let previous_callback = self.attestations_callback.replace(callback);

        if self.registration.is_registered() {
            self.emit_attestations().await?;
        }

        Ok(previous_callback)
    }

    pub fn clear_attestations_callback(&mut self) -> Option<AttestationsCallback> {
        self.attestations_callback.take()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use assert_matches::assert_matches;

    use sd_jwt_vc_metadata::NormalizedTypeMetadata;

    use super::super::test;
    use super::super::test::WalletDeviceVendor;
    use super::super::test::WalletWithMocks;
    use super::*;

    // Tests both setting and clearing the attestations callback on an unregistered `Wallet`.
    #[tokio::test]
    async fn test_wallet_set_clear_attestations_callback() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithMocks::new_unregistered(WalletDeviceVendor::Apple);

        // Register mock document_callback
        let attestations = test::setup_mock_attestations_callback(&mut wallet)
            .await
            .expect("Failed to set mock attestations callback");

        // Infer that the closure is still alive by counting the `Arc` references.
        assert_eq!(Arc::strong_count(&attestations), 2);

        // Confirm that the callback was not called.
        {
            let attestations = attestations.lock();

            assert!(attestations.is_empty());
        }

        // Clear the documents callback on the `Wallet.`
        wallet.clear_attestations_callback();

        // Infer that the closure is now dropped by counting the `Arc` references.
        assert_eq!(Arc::strong_count(&attestations), 1);
    }

    // Tests both setting and clearing the documents callback on a registered `Wallet`.
    #[tokio::test]
    async fn test_wallet_set_clear_documents_callback_registered() {
        let mut wallet = Wallet::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // The database contains a single `Mdoc`.
        let mdoc = test::create_example_pid_mdoc();
        let mdoc_doc_type = mdoc.doc_type().clone();
        wallet.storage.write().await.mdocs.insert(
            mdoc.doc_type().clone(),
            vec![(vec![mdoc].try_into().unwrap(), NormalizedTypeMetadata::pid_example())],
        );

        // Register mock document_callback
        let attestations = test::setup_mock_attestations_callback(&mut wallet)
            .await
            .expect("Failed to set mock attestations callback");

        // Infer that the closure is still alive by counting the `Arc` references.
        assert_eq!(Arc::strong_count(&attestations), 2);

        // Confirm that we received a single `Document` on the callback.
        {
            let attestations = attestations.lock();

            let attestation = attestations
                .first()
                .expect("Attestations callback should have been called")
                .first()
                .expect("Attestations callback should have been provided an Mdoc");
            assert_eq!(attestation.attestation_type, mdoc_doc_type);
        }

        // Clear the documents callback on the `Wallet.`
        wallet.clear_attestations_callback();

        // Infer that the closure is now dropped by counting the `Arc` references.
        assert_eq!(Arc::strong_count(&attestations), 1);
    }

    // Tests that setting the documents callback on a registered `Wallet`, with invalid issuer certificate raises
    // a `MissingIssuerRegistration` error.
    #[tokio::test]
    async fn test_wallet_set_clear_documents_callback_registered_no_issuer_registration() {
        let mut wallet = Wallet::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // The database contains a single `Mdoc`, without Issuer registration.
        let mdoc = test::create_example_pid_mdoc_unauthenticated();
        wallet.storage.write().await.mdocs.insert(
            mdoc.doc_type().clone(),
            vec![(vec![mdoc].try_into().unwrap(), NormalizedTypeMetadata::pid_example())],
        );

        // Register mock attestation_callback
        let (attestations, error) = test::setup_mock_attestations_callback(&mut wallet)
            .await
            .map(|_| ())
            .expect_err("Expected error");

        assert_matches!(error, AttestationsError::MissingIssuerRegistration);

        // Infer that the closure is still alive by counting the `Arc` references.
        assert_eq!(Arc::strong_count(&attestations), 2);
    }

    #[tokio::test]
    async fn test_wallet_set_attestations_callback_error() {
        let mut wallet = Wallet::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Have the database return an error on query.
        wallet.storage.write().await.has_query_error = true;

        // Confirm that setting the callback returns an error.
        let error = wallet
            .set_attestations_callback(Box::new(|_| {}))
            .await
            .map(|_| ())
            .expect_err("Setting attestations callback should have resulted in an error");

        assert_matches!(error, AttestationsError::Storage(_));
    }
}
