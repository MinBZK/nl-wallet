use std::sync::Arc;

use cfg_if::cfg_if;
use futures::try_join;
use reqwest::ClientBuilder;
use tokio::sync::RwLock;

use error_category::ErrorCategory;
use error_category::sentry_capture_error;
use http_utils::reqwest::default_reqwest_client_builder;
use http_utils::tls::pinning::TlsPinningConfig;
use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::disclosure_session::VpDisclosureClient;
use platform_support::attested_key::AttestedKeyHolder;
use platform_support::hw_keystore::hardware::HardwareEncryptionKey;
use platform_support::utils::PlatformUtilities;
use platform_support::utils::UtilitiesError;
use platform_support::utils::hardware::HardwareUtilities;
use update_policy_model::update_policy::VersionState;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::config::ConfigurationError;
use crate::config::UpdatingConfigurationRepository;
use crate::config::WalletConfigurationRepository;
use crate::config::default_config_server_config;
use crate::config::default_wallet_config;
use crate::config::init_universal_link_base_url;
use crate::digid::DigidClient;
use crate::lock::WalletLock;
use crate::repository::BackgroundUpdateableRepository;
use crate::repository::Repository;
use crate::storage::DatabaseStorage;
use crate::storage::KeyData;
use crate::storage::RegistrationData;
use crate::storage::Storage;
use crate::storage::StorageError;
use crate::storage::StorageState;
use crate::update_policy::UpdatePolicyRepository;

use super::KeyHolderType;
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
    #[error("could not initialize HTTP client: {0}")]
    #[category(critical)]
    HttpClient(#[from] reqwest::Error),
}

#[cfg(feature = "fake_attestation")]
static KEY_HOLDER_ONCE: tokio::sync::OnceCell<()> = tokio::sync::OnceCell::const_new();

pub(super) enum RegistrationStatus {
    Unregistered,
    KeyIdentifierGenerated(String),
    Registered(RegistrationData),
}

#[cfg(feature = "fake_attestation")]
async fn init_mock_key_holder() -> platform_support::attested_key::mock::PersistentMockAttestedKeyHolder {
    use apple_app_attest::AttestationEnvironment;
    use platform_support::attested_key::mock::PersistentMockAttestedKeyHolder;

    // Initialize the key holder, but make sure this happens only once.
    KEY_HOLDER_ONCE
        .get_or_init(|| async {
            PersistentMockAttestedKeyHolder::init::<HardwareUtilities>().await;
        })
        .await;

    // Read the Apple attestation environment from an environment variable, defaulting to the development environment.
    let apple_attestation_environment = option_env!("APPLE_ATTESTATION_ENVIRONMENT")
        .map(|environment| match environment {
            "development" => AttestationEnvironment::Development,
            "production" => AttestationEnvironment::Production,
            _ => panic!("Invalid Apple attestation environment"),
        })
        .unwrap_or(AttestationEnvironment::Development);

    PersistentMockAttestedKeyHolder::new_mock_xcode(apple_attestation_environment)
}

impl<APC, DC, IS>
    Wallet<
        WalletConfigurationRepository,
        UpdatePolicyRepository,
        DatabaseStorage<HardwareEncryptionKey>,
        KeyHolderType,
        APC,
        DC,
        IS,
        VpDisclosureClient,
    >
where
    APC: Default,
    DC: DigidClient + Default,
{
    #[sentry_capture_error]
    pub async fn init_all() -> Result<Self, WalletInitError> {
        init_universal_link_base_url();

        // When using fake attestations, initialize the key holder, but make sure this happens only once.
        cfg_if! {
            if #[cfg(feature = "fake_attestation")] {
                let key_holder = init_mock_key_holder().await;
            } else {
                let key_holder = platform_support::attested_key::hardware::HardwareAttestedKeyHolder::default();
            }
        }

        let update_policy_repository = UpdatePolicyRepository::init();

        let storage_path = HardwareUtilities::storage_path().await?;
        let storage = DatabaseStorage::<HardwareEncryptionKey>::new(storage_path.clone());
        let config_repository = UpdatingConfigurationRepository::init(
            storage_path.clone(),
            default_config_server_config(),
            default_wallet_config(),
        )
        .await?;

        let wallet_clients = WalletClients::new_http(default_reqwest_client_builder())?;

        Self::init_registration(
            config_repository,
            update_policy_repository,
            storage,
            key_holder,
            wallet_clients,
        )
        .await
    }
}

#[derive(Debug, Default)]
pub struct WalletClients<APC, DC, DCC> {
    pub account_provider_client: APC,
    pub digid_client: DC,
    pub disclosure_client: DCC,
}

impl<APC, DC> WalletClients<APC, DC, VpDisclosureClient>
where
    APC: Default,
    DC: Default,
{
    pub fn new_http(disclosure_client_builder: ClientBuilder) -> Result<Self, reqwest::Error> {
        let disclosure_client = VpDisclosureClient::new_http(disclosure_client_builder)?;

        let clients = Self {
            account_provider_client: APC::default(),
            digid_client: DC::default(),
            disclosure_client,
        };

        Ok(clients)
    }
}

impl<CR, UR, S, AKH, APC, DC, IS, DCC> Wallet<CR, UR, S, AKH, APC, DC, IS, DCC>
where
    AKH: AttestedKeyHolder,
    DC: DigidClient,
    DCC: DisclosureClient,
{
    pub(super) fn new(
        config_repository: CR,
        update_policy_repository: UR,
        storage: S,
        key_holder: AKH,
        wallet_clients: WalletClients<APC, DC, DCC>,
        registration_status: RegistrationStatus,
    ) -> Self {
        let registration = match registration_status {
            RegistrationStatus::Unregistered => WalletRegistration::Unregistered,
            RegistrationStatus::KeyIdentifierGenerated(key_identifier) => {
                WalletRegistration::KeyIdentifierGenerated(key_identifier)
            }
            RegistrationStatus::Registered(data) => {
                // If the database contains a registration, an attested key
                // already exists and we can reference it by its identifier.
                // If a reference to this key already exists within the process
                // this is programmer error and should result in a panic.
                let attested_key = key_holder
                    .attested_key(data.attested_key_identifier.clone())
                    .expect("should be able to instantiate hardware attested key");

                WalletRegistration::Registered {
                    attested_key: Arc::new(attested_key),
                    data,
                }
            }
        };

        Wallet {
            config_repository,
            update_policy_repository,
            storage: Arc::new(RwLock::new(storage)),
            key_holder,
            registration,
            account_provider_client: Arc::new(wallet_clients.account_provider_client),
            digid_client: wallet_clients.digid_client,
            disclosure_client: wallet_clients.disclosure_client,
            session: None,
            lock: WalletLock::new(true),
            attestations_callback: None,
            recent_history_callback: None,
        }
    }

    /// Initialize the wallet by loading initial state.
    pub async fn init_registration(
        config_repository: CR,
        update_policy_repository: UR,
        mut storage: S,
        key_holder: AKH,
        wallet_clients: WalletClients<APC, DC, DCC>,
    ) -> Result<Self, WalletInitError>
    where
        CR: Repository<Arc<WalletConfiguration>>,
        UR: BackgroundUpdateableRepository<VersionState, TlsPinningConfig>,
        S: Storage,
    {
        let http_config = config_repository.get().update_policy_server.http_config.clone();
        update_policy_repository.fetch_in_background(http_config);

        let registration_status = Self::fetch_registration_status(&mut storage).await?;

        let wallet = Self::new(
            config_repository,
            update_policy_repository,
            storage,
            key_holder,
            wallet_clients,
            registration_status,
        );

        Ok(wallet)
    }
}

impl<CR, UR, S, AKH, APC, DC, IS, DCC> Wallet<CR, UR, S, AKH, APC, DC, IS, DCC>
where
    S: Storage,
    AKH: AttestedKeyHolder,
    DC: DigidClient,
    DCC: DisclosureClient,
{
    /// Attempts to fetch the initial data from storage, without creating a database if there is none.
    async fn fetch_registration_status(storage: &mut S) -> Result<RegistrationStatus, StorageError> {
        match storage.state().await? {
            // If there is no database file, we can conclude early that there is no registration.
            StorageState::Uninitialized => return Ok(RegistrationStatus::Unregistered),
            // Open the database, if necessary.
            StorageState::Unopened => storage.open().await?,
            StorageState::Opened => (),
        }

        let (key_data, registration_data) = try_join!(
            storage.fetch_data::<KeyData>(),
            storage.fetch_data::<RegistrationData>()
        )?;

        let registration_status = match (key_data, registration_data) {
            (None, None) => RegistrationStatus::Unregistered,
            (Some(key_data), None) => RegistrationStatus::KeyIdentifierGenerated(key_data.identifier),
            (_, Some(registration_data)) => RegistrationStatus::Registered(registration_data),
        };

        Ok(registration_status)
    }
}

#[cfg(test)]
mod tests {
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;

    use crate::pin::key as pin_key;
    use crate::storage::MockStorage;
    use crate::wallet::test::TestWalletMockStorage;
    use crate::wallet::test::generate_key_holder;

    use super::super::test;
    use super::super::test::WalletDeviceVendor;
    use super::*;

    // Tests if the `Wallet::init_registration()` method completes successfully with the mock generics.
    #[tokio::test]
    async fn test_wallet_init_registration() {
        let wallet = TestWalletMockStorage::new_init_registration(WalletDeviceVendor::Apple)
            .await
            .expect("Could not initialize wallet");

        assert!(!wallet.has_registration());
    }

    // Tests the initialization logic on a wallet without a database file.
    #[tokio::test]
    async fn test_wallet_init_fetch_registration_unopened_database() {
        let mut storage = MockStorage::default();
        storage.expect_state().returning(|| Ok(StorageState::Uninitialized));

        let wallet = TestWalletMockStorage::new_init_registration_with_mock_storage(
            generate_key_holder(WalletDeviceVendor::Apple),
            storage,
        )
        .await
        .expect("Could not initialize wallet");

        // The wallet should have no registration, and no database should be opened.
        assert!(!wallet.registration.is_registered());
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
        let mut storage = MockStorage::default();
        storage.expect_state().times(1).returning(|| Ok(StorageState::Unopened));
        storage.expect_open().returning(|| Ok(()));
        storage.expect_fetch_data::<RegistrationData>().returning(|| Ok(None));
        storage.expect_fetch_data::<KeyData>().returning(|| Ok(None));
        storage.expect_state().times(1).returning(|| Ok(StorageState::Opened));

        let wallet = TestWalletMockStorage::new_init_registration_with_mock_storage(
            generate_key_holder(WalletDeviceVendor::Apple),
            storage,
        )
        .await
        .expect("Could not initialize wallet");
        // The wallet should have no registration, the database should be opened.
        assert!(!wallet.registration.is_registered());
        assert!(!wallet.has_registration());
        assert!(matches!(
            wallet.storage.read().await.state().await.unwrap(),
            StorageState::Opened
        ));
    }

    // Tests the initialization logic on a wallet with a database file that contains a registration.
    #[tokio::test]
    async fn test_wallet_init_fetch_with_registration() {
        let key_holder = test::generate_key_holder(WalletDeviceVendor::Apple);
        key_holder.populate_key_identifier("key_id_123".to_string(), SigningKey::random(&mut OsRng));

        let pin_salt = pin_key::new_pin_salt();
        let expected_pin_salt = pin_salt.clone();

        let mut storage = MockStorage::default();
        storage.expect_state().times(1).returning(|| Ok(StorageState::Unopened));
        storage.expect_open().returning(|| Ok(()));
        storage.expect_fetch_data::<RegistrationData>().returning(move || {
            Ok(Some(RegistrationData {
                attested_key_identifier: "key_id_123".to_string(),
                pin_salt: expected_pin_salt.clone(),
                wallet_id: "wallet_123".to_string(),
                wallet_certificate: "thisisjwt".to_string().into(),
            }))
        });
        storage.expect_fetch_data::<KeyData>().returning(|| Ok(None));
        storage.expect_state().times(1).returning(|| Ok(StorageState::Opened));

        let wallet = TestWalletMockStorage::new_init_registration_with_mock_storage(key_holder, storage)
            .await
            .expect("Could not initialize wallet");

        // The wallet should have a registration, the database should be opened.
        assert!(wallet.registration.is_registered());
        assert!(wallet.has_registration());
        assert!(matches!(
            wallet.storage.read().await.state().await.unwrap(),
            StorageState::Opened
        ));

        // The registration data should now be available.
        let (_, registration_data) = wallet.registration.as_key_and_registration_data().unwrap();
        assert_eq!(registration_data.pin_salt, pin_salt);
    }

    #[tokio::test]
    #[should_panic]
    async fn test_wallet_init_fetch_with_registration_panic() {
        let mut storage = MockStorage::default();
        storage.expect_state().returning(|| Ok(StorageState::Unopened));

        let _ = TestWalletMockStorage::new_init_registration_with_mock_storage(
            generate_key_holder(WalletDeviceVendor::Apple),
            storage,
        )
        .await;
    }
}
