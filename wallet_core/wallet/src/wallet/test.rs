use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
    time::Duration,
};

use p256::ecdsa::{Signature, SigningKey, VerifyingKey};
use parking_lot::Mutex;
use rand_core::OsRng;

use nl_wallet_mdoc::{
    holder::Mdoc, server_keys::KeyPair, unsigned::UnsignedMdoc, utils::issuer_auth::IssuerRegistration, IssuerSigned,
};
use openid4vc::mock::MockIssuanceSession;
use platform_support::hw_keystore::PlatformEcdsaKey;
use wallet_common::{
    account::messages::auth::{WalletCertificate, WalletCertificateClaims},
    generator::TimeGenerator,
    jwt::Jwt,
    keys::{software::SoftwareEcdsaKey, EcdsaKey, SecureEcdsaKey, StoredByIdentifier, WithIdentifier},
    trust_anchor::DerTrustAnchor,
    utils,
};

use crate::{
    account_provider::MockAccountProviderClient,
    config::{default_configuration, LocalConfigurationRepository, UpdatingConfigurationRepository},
    disclosure::MockMdocDisclosureSession,
    document,
    issuance::MockDigidSession,
    pin::key as pin_key,
    storage::{KeyedData, MockStorage, RegistrationData, StorageState},
    wte::tests::MockWteIssuanceClient,
    Document, HistoryEvent,
};

use super::{documents::DocumentsError, HistoryError, Wallet, WalletInitError, WalletRegistration};

static FALLIBLE_KEY_ERRORS: LazyLock<Mutex<HashMap<String, FallibleSoftwareEcdsaKeyErrors>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// This contains key material that is used to generate valid account server responses.
pub struct AccountServerKeys {
    pub certificate_signing_key: SigningKey,
    pub instruction_result_signing_key: SigningKey,
}

/// This contains key material that is used to issue mdocs.
pub struct IssuerKey {
    pub issuance_key: KeyPair<SigningKey>,
    pub trust_anchor: DerTrustAnchor,
}

/// This is used as a mock for `PlatformEcdsaKey`, so we can introduce failure conditions.
#[derive(Debug)]
pub struct FallibleSoftwareEcdsaKey {
    key: SoftwareEcdsaKey,
    pub next_public_key_error: Mutex<Option<<SoftwareEcdsaKey as EcdsaKey>::Error>>,
    pub next_private_key_error: Mutex<Option<<SoftwareEcdsaKey as EcdsaKey>::Error>>,
}

struct FallibleSoftwareEcdsaKeyErrors {
    public_key_error: Option<<SoftwareEcdsaKey as EcdsaKey>::Error>,
    private_key_error: Option<<SoftwareEcdsaKey as EcdsaKey>::Error>,
}

/// An alias for the `Wallet<>` with all mock dependencies.
pub type WalletWithMocks = Wallet<
    UpdatingConfigurationRepository<LocalConfigurationRepository>,
    MockStorage,
    FallibleSoftwareEcdsaKey,
    MockAccountProviderClient,
    MockDigidSession,
    MockIssuanceSession,
    MockMdocDisclosureSession,
    MockWteIssuanceClient,
>;

/// The account server key material, generated once for testing.
pub static ACCOUNT_SERVER_KEYS: LazyLock<AccountServerKeys> = LazyLock::new(|| AccountServerKeys {
    certificate_signing_key: SigningKey::random(&mut OsRng),
    instruction_result_signing_key: SigningKey::random(&mut OsRng),
});

/// The issuer key material, generated once for testing.
pub static ISSUER_KEY: LazyLock<IssuerKey> = LazyLock::new(|| {
    let ca = KeyPair::generate_issuer_mock_ca().unwrap();
    let issuance_key = ca.generate_issuer_mock(IssuerRegistration::new_mock().into()).unwrap();

    IssuerKey {
        issuance_key,
        trust_anchor: ca.certificate().try_into().unwrap(),
    }
});

/// The unauthenticated issuer key material, generated once for testing.
pub static ISSUER_KEY_UNAUTHENTICATED: LazyLock<IssuerKey> = LazyLock::new(|| {
    let ca = KeyPair::generate_issuer_mock_ca().unwrap();
    let issuance_key = ca.generate_issuer_mock(None).unwrap();

    IssuerKey {
        issuance_key,
        trust_anchor: ca.certificate().try_into().unwrap(),
    }
});

/// Generates a valid `Mdoc` that contains a full PID.
pub async fn create_full_pid_mdoc() -> Mdoc {
    let unsigned_mdoc = document::create_full_unsigned_pid_mdoc();

    mdoc_from_unsigned(unsigned_mdoc, &ISSUER_KEY).await
}

/// Generates a valid `Mdoc` that contains a full PID, with an unauthenticated issuer certificate.
pub async fn create_full_pid_mdoc_unauthenticated() -> Mdoc {
    let unsigned_mdoc = document::create_full_unsigned_pid_mdoc();

    mdoc_from_unsigned(unsigned_mdoc, &ISSUER_KEY_UNAUTHENTICATED).await
}

/// Generates a valid `Mdoc`, based on an `UnsignedMdoc` and issuer key.
pub async fn mdoc_from_unsigned(unsigned_mdoc: UnsignedMdoc, issuer_key: &IssuerKey) -> Mdoc {
    let private_key_id = utils::random_string(16);
    let mdoc_public_key = (&SoftwareEcdsaKey::new_random(private_key_id.clone())
        .verifying_key()
        .await
        .unwrap())
        .try_into()
        .unwrap();
    let issuer_signed = IssuerSigned::sign(unsigned_mdoc, mdoc_public_key, &issuer_key.issuance_key)
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

impl FallibleSoftwareEcdsaKey {
    /// Sets the next public key error value for the next
    /// `FallibleSoftwareEcdsaKey` to be returned for the given identifier.
    pub fn next_public_key_error_for_identifier(identifier: String, error: <SoftwareEcdsaKey as EcdsaKey>::Error) {
        FALLIBLE_KEY_ERRORS.lock().insert(
            identifier,
            FallibleSoftwareEcdsaKeyErrors {
                public_key_error: error.into(),
                private_key_error: None,
            },
        );
    }

    /// Sets the next private key error value for the next
    /// `FallibleSoftwareEcdsaKey` to be returned for the given identifier.
    pub fn next_private_key_error_for_identifier(identifier: String, error: <SoftwareEcdsaKey as EcdsaKey>::Error) {
        FALLIBLE_KEY_ERRORS.lock().insert(
            identifier,
            FallibleSoftwareEcdsaKeyErrors {
                public_key_error: None,
                private_key_error: error.into(),
            },
        );
    }
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

impl StoredByIdentifier for FallibleSoftwareEcdsaKey {
    type Error = <SoftwareEcdsaKey as StoredByIdentifier>::Error;

    fn new_unique(identifier: &str) -> Option<Self> {
        let mut key = SoftwareEcdsaKey::new_unique(identifier).map(Self::from);

        if let Some(key) = key.as_mut() {
            if let Some(FallibleSoftwareEcdsaKeyErrors {
                public_key_error,
                private_key_error,
            }) = FALLIBLE_KEY_ERRORS.lock().remove(identifier)
            {
                key.next_public_key_error = public_key_error.into();
                key.next_private_key_error = private_key_error.into();
            }
        }

        key
    }

    async fn delete(self) -> Result<(), Self::Error> {
        self.key.delete().await
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
        let next_error = self.next_public_key_error.lock().take();

        match next_error {
            None => self.key.verifying_key().await,
            Some(error) => Err(error),
        }
    }

    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error> {
        let next_error = self.next_private_key_error.lock().take();

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
        let keys = LazyLock::force(&ACCOUNT_SERVER_KEYS);

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
        let registration_data = RegistrationData {
            pin_salt: pin_key::new_pin_salt(),
            wallet_certificate: Self::valid_certificate().await,
        };

        // Store the registration in `Storage`, populate the field
        // on `Wallet` and set the wallet to unlocked.
        wallet.storage.get_mut().state = StorageState::Opened;
        wallet.storage.get_mut().data.insert(
            <RegistrationData as KeyedData>::KEY,
            serde_json::to_string(&registration_data).unwrap(),
        );
        wallet.registration = WalletRegistration {
            hw_privkey: Self::hw_privkey(),
            data: registration_data,
        }
        .into();
        wallet.lock.unlock();

        wallet
    }

    /// Generates a valid certificate for the `Wallet`.
    pub async fn valid_certificate() -> WalletCertificate {
        Jwt::sign_with_sub(
            &Self::valid_certificate_claims().await,
            &ACCOUNT_SERVER_KEYS.certificate_signing_key,
        )
        .await
        .unwrap()
    }

    /// Generates valid certificate claims for the `Wallet`.
    pub async fn valid_certificate_claims() -> WalletCertificateClaims {
        WalletCertificateClaims {
            wallet_id: utils::random_string(32),
            hw_pubkey: Self::hw_privkey().verifying_key().await.unwrap().into(),
            pin_pubkey_hash: utils::random_bytes(32),
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

pub async fn setup_mock_documents_callback(
    wallet: &mut WalletWithMocks,
) -> Result<Arc<Mutex<Vec<Vec<Document>>>>, (Arc<Mutex<Vec<Vec<Document>>>>, DocumentsError)> {
    // Wrap a `Vec<Document>` in both a `Mutex` and `Arc`,
    // so we can write to it from the closure.
    let documents = Arc::new(Mutex::new(Vec::<Vec<Document>>::with_capacity(1)));
    let callback_documents = Arc::clone(&documents);

    // Set the documents callback on the `Wallet`, which
    // should immediately be called with an empty `Vec`.
    let result = wallet
        .set_documents_callback(Box::new(move |documents| {
            callback_documents.lock().push(documents.clone());
        }))
        .await;

    match result {
        Ok(_) => Ok(documents),
        Err(e) => Err((documents, e)),
    }
}

pub async fn setup_mock_recent_history_callback(
    wallet: &mut WalletWithMocks,
) -> Result<Arc<Mutex<Vec<Vec<HistoryEvent>>>>, (Arc<Mutex<Vec<Vec<HistoryEvent>>>>, HistoryError)> {
    // Wrap a `Vec<HistoryEvent>` in both a `Mutex` and `Arc`,
    // so we can write to it from the closure.
    let events = Arc::new(Mutex::new(Vec::<Vec<HistoryEvent>>::with_capacity(2)));
    let callback_events = Arc::clone(&events);

    // Set the recent_history callback on the `Wallet`, which should immediately be called with an empty `Vec`.
    let result = wallet
        .set_recent_history_callback(Box::new(move |events| callback_events.lock().push(events.clone())))
        .await;

    match result {
        Ok(_) => Ok(events),
        Err(e) => Err((events, e)),
    }
}
