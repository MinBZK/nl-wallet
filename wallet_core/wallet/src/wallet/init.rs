use tokio::sync::RwLock;

use error_category::sentry_capture_error;
use error_category::ErrorCategory;
use platform_support::hw_keystore::hardware::HardwareEncryptionKey;
use platform_support::hw_keystore::PlatformEcdsaKey;
use platform_support::utils::hardware::HardwareUtilities;
use platform_support::utils::PlatformUtilities;
use platform_support::utils::UtilitiesError;

use crate::account_provider::HttpAccountProviderClient;
use crate::config::default_configuration;
use crate::config::init_universal_link_base_url;
use crate::config::ConfigServerConfiguration;
use crate::config::ConfigurationError;
use crate::config::ConfigurationRepository;
use crate::config::UpdatingConfigurationRepository;
use crate::lock::WalletLock;
use crate::storage::DatabaseStorage;
use crate::storage::RegistrationData;
use crate::storage::Storage;
use crate::storage::StorageError;
use crate::storage::StorageState;

use super::Wallet;
use super::WalletRegistration;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum WalletInitError {
    #[error("wallet configuration error")]
    Configuration(#[from] ConfigurationError),
    #[error("platform utilities error: {0}")]
    Utilities(#[from] UtilitiesError),
    #[error("could not initialize database: {0}")]
    Database(#[from] StorageError),
}

impl Wallet {
    #[sentry_capture_error]
    pub async fn init_all() -> Result<Self, WalletInitError> {
        init_universal_link_base_url();

        let storage_path = HardwareUtilities::storage_path().await?;
        let storage = DatabaseStorage::<HardwareEncryptionKey>::new(storage_path.clone());
        let config_repository = UpdatingConfigurationRepository::init(
            storage_path,
            ConfigServerConfiguration::default(),
            default_configuration(),
        )
        .await?;

        Self::init_registration(config_repository, storage, HttpAccountProviderClient::default()).await
    }
}

impl<CR, S, PEK, APC, DS, IC, MDS, WIC> Wallet<CR, S, PEK, APC, DS, IC, MDS, WIC>
where
    CR: ConfigurationRepository,
    S: Storage,
    PEK: PlatformEcdsaKey,
    WIC: Default,
{
    pub(super) fn new(
        config_repository: CR,
        storage: S,
        account_provider_client: APC,
        registration_data: Option<RegistrationData>,
    ) -> Self {
        let registration = registration_data.map(|data| WalletRegistration {
            hw_privkey: Self::hw_privkey(),
            data,
        });

        Wallet {
            config_repository,
            storage: RwLock::new(storage),
            account_provider_client,
            issuance_session: None,
            disclosure_session: None,
            lock: WalletLock::new(true),
            registration,
            documents_callback: None,
            recent_history_callback: None,
            wte_issuance_client: WIC::default(),
        }
    }

    /// Initialize the wallet by loading initial state.
    pub async fn init_registration(
        config_repository: CR,
        mut storage: S,
        account_provider_client: APC,
    ) -> Result<Self, WalletInitError> {
        let registration = Self::fetch_registration(&mut storage).await?;

        let wallet = Self::new(config_repository, storage, account_provider_client, registration);

        Ok(wallet)
    }

    /// Attempts to fetch the initial data from storage, without creating a database if there is none.
    async fn fetch_registration(storage: &mut S) -> Result<Option<RegistrationData>, StorageError> {
        match storage.state().await? {
            // If there is no database file, we can conclude early that there is no registration.
            StorageState::Uninitialized => return Ok(Default::default()),
            // Open the database, if necessary.
            StorageState::Unopened => storage.open().await?,
            StorageState::Opened => (),
        }

        let result = storage.fetch_data::<RegistrationData>().await?;
        Ok(result)
    }

    /// Attempts to update the WalletRegistration from the database. Should be invoked after [`RegistrationData`] is
    /// stored in the database.
    pub(super) async fn update_registration_from_db(&mut self) -> Result<(), StorageError> {
        let storage = self.storage.read().await;
        if let Some(data) = storage.fetch_data::<RegistrationData>().await? {
            if let Some(registration) = self.registration.as_mut() {
                registration.data = data;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use wallet_common::keys::mock_hardware::MockHardwareEcdsaKey;
    use wallet_common::keys::EcdsaKey;

    use crate::pin::key as pin_key;
    use crate::storage::MockStorage;

    use super::super::registration;
    use super::super::test::WalletWithMocks;
    use super::*;

    // Tests if the `Wallet::init_registration()` method completes successfully with the mock generics.
    #[tokio::test]
    async fn test_wallet_init_registration() {
        let wallet = WalletWithMocks::init_registration_mocks()
            .await
            .expect("Could not initialize wallet");

        assert!(!wallet.has_registration());
    }

    // Tests the initialization logic on a wallet without a database file.
    #[tokio::test]
    async fn test_wallet_init_fetch_registration_no_database() {
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
    }

    // Tests the initialization logic on a wallet with a database file, but no registration.
    #[tokio::test]
    async fn test_wallet_init_fetch_registration_no_registration() {
        let wallet =
            WalletWithMocks::init_registration_mocks_with_storage(MockStorage::new(StorageState::Unopened, None))
                .await
                .expect("Could not initialize wallet");

        // The wallet should have no registration, the database should be opened.
        assert!(wallet.registration.is_none());
        assert!(!wallet.has_registration());
        assert!(matches!(
            wallet.storage.read().await.state().await.unwrap(),
            StorageState::Opened
        ));
    }

    // Tests the initialization logic on a wallet with a database file that contains a registration.
    #[tokio::test]
    async fn test_wallet_init_fetch_with_registration() {
        let pin_salt = pin_key::new_pin_salt();
        let wallet = WalletWithMocks::init_registration_mocks_with_storage(MockStorage::new(
            StorageState::Unopened,
            Some(RegistrationData {
                pin_salt: pin_salt.clone(),
                wallet_id: "wallet_123".to_string(),
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
        assert_eq!(wallet.registration.unwrap().data.pin_salt, pin_salt);
    }

    // Tests that the Wallet can be initialized multiple times and uses the same hardware key every time.
    #[tokio::test]
    async fn test_wallet_init_hw_privkey() {
        // The hardware private key should not exist at this point in the test.
        // In a real life scenario it does, as this test models a `Wallet` with
        // a pre-existing registration in its database.
        assert!(!MockHardwareEcdsaKey::identifier_exists(
            registration::wallet_key_id().as_ref()
        ));

        // Create a `Wallet` with `MockStorage`, then drop that `Wallet` again, while stealing
        // said `MockStorage` and getting the public key of the hardware private key.
        let (storage, hw_pubkey) = {
            let wallet = WalletWithMocks::init_registration_mocks_with_storage(MockStorage::new(
                StorageState::Unopened,
                Some(RegistrationData {
                    pin_salt: pin_key::new_pin_salt(),
                    wallet_id: "wallet_123".to_string(),
                    wallet_certificate: "thisisjwt".to_string().into(),
                }),
            ))
            .await
            .expect("Could not initialize wallet");

            let registration = wallet.registration.expect("Wallet should have registration");

            (
                wallet.storage.into_inner(),
                registration.hw_privkey.verifying_key().await.unwrap(),
            )
        };

        // The hardware private key should now exist.
        assert!(MockHardwareEcdsaKey::identifier_exists(
            registration::wallet_key_id().as_ref()
        ));

        // We should be able to create a new `Wallet`,
        // based on the contents of the `MockStorage`.
        let wallet = WalletWithMocks::init_registration_mocks_with_storage(storage)
            .await
            .expect("Could not initialize wallet a second time");
        let registration = wallet.registration.expect("Second Wallet should have registration");

        // The public keys of the hardware private key should match
        // that of the hardware private key of the previous instance.
        assert_eq!(registration.hw_privkey.verifying_key().await.unwrap(), hw_pubkey);
    }
}
