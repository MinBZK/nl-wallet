use std::{sync::Mutex, time::Duration};

use once_cell::sync::Lazy;
use openid4vc::mock::MockIssuerClient;
use p256::{
    ecdsa::{Signature, SigningKey, VerifyingKey},
    elliptic_curve::rand_core::OsRng,
};

use nl_wallet_mdoc::{
    basic_sa_ext::UnsignedMdoc, holder::Mdoc, mock as mdoc_mock, server_keys::PrivateKey, IssuerSigned,
};
use platform_support::hw_keystore::PlatformEcdsaKey;
use wallet_common::{
    account::messages::auth::{WalletCertificate, WalletCertificateClaims},
    generator::TimeGenerator,
    jwt::Jwt,
    keys::{software::SoftwareEcdsaKey, ConstructibleWithIdentifier, EcdsaKey, SecureEcdsaKey, WithIdentifier},
    trust_anchor::DerTrustAnchor,
    utils,
};

use crate::{
    account_provider::MockAccountProviderClient,
    config::{default_configuration, LocalConfigurationRepository, UpdatingConfigurationRepository},
    digid::MockDigidSession,
    disclosure::MockMdocDisclosureSession,
    document,
    pin::key as pin_key,
    storage::{KeyedData, MockStorage, RegistrationData, StorageState},
};

use super::{Wallet, WalletInitError};

/// This contains key material that is used to generate valid account server responses.
pub struct AccountServerKeys {
    pub certificate_signing_key: SigningKey,
    pub instruction_result_signing_key: SigningKey,
}

/// This contains key material that is used to issue mdocs.
pub struct IssuerKey {
    pub issuance_key: PrivateKey,
    pub trust_anchor: DerTrustAnchor,
}

/// This is used as a mock for `PlatformEcdsaKey`, so we can introduce failure conditions.
#[derive(Debug)]

pub struct FallibleSoftwareEcdsaKey {
    key: SoftwareEcdsaKey,
    pub next_public_key_error: Mutex<Option<<SoftwareEcdsaKey as EcdsaKey>::Error>>,
    pub next_private_key_error: Mutex<Option<<SoftwareEcdsaKey as EcdsaKey>::Error>>,
}

/// An alias for the `Wallet<>` with all mock dependencies.
pub type WalletWithMocks = Wallet<
    UpdatingConfigurationRepository<LocalConfigurationRepository>,
    MockStorage,
    FallibleSoftwareEcdsaKey,
    MockAccountProviderClient,
    MockDigidSession,
    MockIssuerClient,
    MockMdocDisclosureSession,
>;

/// The account server key material, generated once for testing.
pub static ACCOUNT_SERVER_KEYS: Lazy<AccountServerKeys> = Lazy::new(|| AccountServerKeys {
    certificate_signing_key: SigningKey::random(&mut OsRng),
    instruction_result_signing_key: SigningKey::random(&mut OsRng),
});

/// The issuer key material, generated once for testing.
pub static ISSUER_KEY: Lazy<IssuerKey> = Lazy::new(|| {
    let (issuance_key, ca) = mdoc_mock::generate_issuance_key_and_ca().unwrap();

    IssuerKey {
        issuance_key,
        trust_anchor: (&ca).try_into().unwrap(),
    }
});

/// The unauthenticated issuer key material, generated once for testing.
pub static ISSUER_KEY_UNAUTHENTICATED: Lazy<IssuerKey> = Lazy::new(|| {
    let (issuance_key, ca) = mdoc_mock::generate_issuance_key_and_ca_unauthenticated().unwrap();

    IssuerKey {
        issuance_key,
        trust_anchor: (&ca).try_into().unwrap(),
    }
});

/// Generates a valid `Mdoc` that contains a full PID.
pub async fn create_full_pid_mdoc() -> Mdoc {
    let private_key_id = utils::random_string(16);
    let unsigned_mdoc = document::create_full_unsigned_pid_mdoc();

    mdoc_from_unsigned(unsigned_mdoc, private_key_id, &ISSUER_KEY).await
}

/// Generates a valid `Mdoc` that contains a full PID, with an unauthenticated issuer certificate.
pub async fn create_full_pid_mdoc_unauthenticated() -> Mdoc {
    let private_key_id = utils::random_string(16);
    let unsigned_mdoc = document::create_full_unsigned_pid_mdoc();

    mdoc_from_unsigned(unsigned_mdoc, private_key_id, &ISSUER_KEY_UNAUTHENTICATED).await
}

/// Generates a valid `Mdoc`, based on an `UnsignedMdoc` and key identifier.
pub async fn mdoc_from_unsigned(unsigned_mdoc: UnsignedMdoc, private_key_id: String, issuer_key: &IssuerKey) -> Mdoc {
    let mdoc_public_key = (&SoftwareEcdsaKey::new(&private_key_id).verifying_key().await.unwrap())
        .try_into()
        .unwrap();
    let (issuer_signed, _) = IssuerSigned::sign(unsigned_mdoc, mdoc_public_key, &issuer_key.issuance_key)
        .await
        .unwrap();

    Mdoc::new::<SoftwareEcdsaKey>(
        private_key_id,
        issuer_signed,
        &TimeGenerator,
        &[(&issuer_key.trust_anchor.owned_trust_anchor).into()],
    )
    .unwrap()
}

// Implement traits for `FallibleSoftwareEcdsaKey` so all calls can be forwarded to `SoftwareEcdsaKey`.
impl From<SoftwareEcdsaKey> for FallibleSoftwareEcdsaKey {
    fn from(value: SoftwareEcdsaKey) -> Self {
        FallibleSoftwareEcdsaKey {
            key: value,
            next_public_key_error: Mutex::new(None),
            next_private_key_error: Mutex::new(None),
        }
    }
}

impl PlatformEcdsaKey for FallibleSoftwareEcdsaKey {}

impl ConstructibleWithIdentifier for FallibleSoftwareEcdsaKey {
    fn new(identifier: &str) -> Self {
        SoftwareEcdsaKey::new(identifier).into()
    }
}

impl WithIdentifier for FallibleSoftwareEcdsaKey {
    fn identifier(&self) -> &str {
        self.key.identifier()
    }
}

impl SecureEcdsaKey for FallibleSoftwareEcdsaKey {}

impl EcdsaKey for FallibleSoftwareEcdsaKey {
    type Error = <SoftwareEcdsaKey as EcdsaKey>::Error;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        let next_error = self.next_public_key_error.lock().unwrap().take();

        match next_error {
            None => self.key.verifying_key().await,
            Some(error) => Err(error),
        }
    }

    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error> {
        let next_error = self.next_private_key_error.lock().unwrap().take();

        match next_error {
            None => self.key.try_sign(msg).await,
            Some(error) => Err(error),
        }
    }
}

// Implement a number of methods on the the `Wallet<>` alias that can be used during testing.
impl WalletWithMocks {
    /// Creates an unregistered `Wallet` with mock dependencies.
    pub async fn new_unregistered() -> Self {
        let keys = Lazy::force(&ACCOUNT_SERVER_KEYS);

        // Override public key material in the `Configuration`.
        let config = {
            let mut config = default_configuration();

            config.account_server.certificate_public_key = (*keys.certificate_signing_key.verifying_key()).into();
            config.account_server.instruction_result_public_key =
                (*keys.instruction_result_signing_key.verifying_key()).into();

            config.mdoc_trust_anchors = vec![ISSUER_KEY.trust_anchor.clone()];

            config
        };

        let config_repository =
            UpdatingConfigurationRepository::new(LocalConfigurationRepository::new(config), Duration::from_secs(300))
                .await;

        Wallet::new(
            config_repository,
            MockStorage::default(),
            MockAccountProviderClient::default(),
            None,
        )
    }

    /// Creates a registered and unlocked `Wallet` with mock dependencies.
    pub async fn new_registered_and_unlocked() -> Self {
        let mut wallet = Self::new_unregistered().await;

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
        Jwt::sign_with_sub(
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
        let config_repository =
            UpdatingConfigurationRepository::new(LocalConfigurationRepository::default(), Duration::from_secs(300))
                .await;

        Wallet::init_registration(config_repository, storage, MockAccountProviderClient::default()).await
    }
}
