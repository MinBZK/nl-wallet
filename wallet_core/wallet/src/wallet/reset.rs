use std::mem;

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
use wallet_account::messages::errors::AccountRevokedData;
use wallet_account::messages::errors::RevocationReason;

use crate::digid::DigidClient;
use crate::errors::InstructionError;
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

impl<CR, UR, S, AKH, APC, DC, IS, DCC, SLC> Wallet<CR, UR, S, AKH, APC, DC, IS, DCC, SLC>
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

            // Delete the hardware attested key if we are on Android, log any potential error.
            match attested_key.as_ref() {
                AttestedKey::Apple(_) => {}
                AttestedKey::Google(key) => {
                    if let Err(error) = key.delete().await {
                        warn!("Could not delete hardware attested key: {0}", error);
                    };
                }
            };

            self.session.take();

            // Send empty collections to both the attestations and recent history callbacks, if present.
            if let Some(ref mut attestations_callback) = self.attestations_callback.lock().as_deref_mut() {
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

    #[instrument(skip_all)]
    pub(crate) async fn check_result_for_wallet_revocation<T>(
        &mut self,
        result: Result<T, InstructionError>,
    ) -> Result<T, InstructionError> {
        match result {
            Err(error @ InstructionError::AccountRevoked(data)) => {
                self.handle_wallet_revocation(data).await;
                Err(error)
            }
            _ => Ok(result?),
        }
    }

    #[instrument(skip_all)]
    pub(crate) async fn handle_wallet_revocation(&mut self, revocation_data: AccountRevokedData) {
        info!("wallet has been revoked: {}", revocation_data.revocation_reason);
        if revocation_data.revocation_reason == RevocationReason::UserRequest {
            // In this case, the wallet user is probably not the owner of the wallet.
            // We wipe its contents to protect its from leaking/being stolen.
            info!("resetting wallet to initial state");
            self.reset_to_initial_state().await;
        } else {
            // In this case, store the fact that the wallet has been revoked so
            // self.get_state() can indicate so in its response.
            if let Err(writing_error) = self.storage.write().await.insert_data(&revocation_data).await {
                // Log the error but do nothing else.
                warn!("failed to write revocation reason to storage: {writing_error}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    use openid4vc::mock::MockIssuanceSession;

    use crate::PidIssuancePurpose;
    use crate::attestation::AttestationPresentation;
    use crate::errors::InstructionError;
    use crate::storage::StorageState;
    use crate::wallet::Session;
    use crate::wallet::test::TestWalletInMemoryStorage;

    use super::super::issuance::WalletIssuanceSession;
    use super::super::test;
    use super::super::test::TestWalletMockStorage;
    use super::super::test::WalletDeviceVendor;
    use super::*;

    // TODO: Test key deletion for Google attested key.

    #[tokio::test]
    async fn test_wallet_reset() {
        // Test resetting a registered and unlocked Wallet.
        let mut wallet = TestWalletInMemoryStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

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
        let mut wallet = TestWalletInMemoryStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        wallet.session = Some(Session::Issuance(WalletIssuanceSession::new(
            Some(PidIssuancePurpose::Enrollment),
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
        let mut wallet = TestWalletMockStorage::new_unregistered(WalletDeviceVendor::Apple).await;

        // Attempting to reset an unregistered Wallet should result in an error.
        let error = wallet
            .reset()
            .await
            .expect_err("resetting the Wallet should have resulted in an error");

        assert_matches!(error, ResetError::NotRegistered);
    }

    #[tokio::test]
    async fn test_check_result_for_wallet_revocation_ok() {
        let mut wallet = TestWalletInMemoryStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        // An Ok result should be passed through unchanged.
        let result = wallet
            .check_result_for_wallet_revocation(Ok::<u32, InstructionError>(42))
            .await;

        assert_matches!(result, Ok(42));
        assert!(wallet.registration.is_registered());
    }

    #[tokio::test]
    async fn test_check_result_for_wallet_revocation_other_error() {
        let mut wallet = TestWalletInMemoryStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        // A non-revocation error should be passed through unchanged without affecting the wallet.
        let result = wallet
            .check_result_for_wallet_revocation(Err::<(), _>(InstructionError::Blocked))
            .await;

        assert_matches!(result, Err(InstructionError::Blocked));
        assert!(wallet.registration.is_registered());
    }

    #[tokio::test]
    async fn test_check_result_for_wallet_revocation_user_request() {
        let mut wallet = TestWalletInMemoryStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        // A UserRequest revocation should reset the wallet to its initial (unregistered) state.
        let result = wallet
            .check_result_for_wallet_revocation(Err::<(), _>(InstructionError::AccountRevoked(AccountRevokedData {
                revocation_reason: RevocationReason::UserRequest,
                can_register_new_account: true,
            })))
            .await;

        assert_matches!(
            result,
            Err(InstructionError::AccountRevoked(AccountRevokedData {
                revocation_reason: RevocationReason::UserRequest,
                can_register_new_account: true,
            }))
        );
        assert!(!wallet.registration.is_registered());
        assert_matches!(
            wallet.storage.read().await.state().await.unwrap(),
            StorageState::Uninitialized
        );
    }

    #[tokio::test]
    async fn test_check_result_for_wallet_revocation_admin_request() {
        let mut wallet = TestWalletInMemoryStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        // An AdminRequest revocation should store the revocation reason without resetting the wallet.
        let result = wallet
            .check_result_for_wallet_revocation(Err::<(), _>(InstructionError::AccountRevoked(AccountRevokedData {
                revocation_reason: RevocationReason::AdminRequest,
                can_register_new_account: true,
            })))
            .await;

        assert_matches!(
            result,
            Err(InstructionError::AccountRevoked(AccountRevokedData {
                revocation_reason: RevocationReason::AdminRequest,
                can_register_new_account: true,
            }))
        );
        assert!(wallet.registration.is_registered());

        let revocation_data = wallet
            .storage
            .read()
            .await
            .fetch_data::<AccountRevokedData>()
            .await
            .unwrap();
        assert_matches!(
            revocation_data,
            Some(AccountRevokedData {
                revocation_reason: RevocationReason::AdminRequest,
                can_register_new_account: true,
            })
        );
    }
}
