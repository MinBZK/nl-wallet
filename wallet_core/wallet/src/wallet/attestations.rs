use std::sync::Arc;

use tracing::info;

use error_category::ErrorCategory;
use error_category::sentry_capture_error;
use openid4vc::disclosure_session::DisclosureClient;
use platform_support::attested_key::AttestedKeyHolder;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::attestation::AttestationPresentation;
use crate::digid::DigidClient;
use crate::repository::Repository;
use crate::storage::Storage;
use crate::storage::StorageError;

use super::Wallet;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum AttestationsError {
    #[error("could not fetch documents from database storage: {0}")]
    Storage(#[from] StorageError),
}

pub type AttestationsCallback = Box<dyn FnMut(Vec<AttestationPresentation>) + Send + Sync>;

impl<CR, UR, S, AKH, APC, DC, IS, DCC> Wallet<CR, UR, S, AKH, APC, DC, IS, DCC>
where
    CR: Repository<Arc<WalletConfiguration>>,
    S: Storage,
    AKH: AttestedKeyHolder,
    DC: DigidClient,
    DCC: DisclosureClient,
{
    pub(super) async fn emit_attestations(&mut self) -> Result<(), AttestationsError> {
        info!("Emit attestations from storage");

        let wallet_config = self.config_repository.get();
        let storage = self.storage.read().await;

        let attestations = storage
            .fetch_unique_attestations()
            .await?
            .into_iter()
            .map(|copy| copy.into_attestation_presentation(&wallet_config.pid_attributes))
            .collect();

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
    use uuid::Uuid;

    use crate::storage::StoredAttestation;
    use crate::storage::StoredAttestationCopy;
    use crate::wallet::test::create_example_pid_mdoc;
    use crate::wallet::test::create_example_pid_sd_jwt;

    use super::super::test;
    use super::super::test::TestWalletMockStorage;
    use super::super::test::WalletDeviceVendor;
    use super::*;

    // Tests both setting and clearing the attestations callback on an unregistered `Wallet`.
    #[tokio::test]
    async fn test_wallet_set_clear_attestations_callback() {
        // Prepare an unregistered wallet.
        let mut wallet = TestWalletMockStorage::new_unregistered(WalletDeviceVendor::Apple).await;

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
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let (sd_jwt, sd_jwt_metadata) = create_example_pid_sd_jwt();
        let attestation_type = sd_jwt.claims().vct.clone();
        let (mdoc, mdoc_metadata) = create_example_pid_mdoc();

        let storage = wallet.mut_storage();
        storage.expect_fetch_unique_attestations().return_once(move || {
            Ok(vec![
                StoredAttestationCopy::new(
                    Uuid::new_v4(),
                    Uuid::new_v4(),
                    StoredAttestation::SdJwt {
                        key_identifier: "sd_jwt_key_id".to_string(),
                        sd_jwt,
                    },
                    sd_jwt_metadata,
                    None,
                ),
                StoredAttestationCopy::new(
                    Uuid::new_v4(),
                    Uuid::new_v4(),
                    StoredAttestation::MsoMdoc { mdoc },
                    mdoc_metadata,
                    None,
                ),
            ])
        });

        // Register mock document_callback
        let attestations = test::setup_mock_attestations_callback(&mut wallet)
            .await
            .expect("Failed to set mock attestations callback");

        // Infer that the closure is still alive by counting the `Arc` references.
        assert_eq!(Arc::strong_count(&attestations), 2);

        // Confirm that we received a single `Document` on the callback.
        {
            let attestations = attestations.lock();

            assert_eq!(attestations.first().unwrap().len(), 2);

            let attestation = attestations
                .first()
                .expect("attestations callback should have been called")
                .first()
                .expect("attestations callback should have been provided a document");
            assert_eq!(attestation.attestation_type, attestation_type);
        }

        // Clear the documents callback on the `Wallet.`
        wallet.clear_attestations_callback();

        // Infer that the closure is now dropped by counting the `Arc` references.
        assert_eq!(Arc::strong_count(&attestations), 1);
    }

    #[tokio::test]
    async fn test_wallet_set_attestations_callback_error() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        // Have the database return an error on query.
        let storage = wallet.mut_storage();
        storage
            .expect_fetch_unique_attestations()
            .returning(|| Err(StorageError::NotOpened));

        // Confirm that setting the callback returns an error.
        let error = wallet
            .set_attestations_callback(Box::new(|_| {}))
            .await
            .map(|_| ())
            .expect_err("Setting attestations callback should have resulted in an error");

        assert_matches!(error, AttestationsError::Storage(_));
    }
}
