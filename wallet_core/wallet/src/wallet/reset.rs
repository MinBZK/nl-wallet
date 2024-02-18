use tracing::{info, instrument, warn};

use platform_support::hw_keystore::PlatformEcdsaKey;

use crate::storage::Storage;

use super::Wallet;

impl<CR, S, PEK, APC, DGS, PIC, MDS> Wallet<CR, S, PEK, APC, DGS, PIC, MDS>
where
    S: Storage,
    PEK: PlatformEcdsaKey,
{
    #[instrument(skip_all)]
    pub async fn reset(mut self) {
        info!("Resetting wallet to inital state and wiping all local data");

        // Clear the database and its encryption key.
        self.storage.get_mut().clear().await;

        // Delete the hardware-backed private ECDSA key.
        if let Err(error) = self.hw_privkey.delete().await {
            warn!("Could not delete hardware private key: {0}", error);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use assert_matches::assert_matches;
    use wallet_common::keys::software::SoftwareEcdsaKey;

    use crate::storage::StorageState;

    use super::super::{init, test::WalletWithMocks};

    #[tokio::test]
    async fn test_wallet_reset() {
        // Test resetting a registered and unlocked Wallet.
        let wallet = WalletWithMocks::new_registered_and_unlocked().await;

        let storage_state = Arc::clone(&wallet.storage.read().await.state);

        // Check that the database is open and the hardware ECDSA key exists.
        assert_matches!(*storage_state.lock(), StorageState::Opened);
        assert!(SoftwareEcdsaKey::identifier_exists(init::wallet_key_id().as_ref()));

        // Reset the `Wallet`, which consumes it.
        wallet.reset().await;

        // The database should have been reset to uninitialized
        // and the hardware ECDSA key should no longer be present.
        assert_matches!(*storage_state.lock(), StorageState::Uninitialized);
        assert!(!SoftwareEcdsaKey::identifier_exists(init::wallet_key_id().as_ref()));
    }
}
