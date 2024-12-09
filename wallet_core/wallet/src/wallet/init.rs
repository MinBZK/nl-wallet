use tokio::sync::RwLock;

use error_category::sentry_capture_error;
use error_category::ErrorCategory;
use platform_support::attested_key::hardware::HardwareAttestedKeyHolder;
use platform_support::attested_key::AttestedKeyHolder;
use platform_support::hw_keystore::hardware::HardwareEncryptionKey;
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

        let key_holder = HardwareAttestedKeyHolder::default();
        let storage_path = HardwareUtilities::storage_path().await?;
        let storage = DatabaseStorage::<HardwareEncryptionKey>::new(storage_path.clone());
        let config_repository = UpdatingConfigurationRepository::init(
            storage_path,
            ConfigServerConfiguration::default(),
            default_configuration(),
        )
        .await?;

        Self::init_registration(
            config_repository,
            storage,
            key_holder,
            HttpAccountProviderClient::default(),
        )
        .await
    }
}

impl<CR, S, AKH, APC, DS, IS, MDS, WIC> Wallet<CR, S, AKH, APC, DS, IS, MDS, WIC>
where
    CR: ConfigurationRepository,
    S: Storage,
    AKH: AttestedKeyHolder,
    WIC: Default,
{
    pub(super) fn new(
        config_repository: CR,
        storage: S,
        key_holder: AKH,
        account_provider_client: APC,
        registration_data: Option<RegistrationData>,
    ) -> Self {
        let registration = registration_data.map(|data| {
            // If the database contains a registration, an attested key
            // already exists and we can reference it by its identifier.
            // If a reference to this key already exists within the process
            // this is programmer error and should result in a panic.
            let attested_key = key_holder
                .attested_key(data.attested_key_identifier.clone())
                .expect("should be able to instantiate hardware attested key");

            WalletRegistration { attested_key, data }
        });

        Wallet {
            config_repository,
            storage: RwLock::new(storage),
            key_holder,
            registration,
            account_provider_client,
            issuance_session: None,
            disclosure_session: None,
            wte_issuance_client: WIC::default(),
            lock: WalletLock::new(true),
            documents_callback: None,
            recent_history_callback: None,
        }
    }

    /// Initialize the wallet by loading initial state.
    pub async fn init_registration(
        config_repository: CR,
        mut storage: S,
        key_holder: AKH,
        account_provider_client: APC,
    ) -> Result<Self, WalletInitError> {
        let registration = Self::fetch_registration(&mut storage).await?;

        let wallet = Self::new(
            config_repository,
            storage,
            key_holder,
            account_provider_client,
            registration,
        );

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
}

#[cfg(test)]
mod tests {
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;

    use platform_support::attested_key::mock::MockAppleHardwareAttestedKeyHolder;

    use crate::pin::key as pin_key;
    use crate::storage::MockStorage;

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
        MockAppleHardwareAttestedKeyHolder::populate_key_identifier(
            "key_id_123".to_string(),
            SigningKey::random(&mut OsRng),
            1,
        );
        let pin_salt = pin_key::new_pin_salt();

        let wallet = WalletWithMocks::init_registration_mocks_with_storage(MockStorage::new(
            StorageState::Unopened,
            Some(RegistrationData {
                attested_key_identifier: "key_id_123".to_string(),
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

    #[tokio::test]
    #[should_panic]
    async fn test_wallet_init_fetch_with_registration_panic() {
        let _ = WalletWithMocks::init_registration_mocks_with_storage(MockStorage::new(
            StorageState::Unopened,
            Some(RegistrationData {
                attested_key_identifier: "key_id_321".to_string(),
                pin_salt: pin_key::new_pin_salt(),
                wallet_id: "wallet_123".to_string(),
                wallet_certificate: "thisisjwt".to_string().into(),
            }),
        ))
        .await;
    }
}
