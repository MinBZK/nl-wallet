use wallet_common::keys::software::SoftwareEcdsaKey;

use crate::{
    account_provider::MockAccountProviderClient, config::LocalConfigurationRepository, digid::MockDigidSession,
    pid_issuer::MockPidIssuerClient, storage::MockStorage,
};

use super::{Wallet, WalletInitError};

pub type WalletWithMocks = Wallet<
    LocalConfigurationRepository,
    MockStorage,
    SoftwareEcdsaKey,
    MockAccountProviderClient,
    MockDigidSession,
    MockPidIssuerClient,
>;

impl WalletWithMocks {
    /// Creates all mocks and calls `Wallet::init_registration()`.
    pub async fn init_registration_mocks() -> Result<Self, WalletInitError> {
        Self::init_registration_mocks_with_storage(MockStorage::default()).await
    }

    /// Creates mocks and calls `Wallet::init_registration()`, except for the `MockStorage` instance.
    pub async fn init_registration_mocks_with_storage(storage: MockStorage) -> Result<Self, WalletInitError> {
        Wallet::init_registration(
            LocalConfigurationRepository::default(),
            storage,
            MockAccountProviderClient::default(),
            MockPidIssuerClient::default(),
        )
        .await
    }
}

impl Default for WalletWithMocks {
    fn default() -> Self {
        Wallet::new(
            LocalConfigurationRepository::default(),
            MockStorage::default(),
            MockAccountProviderClient::default(),
            MockPidIssuerClient::default(),
            None,
        )
    }
}
