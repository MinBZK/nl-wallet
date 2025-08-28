use tracing::info;

use error_category::ErrorCategory;
use error_category::sentry_capture_error;
use openid4vc::disclosure_session::DisclosureClient;
use platform_support::attested_key::AttestedKeyHolder;

use crate::attestation::AttestationPresentation;
use crate::digid::DigidClient;
use crate::storage::Storage;
use crate::storage::StorageError;
use crate::storage::StoredAttestationCopy;

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
    S: Storage,
    AKH: AttestedKeyHolder,
    DC: DigidClient,
    DCC: DisclosureClient,
{
    pub(super) async fn emit_attestations(&mut self) -> Result<(), AttestationsError> {
        info!("Emit attestations from storage");

        let storage = self.storage.read().await;

        let attestations = storage
            .fetch_unique_attestations()
            .await?
            .into_iter()
            .map(StoredAttestationCopy::into_attestation_presentation)
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

    use attestation_data::auth::issuer_auth::IssuerRegistration;
    use attestation_data::x509::generate::mock::generate_issuer_mock;
    use crypto::server_keys::generate::Ca;
    use sd_jwt::sd_jwt::VerifiedSdJwt;
    use sd_jwt_vc_metadata::NormalizedTypeMetadata;

    use crate::storage::StoredAttestation;
    use crate::wallet::test::create_example_pid_mdoc;

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
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuance_keypair = generate_issuer_mock(&ca, IssuerRegistration::new_mock().into()).unwrap();

        let sd_jwt = VerifiedSdJwt::pid_example(&issuance_keypair);
        let attestation_type = sd_jwt.as_ref().claims().vct.as_ref().unwrap().to_owned();

        let storage = wallet.mut_storage();
        storage.expect_fetch_unique_attestations().return_once(move || {
            Ok(vec![
                StoredAttestationCopy::new(
                    Uuid::new_v4(),
                    Uuid::new_v4(),
                    StoredAttestation::SdJwt {
                        sd_jwt: Box::new(sd_jwt),
                    },
                    NormalizedTypeMetadata::nl_pid_example(),
                ),
                StoredAttestationCopy::new(
                    Uuid::new_v4(),
                    Uuid::new_v4(),
                    StoredAttestation::MsoMdoc {
                        mdoc: Box::new(create_example_pid_mdoc()),
                    },
                    NormalizedTypeMetadata::nl_pid_example(),
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
                .expect("Attestations callback should have been called")
                .first()
                .expect("Attestations callback should have been provided an Mdoc");
            assert_eq!(attestation.attestation_type, attestation_type);
        }

        // Clear the documents callback on the `Wallet.`
        wallet.clear_attestations_callback();

        // Infer that the closure is now dropped by counting the `Arc` references.
        assert_eq!(Arc::strong_count(&attestations), 1);
    }

    #[tokio::test]
    async fn test_wallet_set_attestations_callback_error() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

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
