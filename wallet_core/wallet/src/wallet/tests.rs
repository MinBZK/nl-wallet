use once_cell::sync::Lazy;
use p256::{ecdsa::SigningKey, elliptic_curve::rand_core::OsRng};

use wallet_common::{
    account::{
        jwt::Jwt,
        messages::auth::{WalletCertificate, WalletCertificateClaims},
    },
    keys::{software::SoftwareEcdsaKey, EcdsaKey},
    utils,
};

use crate::{
    account_provider::MockAccountProviderClient,
    config::LocalConfigurationRepository,
    digid::MockDigidSession,
    pid_issuer::MockPidIssuerClient,
    pin::key as pin_key,
    storage::{KeyedData, MockStorage, RegistrationData, StorageState},
    Configuration,
};

use super::{Wallet, WalletInitError};

/// This contains key material that is used to generate valid account server responses.
pub struct AccountServerKeys {
    pub certificate_signing_key: SigningKey,
}

pub type WalletWithMocks = Wallet<
    LocalConfigurationRepository,
    MockStorage,
    SoftwareEcdsaKey,
    MockAccountProviderClient,
    MockDigidSession,
    MockPidIssuerClient,
>;

/// The account server key material, generated once for testing.
pub static ACCOUNT_SERVER_KEYS: Lazy<AccountServerKeys> = Lazy::new(|| AccountServerKeys {
    certificate_signing_key: SigningKey::random(&mut OsRng),
});

impl WalletWithMocks {
    /// Creates a registered and unlocked `Wallet` with mock dependencies.
    pub async fn registered() -> Self {
        let mut wallet = Self::default();

        // Generate registration data.
        let registration = RegistrationData {
            pin_salt: pin_key::new_pin_salt().into(),
            wallet_certificate: wallet.valid_certificate().await,
        };

        // Store the registration in `Storage`, populate the field
        // on `Wallet` and set the wallet to unlocked.
        wallet.storage.get_mut().state = StorageState::Opened;
        wallet.storage.get_mut().data.insert(
            <RegistrationData as KeyedData>::KEY,
            serde_json::to_string(&registration).unwrap(),
        );
        wallet.registration = registration.into();
        wallet.lock.unlock();

        wallet
    }

    /// Generates a valid certificate for the `Wallet`.
    pub async fn valid_certificate(&self) -> WalletCertificate {
        Jwt::sign(
            &self.valid_certificate_claims().await,
            &ACCOUNT_SERVER_KEYS.certificate_signing_key,
        )
        .await
        .unwrap()
    }

    /// Generates valid certificate claims for the `Wallet`.
    pub async fn valid_certificate_claims(&self) -> WalletCertificateClaims {
        WalletCertificateClaims {
            wallet_id: utils::random_string(32),
            hw_pubkey: self.hw_privkey.verifying_key().await.unwrap().into(),
            pin_pubkey_hash: utils::random_bytes(32).into(),
            version: 0,
            iss: "wallet_unit_test".to_string(),
            iat: jsonwebtoken::get_current_timestamp(),
        }
    }

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
    /// Creates an unregistered `Wallet` with mock dependencies.
    fn default() -> Self {
        let keys = Lazy::force(&ACCOUNT_SERVER_KEYS);

        // Override public keys in the `Configuration`.
        let config = {
            let mut config = Configuration::default();
            config.account_server.certificate_public_key = (*keys.certificate_signing_key.verifying_key()).into();

            config
        };

        let config_repository = LocalConfigurationRepository::new(config);

        Wallet::new(
            config_repository,
            MockStorage::default(),
            MockAccountProviderClient::default(),
            MockPidIssuerClient::default(),
            None,
        )
    }
}
