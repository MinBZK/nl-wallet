use std::mem;

use tracing::info;
use tracing::instrument;
use tracing::warn;

use error_category::sentry_capture_error;
use error_category::ErrorCategory;
use platform_support::attested_key::AttestedKey;
use platform_support::attested_key::AttestedKeyHolder;
use platform_support::attested_key::GoogleAttestedKey;
use wallet_common::update_policy::VersionState;

use crate::repository::Repository;
use crate::storage::Storage;

use super::Wallet;
use super::WalletRegistration;

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum ResetError {
    #[category(expected)]
    #[error("app version is blocked")]
    VersionBlocked,
    #[error("wallet is not registered")]
    #[category(expected)]
    NotRegistered,
}

type ResetResult<T> = std::result::Result<T, ResetError>;

impl<CR, UR, S, AKH, APC, DS, IS, MDS, WIC> Wallet<CR, UR, S, AKH, APC, DS, IS, MDS, WIC>
where
    UR: Repository<VersionState>,
    S: Storage,
    AKH: AttestedKeyHolder,
{
    pub(super) async fn reset_to_initial_state(&mut self) -> bool {
        // Only reset if we actually have a registration. If we did generate a key but never
        // finished attestation, we can re-use this identifier in a later registration.
        if let WalletRegistration::Registered { attested_key, .. } = mem::take(&mut self.registration) {
            info!("Resetting wallet to inital state and wiping all local data");

            // Clear the database and its encryption key.
            self.storage.get_mut().clear().await;

            // Delete the hardware attested key if we are on Android, log any potential error.
            match attested_key {
                AttestedKey::Apple(_) => {}
                AttestedKey::Google(key) => {
                    if let Err(error) = key.delete().await {
                        warn!("Could not delete hardware attested key: {0}", error);
                    };
                }
            };

            self.issuance_session.take();
            self.disclosure_session.take();

            // Send empty collections to both the documents and recent history callbacks, if present.
            if let Some(ref mut documents_callback) = self.documents_callback {
                documents_callback(vec![]);
            }

            if let Some(ref mut recent_history_callback) = self.recent_history_callback {
                recent_history_callback(vec![]);
            }

            // The wallet should be locked in its initial state.
            self.lock.lock();

            return true;
        }

        false
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn reset(&mut self) -> ResetResult<()> {
        info!("Resetting of wallet requested");

        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(ResetError::VersionBlocked);
        }

        // Note that this method can be called even if the Wallet is locked!

        info!("Checking if registered");
        if !self.reset_to_initial_state().await {
            return Err(ResetError::NotRegistered);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    use openid4vc::mock::MockIssuanceSession;

    use crate::disclosure::MockMdocDisclosureSession;
    use crate::storage::StorageState;

    use super::super::issuance::PidIssuanceSession;
    use super::super::test::WalletWithMocks;
    use super::super::test::{self};
    use super::*;

    // TODO: Test key deletion for Google attested key.

    #[tokio::test]
    async fn test_wallet_reset() {
        // Test resetting a registered and unlocked Wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked_apple();

        // Register callbacks for both documents and history events and clear anything received on them.
        let documents = test::setup_mock_documents_callback(&mut wallet)
            .await
            .expect("Failed to set mock documents callback");
        let events = test::setup_mock_recent_history_callback(&mut wallet)
            .await
            .expect("Failed to set mock recent history callback");

        documents.lock().clear();
        events.lock().clear();

        // Check that the hardware key exists.
        wallet
            .reset()
            .await
            .expect("resetting the Wallet should have succeeded");

        // The database should now be uninitialized, the hardware key should
        // be gone and the `Wallet` should be both unregistered and locked.
        assert!(!wallet.registration.is_registered());
        assert_matches!(
            wallet.storage.get_mut().state().await.unwrap(),
            StorageState::Uninitialized
        );
        assert!(wallet.is_locked());

        // We should have received both an empty documents and history events callback during the reset.
        let documents = documents.lock();
        assert_eq!(documents.len(), 1);
        assert!(documents.first().unwrap().is_empty());

        let events = events.lock();
        assert_eq!(events.len(), 1);
        assert!(events.first().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_wallet_reset_full() {
        // Create the impossible Wallet that is doing everything at once and reset it.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked_apple();
        wallet.issuance_session = PidIssuanceSession::Openid4vci(MockIssuanceSession::default()).into();
        wallet.disclosure_session = MockMdocDisclosureSession::default().into();

        wallet
            .reset()
            .await
            .expect("resetting the Wallet should have succeeded");

        // The wallet should now be totally cleared, even though the PidIssuerClient returned an error.
        assert!(!wallet.registration.is_registered());
        assert_matches!(
            wallet.storage.get_mut().state().await.unwrap(),
            StorageState::Uninitialized
        );
        assert!(wallet.issuance_session.is_none());
        assert!(wallet.disclosure_session.is_none());
        assert!(wallet.is_locked());
    }

    #[tokio::test]
    async fn test_wallet_reset_error_not_registered() {
        let mut wallet = WalletWithMocks::new_unregistered();

        // Attempting to reset an unregistered Wallet should result in an error.
        let error = wallet
            .reset()
            .await
            .expect_err("resetting the Wallet should have resulted in an error");

        assert_matches!(error, ResetError::NotRegistered);
    }
}
