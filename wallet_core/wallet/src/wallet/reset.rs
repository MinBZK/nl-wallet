use tracing::{info, instrument, warn};

use crate::{pid_issuer::PidIssuerClient, storage::Storage};

use super::Wallet;

#[derive(Debug, thiserror::Error)]
pub enum ResetError {
    #[error("wallet is not registered")]
    NotRegistered,
}

type ResetResult<T> = std::result::Result<T, ResetError>;

impl<CR, S, PEK, APC, DGS, PIC, MDS> Wallet<CR, S, PEK, APC, DGS, PIC, MDS>
where
    S: Storage,
    PIC: PidIssuerClient,
{
    pub(super) async fn reset_to_initial_state(&mut self) {
        info!("Resetting wallet to inital state and wiping all local data");

        // Clear the database and its encryption key.
        self.storage.get_mut().clear().await;

        // TODO: Reset the hardware private key and database key, as well as all credential keys.

        self.digid_session.take();
        self.disclosure_session.take();
        self.registration.take();

        if self.pid_issuer.has_session() {
            // Clear the PID issuer state by rejecting the PID.
            // Do not propagate if this results in an error.
            if let Err(error) = self.pid_issuer.reject_pid().await {
                warn!("Could not reject PID issuance: {0}", error);
            }
        }

        // The wallet should be locked in its initial state.
        self.lock.lock();
    }

    #[instrument(skip_all)]
    pub async fn reset(&mut self) -> ResetResult<()> {
        info!("Resetting of wallet requested");

        // Note that this method can be called even if the Wallet is locked!

        info!("Checking if registered");
        if self.registration.is_none() {
            return Err(ResetError::NotRegistered);
        }

        self.reset_to_initial_state().await;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use nl_wallet_mdoc::issuer_shared::IssuanceError;

    use crate::{
        digid::MockDigidSession, disclosure::MockMdocDisclosureSession, pid_issuer::PidIssuerError,
        storage::StorageState,
    };

    use super::{super::test::WalletWithMocks, *};

    #[tokio::test]
    async fn test_wallet_reset() {
        // Test resetting a registered and unlocked Wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        wallet
            .reset()
            .await
            .expect("resetting the Wallet should have succeeded");

        // The Wallet should now have an uninitialized database
        // and should be both unregistered and locked.
        assert_matches!(
            wallet.storage.get_mut().state().await.unwrap(),
            StorageState::Uninitialized
        );
        assert!(wallet.registration.is_none());
        assert!(wallet.is_locked());
    }

    #[tokio::test]
    async fn test_wallet_reset_full() {
        // Create the impossible Wallet that is doing everything at once and reset it.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;
        wallet.digid_session = MockDigidSession::new().into();
        wallet.disclosure_session = MockMdocDisclosureSession::default().into();
        wallet.pid_issuer.has_session = true;
        wallet.pid_issuer.next_error =
            PidIssuerError::MdocError(nl_wallet_mdoc::Error::Issuance(IssuanceError::SessionEnded)).into();

        wallet
            .reset()
            .await
            .expect("resetting the Wallet should have succeeded");

        // The wallet should now be totally cleared, even though the PidIssuerClient returned an error.
        assert_matches!(
            wallet.storage.get_mut().state().await.unwrap(),
            StorageState::Uninitialized
        );
        assert!(wallet.digid_session.is_none());
        assert!(wallet.disclosure_session.is_none());
        assert!(wallet.registration.is_none());
        assert!(wallet.is_locked());
    }

    #[tokio::test]
    async fn test_wallet_reset_error_not_registered() {
        let mut wallet = WalletWithMocks::new_unregistered().await;

        // Attempting to reset an unregistered Wallet should result in an error.
        let error = wallet
            .reset()
            .await
            .expect_err("resetting the Wallet should have resulted in an error");

        assert_matches!(error, ResetError::NotRegistered);
    }
}
