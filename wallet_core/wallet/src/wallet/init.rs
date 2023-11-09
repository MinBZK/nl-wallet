use tokio::sync::RwLock;

use platform_support::{
    hw_keystore::{hardware::HardwareEncryptionKey, PlatformEcdsaKey},
    utils::hardware::HardwareUtilities,
};

use crate::{
    account_provider::HttpAccountProviderClient,
    config::LocalConfigurationRepository,
    lock::WalletLock,
    pid_issuer::HttpPidIssuerClient,
    storage::{DatabaseStorage, RegistrationData, Storage, StorageError, StorageState},
};

use super::Wallet;

const WALLET_KEY_ID: &str = "wallet";

#[derive(Debug, thiserror::Error)]
pub enum WalletInitError {
    #[error("could not initialize database: {0}")]
    Database(#[from] StorageError),
}

impl Wallet {
    pub async fn init_all() -> Result<Self, WalletInitError> {
        #[cfg(feature = "disable_tls_validation")]
        tracing::warn!("TLS validation disabled");

        let storage = DatabaseStorage::<HardwareEncryptionKey>::init::<HardwareUtilities>().await?;

        Self::init_registration(
            LocalConfigurationRepository::default(),
            storage,
            HttpAccountProviderClient::default(),
            HttpPidIssuerClient::default(),
        )
        .await
    }
}

impl<CR, S, PEK, APC, DGS, PIC, MDS> Wallet<CR, S, PEK, APC, DGS, PIC, MDS>
where
    S: Storage,
    PEK: PlatformEcdsaKey,
{
    pub(super) fn new(
        config_repository: CR,
        storage: S,
        account_provider_client: APC,
        pid_issuer: PIC,
        registration: Option<RegistrationData>,
    ) -> Self {
        Wallet {
            config_repository,
            storage: RwLock::new(storage),
            hw_privkey: PEK::new(WALLET_KEY_ID),
            account_provider_client,
            digid_session: None,
            pid_issuer,
            disclosure_session: None,
            lock: WalletLock::new(true),
            registration,
            config_callback: None,
            documents_callback: None,
        }
    }

    /// Initialize the wallet by loading initial state.
    pub async fn init_registration(
        config_repository: CR,
        mut storage: S,
        account_provider_client: APC,
        pid_issuer: PIC,
    ) -> Result<Self, WalletInitError> {
        let registration = Self::fetch_registration(&mut storage).await?;

        let wallet = Self::new(
            config_repository,
            storage,
            account_provider_client,
            pid_issuer,
            registration,
        );

        Ok(wallet)
    }

    /// Attempts to fetch the registration data from storage, without creating a database if there is none.
    async fn fetch_registration(storage: &mut S) -> Result<Option<RegistrationData>, StorageError> {
        match storage.state().await? {
            // If there is no database file, we can conclude early that there is no registration.
            StorageState::Uninitialized => return Ok(None),
            // Open the database, if necessary.
            StorageState::Unopened => storage.open().await?,
            StorageState::Opened => (),
        }

        // Finally, fetch the registration.
        storage.fetch_data::<RegistrationData>().await
    }
}

#[cfg(test)]
mod tests {
    use crate::{pin::key as pin_key, storage::MockStorage};

    use super::{super::tests::WalletWithMocks, *};

    // Tests if the Wallet::init() method completes successfully with the mock generics.
    #[tokio::test]
    async fn test_wallet_init_registration() {
        let wallet = WalletWithMocks::init_registration_mocks()
            .await
            .expect("Could not initialize wallet");

        assert!(!wallet.has_registration());
    }

    // Tests the logic of fetching the wallet registration during init and its interaction with the database.
    #[tokio::test]
    async fn test_wallet_init_fetch_registration() {
        // Test with a wallet without a database file.
        let wallet = WalletWithMocks::init_registration_mocks()
            .await
            .expect("Could not initialize wallet");

        // The wallet should have no registration, and no database should be opened.
        assert!(wallet.registration.is_none());
        assert!(!wallet.has_registration());
        assert!(matches!(
            wallet.storage.read().await.state().await.unwrap(),
            StorageState::Uninitialized
        ));

        // The wallet should be locked by default
        assert!(wallet.is_locked());

        // Test with a wallet with a database file, no registration.
        let wallet =
            WalletWithMocks::init_registration_mocks_with_storage(MockStorage::mock(StorageState::Unopened, None))
                .await
                .expect("Could not initialize wallet");

        // The wallet should have no registration, the database should be opened.
        assert!(wallet.registration.is_none());
        assert!(!wallet.has_registration());
        assert!(matches!(
            wallet.storage.read().await.state().await.unwrap(),
            StorageState::Opened
        ));

        // Test with a wallet with a database file, contains registration.
        let pin_salt = pin_key::new_pin_salt();
        let wallet = WalletWithMocks::init_registration_mocks_with_storage(MockStorage::mock(
            StorageState::Unopened,
            Some(RegistrationData {
                pin_salt: pin_salt.clone().into(),
                wallet_certificate: "thisisjwt".to_string().into(),
            }),
        ))
        .await
        .expect("Could not initialize wallet");

        // The wallet should have a registration, the database should be opened.
        assert!(wallet.registration.is_some());
        assert!(wallet.has_registration());
        assert!(matches!(
            wallet.storage.read().await.state().await.unwrap(),
            StorageState::Opened
        ));

        // The registration data should now be available.
        assert_eq!(wallet.registration.unwrap().pin_salt.0, pin_salt);
    }
}
