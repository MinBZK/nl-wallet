use std::sync::Arc;
use std::sync::LazyLock;

use futures::future::FutureExt;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use parking_lot::Mutex;
use rand_core::OsRng;

use apple_app_attest::AppIdentifier;
use apple_app_attest::AttestationEnvironment;
use nl_wallet_mdoc::holder::Mdoc;
use nl_wallet_mdoc::server_keys::generate::Ca;
use nl_wallet_mdoc::server_keys::KeyPair;
use nl_wallet_mdoc::unsigned::UnsignedMdoc;
use nl_wallet_mdoc::utils::issuer_auth::IssuerRegistration;
use nl_wallet_mdoc::IssuerSigned;
use openid4vc::mock::MockIssuanceSession;
use platform_support::attested_key::mock::MockHardwareAttestedKeyHolder;
use platform_support::attested_key::AttestedKey;
use sd_jwt::metadata::TypeMetadata;
use sd_jwt::metadata::TypeMetadataChain;
use wallet_account::messages::registration::WalletCertificate;
use wallet_account::messages::registration::WalletCertificateClaims;
use wallet_common::generator::TimeGenerator;
use wallet_common::jwt::Jwt;
use wallet_common::keys::mock_remote::MockRemoteEcdsaKey;
use wallet_common::p256_der::DerVerifyingKey;
use wallet_common::trust_anchor::BorrowingTrustAnchor;
use wallet_common::utils;

use crate::account_provider::MockAccountProviderClient;
use crate::config::default_config_server_config;
use crate::config::default_wallet_config;
use crate::config::LocalConfigurationRepository;
use crate::config::UpdatingConfigurationRepository;
use crate::disclosure::MockMdocDisclosureSession;
use crate::issuance;
use crate::issuance::MockDigidSession;
use crate::pin::key as pin_key;
use crate::storage::KeyedData;
use crate::storage::KeyedDataResult;
use crate::storage::MockStorage;
use crate::storage::RegistrationData;
use crate::storage::StorageState;
use crate::update_policy::MockUpdatePolicyRepository;
use crate::wallet::attestations::AttestationsError;
use crate::wte::tests::MockWteIssuanceClient;
use crate::Attestation;
use crate::WalletEvent;

use super::init::RegistrationStatus;
use super::HistoryError;
use super::Wallet;
use super::WalletInitError;
use super::WalletRegistration;

/// This contains key material that is used to generate valid account server responses.
pub struct AccountServerKeys {
    pub certificate_signing_key: SigningKey,
    pub instruction_result_signing_key: SigningKey,
}

/// This contains key material that is used to issue mdocs.
pub struct IssuerKey {
    pub issuance_key: KeyPair,
    pub trust_anchor: BorrowingTrustAnchor,
}

#[derive(Debug, Clone, Copy)]
pub enum WalletDeviceVendor {
    Apple,
    Google,
}

/// An alias for the `Wallet<>` with all mock dependencies.
pub type WalletWithMocks = Wallet<
    UpdatingConfigurationRepository<LocalConfigurationRepository>,
    MockUpdatePolicyRepository,
    MockStorage,
    MockHardwareAttestedKeyHolder,
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
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let issuance_key = ca.generate_issuer_mock(IssuerRegistration::new_mock().into()).unwrap();
    let trust_anchor = ca.as_borrowing_trust_anchor().clone();

    IssuerKey {
        issuance_key,
        trust_anchor,
    }
});

/// The unauthenticated issuer key material, generated once for testing.
pub static ISSUER_KEY_UNAUTHENTICATED: LazyLock<IssuerKey> = LazyLock::new(|| {
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let issuance_key = ca.generate_issuer_mock(None).unwrap();
    let trust_anchor = ca.as_borrowing_trust_anchor().clone();

    IssuerKey {
        issuance_key,
        trust_anchor,
    }
});

/// Generates a valid `Mdoc` that contains a full PID.
pub fn create_example_pid_mdoc() -> Mdoc {
    let (unsigned_mdoc, metadata) = issuance::mock::create_example_unsigned_mdoc();

    mdoc_from_unsigned(unsigned_mdoc, &metadata, &ISSUER_KEY)
}

/// Generates a valid `Mdoc` that contains a full PID, with an unauthenticated issuer certificate.
pub fn create_example_pid_mdoc_unauthenticated() -> Mdoc {
    let (unsigned_mdoc, metadata) = issuance::mock::create_example_unsigned_mdoc();

    mdoc_from_unsigned(unsigned_mdoc, &metadata, &ISSUER_KEY_UNAUTHENTICATED)
}

/// Generates a valid `Mdoc`, based on an `UnsignedMdoc`, the `TypeMetadata` and issuer key.
pub fn mdoc_from_unsigned(unsigned_mdoc: UnsignedMdoc, metadata: &TypeMetadata, issuer_key: &IssuerKey) -> Mdoc {
    let private_key_id = utils::random_string(16);
    let mdoc_remote_key = MockRemoteEcdsaKey::new_random(private_key_id.clone());
    let mdoc_public_key = mdoc_remote_key.verifying_key().try_into().unwrap();
    let metadata_chain = TypeMetadataChain::create(metadata.clone(), vec![]).unwrap();
    let issuer_signed = IssuerSigned::sign(unsigned_mdoc, metadata_chain, mdoc_public_key, &issuer_key.issuance_key)
        .now_or_never()
        .unwrap()
        .unwrap();

    Mdoc::new::<MockRemoteEcdsaKey>(
        private_key_id,
        issuer_signed,
        &TimeGenerator,
        &[issuer_key.trust_anchor.as_trust_anchor().clone()],
    )
    .unwrap()
}

pub fn generate_key_holder(vendor: WalletDeviceVendor) -> MockHardwareAttestedKeyHolder {
    match vendor {
        WalletDeviceVendor::Apple => MockHardwareAttestedKeyHolder::generate_apple(
            AttestationEnvironment::Development,
            AppIdentifier::new_mock(),
        ),
        WalletDeviceVendor::Google => MockHardwareAttestedKeyHolder::generate_google(),
    }
}

// Implement a number of methods on the the `Wallet<>` alias that can be used during testing.
impl WalletWithMocks {
    /// Creates an unregistered `Wallet` with mock dependencies.
    pub fn new_unregistered(vendor: WalletDeviceVendor) -> Self {
        let keys = LazyLock::force(&ACCOUNT_SERVER_KEYS);

        // Override public key material in the `Configuration`.
        let config = {
            let mut config = default_wallet_config();

            config.account_server.certificate_public_key = (*keys.certificate_signing_key.verifying_key()).into();
            config.account_server.instruction_result_public_key =
                (*keys.instruction_result_signing_key.verifying_key()).into();

            config.mdoc_trust_anchors = vec![ISSUER_KEY.trust_anchor.clone()];

            config
        };

        let config_server_config = default_config_server_config();
        let config_repository =
            UpdatingConfigurationRepository::new(LocalConfigurationRepository::new(config), config_server_config)
                .now_or_never()
                .unwrap();

        Wallet::new(
            config_repository,
            MockUpdatePolicyRepository::default(),
            MockStorage::default(),
            generate_key_holder(vendor),
            MockAccountProviderClient::default(),
            RegistrationStatus::Unregistered,
        )
    }

    /// Creates a registered and unlocked `Wallet` with mock dependencies.
    pub fn new_registered_and_unlocked(vendor: WalletDeviceVendor) -> Self {
        let mut wallet = Self::new_unregistered(vendor);

        let (attested_key, attested_key_identifier) = wallet.key_holder.random_key();
        let verifying_key = match &attested_key {
            AttestedKey::Apple(key) => *key.verifying_key(),
            AttestedKey::Google(key) => *key.verifying_key(),
        };
        let wallet_certificate = Self::valid_certificate(None, verifying_key);
        let wallet_id = wallet_certificate.dangerous_parse_unverified().unwrap().1.wallet_id;

        // Generate registration data.
        let registration_data = RegistrationData {
            attested_key_identifier,
            pin_salt: pin_key::new_pin_salt(),
            wallet_id,
            wallet_certificate,
        };

        // Store the registration in `Storage`, populate the field
        // on `Wallet` and set the wallet to unlocked.
        let storage = Arc::get_mut(&mut wallet.storage).unwrap().get_mut();
        storage.state = StorageState::Opened;
        storage.data.insert(
            <RegistrationData as KeyedData>::KEY,
            KeyedDataResult::Data(serde_json::to_string(&registration_data).unwrap()),
        );
        wallet.registration = WalletRegistration::Registered {
            attested_key: Arc::new(attested_key),
            data: registration_data,
        };
        wallet.lock.unlock();

        wallet
    }

    /// Generates a valid certificate for the `Wallet`.
    pub fn valid_certificate(wallet_id: Option<String>, hw_pubkey: VerifyingKey) -> WalletCertificate {
        Jwt::sign_with_sub(
            &Self::valid_certificate_claims(wallet_id, hw_pubkey),
            &ACCOUNT_SERVER_KEYS.certificate_signing_key,
        )
        .now_or_never()
        .unwrap()
        .unwrap()
    }

    /// Generates valid certificate claims for the `Wallet`.
    pub fn valid_certificate_claims(wallet_id: Option<String>, hw_pubkey: VerifyingKey) -> WalletCertificateClaims {
        let wallet_id = wallet_id.unwrap_or_else(|| utils::random_string(32));

        WalletCertificateClaims {
            wallet_id,
            hw_pubkey: DerVerifyingKey::from(hw_pubkey),
            pin_pubkey_hash: utils::random_bytes(32),
            version: 0,
            iss: "wallet_unit_test".to_string(),
            iat: jsonwebtoken::get_current_timestamp(),
        }
    }

    /// Creates all mocks and calls `Wallet::init_registration()`.
    pub async fn new_init_registration(vendor: WalletDeviceVendor) -> Result<Self, WalletInitError> {
        Self::new_init_registration_with_mocks(MockStorage::default(), generate_key_holder(vendor)).await
    }

    /// Creates mocks and calls `Wallet::init_registration()`, except for
    /// the `MockStorage` and `MockHardwareAttestedKeyHolder` instances.
    pub async fn new_init_registration_with_mocks(
        storage: MockStorage,
        key_holder: MockHardwareAttestedKeyHolder,
    ) -> Result<Self, WalletInitError> {
        let config_server_config = default_config_server_config();
        let config_repository =
            UpdatingConfigurationRepository::new(LocalConfigurationRepository::default(), config_server_config).await;

        Wallet::init_registration(
            config_repository,
            MockUpdatePolicyRepository::default(),
            storage,
            key_holder,
            MockAccountProviderClient::default(),
        )
        .await
    }
}

pub async fn setup_mock_attestations_callback(
    wallet: &mut WalletWithMocks,
) -> Result<Arc<Mutex<Vec<Vec<Attestation>>>>, (Arc<Mutex<Vec<Vec<Attestation>>>>, AttestationsError)> {
    // Wrap a `Vec<Attestation>` in both a `Mutex` and `Arc`,
    // so we can write to it from the closure.
    let attestations = Arc::new(Mutex::new(Vec::<Vec<Attestation>>::with_capacity(1)));
    let callback_attestations = Arc::clone(&attestations);

    // Set the attestations callback on the `Wallet`, which
    // should immediately be called with an empty `Vec`.
    let result = wallet
        .set_attestations_callback(Box::new(move |attestations| {
            callback_attestations.lock().push(attestations.clone());
        }))
        .await;

    match result {
        Ok(_) => Ok(attestations),
        Err(e) => Err((attestations, e)),
    }
}

pub async fn setup_mock_recent_history_callback(
    wallet: &mut WalletWithMocks,
) -> Result<Arc<Mutex<Vec<Vec<WalletEvent>>>>, (Arc<Mutex<Vec<Vec<WalletEvent>>>>, HistoryError)> {
    // Wrap a `Vec<HistoryEvent>` in both a `Mutex` and `Arc`,
    // so we can write to it from the closure.
    let events = Arc::new(Mutex::new(Vec::<Vec<WalletEvent>>::with_capacity(2)));
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
