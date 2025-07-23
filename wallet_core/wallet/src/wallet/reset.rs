use std::mem;
use std::sync::Arc;

use tracing::info;
use tracing::instrument;
use tracing::warn;

use error_category::ErrorCategory;
use error_category::sentry_capture_error;
use openid4vc::disclosure_session::DisclosureClient;
use platform_support::attested_key::AttestedKey;
use platform_support::attested_key::AttestedKeyHolder;
use platform_support::attested_key::GoogleAttestedKey;
use update_policy_model::update_policy::VersionState;

use crate::digid::DigidClient;
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

type ResetResult<T> = Result<T, ResetError>;

impl<CR, UR, S, AKH, APC, DC, IS, DCC> Wallet<CR, UR, S, AKH, APC, DC, IS, DCC>
where
    UR: Repository<VersionState>,
    S: Storage,
    AKH: AttestedKeyHolder,
    DC: DigidClient,
    DCC: DisclosureClient,
{
    pub(super) async fn reset_to_initial_state(&mut self) -> bool {
        // Only reset if we actually have a registration. If we did generate a key but never
        // finished attestation, we can re-use this identifier in a later registration.
        if let WalletRegistration::Registered { attested_key, .. } = mem::take(&mut self.registration) {
            info!("Resetting wallet to inital state and wiping all local data");

            // Clear the database and its encryption key.
            self.storage.write().await.clear().await;

            // This is guaranteed to succeed for the following reasons:
            // * The reference count for the key is only ever incremented when sending an instruction.
            // * All instructions are sent and wrapped up within methods that take `&mut self`.
            // * This method takes `&mut self`, so an instruction can never be in flight at the same time.
            let attested_key = Arc::into_inner(attested_key)
                .expect("attested key should have no outstanding outside references to it on wallet reset");

            // Delete the hardware attested key if we are on Android, log any potential error.
            match attested_key {
                AttestedKey::Apple(_) => {}
                AttestedKey::Google(key) => {
                    if let Err(error) = key.delete().await {
                        warn!("Could not delete hardware attested key: {0}", error);
                    };
                }
            };

            self.session.take();

            // Send empty collections to both the attestations and recent history callbacks, if present.
            if let Some(ref mut attestations_callback) = self.attestations_callback {
                attestations_callback(vec![]);
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

    use crate::attestation::AttestationPresentation;
    use crate::storage::StorageState;
    use crate::wallet::Session;

    use super::super::issuance::WalletIssuanceSession;
    use super::super::test;
    use super::super::test::WalletDeviceVendor;
    use super::super::test::WalletWithMocks;
    use super::*;

    // TODO: Test key deletion for Google attested key.

    #[tokio::test]
    async fn test_wallet_reset() {
        // Test resetting a registered and unlocked Wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Register callbacks for both documents and history events and clear anything received on them.
        let attestations = test::setup_mock_attestations_callback(&mut wallet)
            .await
            .expect("Failed to set mock attestations callback");
        let events = test::setup_mock_recent_history_callback(&mut wallet)
            .await
            .expect("Failed to set mock recent history callback");

        attestations.lock().clear();
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
            wallet.storage.read().await.state().await.unwrap(),
            StorageState::Uninitialized
        );
        assert!(wallet.is_locked());

        // We should have received both an empty attestations and history events callback during the reset.
        let attestations = attestations.lock();
        assert_eq!(attestations.len(), 1);
        assert!(attestations.first().unwrap().is_empty());

        let events = events.lock();
        assert_eq!(events.len(), 1);
        assert!(events.first().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_wallet_reset_full() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);
        wallet.session = Some(Session::Issuance(WalletIssuanceSession::new(
            true,
            vec![AttestationPresentation::new_mock()].try_into().unwrap(),
            MockIssuanceSession::default(),
        )));

        wallet
            .reset()
            .await
            .expect("resetting the Wallet should have succeeded");

        // The wallet should now be totally cleared, even though the PidIssuerClient returned an error.
        assert!(!wallet.registration.is_registered());
        assert_matches!(
            wallet.storage.read().await.state().await.unwrap(),
            StorageState::Uninitialized
        );
        assert!(wallet.session.is_none());
        assert!(wallet.is_locked());
    }

    #[tokio::test]
    async fn test_wallet_reset_error_not_registered() {
        let mut wallet = WalletWithMocks::new_unregistered(WalletDeviceVendor::Apple);

        // Attempting to reset an unregistered Wallet should result in an error.
        let error = wallet
            .reset()
            .await
            .expect_err("resetting the Wallet should have resulted in an error");

        assert_matches!(error, ResetError::NotRegistered);
    }
}
