use std::num::NonZeroU8;
use std::sync::Arc;
use std::sync::LazyLock;

use chrono::DateTime;
use chrono::Utc;
use futures::future::FutureExt;
use indexmap::IndexMap;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use parking_lot::Mutex;
use rand_core::OsRng;
use serde::Serialize;
use serde::de::DeserializeOwned;
use ssri::Integrity;

use apple_app_attest::AppIdentifier;
use apple_app_attest::AttestationEnvironment;
use attestation_data::attributes::AttributeValue;
use attestation_data::attributes::Attributes;
use attestation_data::auth::issuer_auth::IssuerRegistration;
use attestation_data::constants::PID_ATTESTATION_TYPE;
use attestation_data::credential_payload::CredentialPayload;
use attestation_data::credential_payload::PreviewableCredentialPayload;
use attestation_data::x509::generate::mock::generate_issuer_mock_with_registration;
use crypto::p256_der::DerVerifyingKey;
use crypto::server_keys::KeyPair;
use crypto::server_keys::generate::Ca;
use crypto::trust_anchor::BorrowingTrustAnchor;
use http_utils::tls::pinning::TlsPinningConfig;
use jwt::SignedJwt;
use jwt::UnverifiedJwt;
use mdoc::holder::Mdoc;
use openid4vc::Format;
use openid4vc::disclosure_session::mock::MockDisclosureClient;
use openid4vc::issuance_session::NormalizedCredentialPreview;
use openid4vc::mock::MockIssuanceSession;
use openid4vc::token::CredentialPreviewContent;
use platform_support::attested_key::AttestedKey;
use platform_support::attested_key::mock::MockAppleAttestedKey;
use platform_support::attested_key::mock::MockGoogleAttestedKey;
use platform_support::attested_key::mock::MockHardwareAttestedKeyHolder;
use sd_jwt::sd_jwt::VerifiedSdJwt;
use sd_jwt_vc_metadata::JsonSchemaPropertyFormat;
use sd_jwt_vc_metadata::JsonSchemaPropertyType;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;
use sd_jwt_vc_metadata::SortedTypeMetadataDocuments;
use sd_jwt_vc_metadata::TypeMetadata;
use sd_jwt_vc_metadata::TypeMetadataDocuments;
use utils::generator::Generator;
use utils::generator::mock::MockTimeGenerator;
use wallet_account::messages::instructions::InstructionResultClaims;
use wallet_account::messages::registration::WalletCertificate;
use wallet_account::messages::registration::WalletCertificateClaims;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::account_provider::MockAccountProviderClient;
use crate::attestation::AttestationPresentation;
use crate::config::LocalConfigurationRepository;
use crate::config::UpdatingConfigurationRepository;
use crate::config::default_config_server_config;
use crate::config::default_wallet_config;
use crate::digid::MockDigidClient;
use crate::pin::key as pin_key;
use crate::storage::KeyData;
use crate::storage::MockHardwareDatabaseStorage;
use crate::storage::MockStorage;
use crate::storage::RegistrationData;
use crate::storage::Storage;
use crate::storage::StorageState;
use crate::storage::WalletEvent;
use crate::update_policy::MockUpdatePolicyRepository;
use crate::wallet::attestations::AttestationsError;
use crate::wallet::init::WalletClients;

use super::HistoryError;
use super::Wallet;
use super::WalletInitError;
use super::WalletRegistration;
use super::init::RegistrationStatus;

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

pub type MockAttestedKey = AttestedKey<MockAppleAttestedKey, MockGoogleAttestedKey>;

pub trait TestStorageRegistration {
    async fn init() -> Self;
    async fn register(&mut self, registration_data: RegistrationData);
}

/// An alias for the `Wallet<>` with mock dependencies and generic storage.
pub type TestWallet<S> = Wallet<
    UpdatingConfigurationRepository<LocalConfigurationRepository>,
    MockUpdatePolicyRepository,
    S,
    MockHardwareAttestedKeyHolder,
    MockAccountProviderClient,
    MockDigidClient<TlsPinningConfig>,
    MockIssuanceSession,
    MockDisclosureClient,
>;

/// An alias for the `Wallet<>` with all dependencies, including the storage, mocked.
pub type TestWalletMockStorage = TestWallet<MockStorage>;

/// An alias for the `Wallet<>` with an in-memory SQLite database and mock dependencies.
pub type TestWalletInMemoryStorage = TestWallet<MockHardwareDatabaseStorage>;

/// The account server key material, generated once for testing.
pub static ACCOUNT_SERVER_KEYS: LazyLock<AccountServerKeys> = LazyLock::new(|| AccountServerKeys {
    certificate_signing_key: SigningKey::random(&mut OsRng),
    instruction_result_signing_key: SigningKey::random(&mut OsRng),
});

/// The issuer key material, generated once for testing.
pub static ISSUER_KEY: LazyLock<IssuerKey> = LazyLock::new(|| {
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let issuance_key = generate_issuer_mock_with_registration(&ca, IssuerRegistration::new_mock().into()).unwrap();
    let trust_anchor = ca.as_borrowing_trust_anchor().clone();

    IssuerKey {
        issuance_key,
        trust_anchor,
    }
});

/// Generates a valid `CredentialPayload` along with its metadata `SortedTypeMetadataDocuments` and
/// `NormalizedTypeMetadata`.
pub fn create_example_credential_payload(
    time_generator: &impl Generator<DateTime<Utc>>,
) -> (CredentialPayload, SortedTypeMetadataDocuments, NormalizedTypeMetadata) {
    let credential_payload = CredentialPayload::example_with_attributes(
        PID_ATTESTATION_TYPE,
        Attributes::example([
            (["family_name"], AttributeValue::Text("De Bruijn".to_string())),
            (["given_name"], AttributeValue::Text("Willeke Liselotte".to_string())),
            (["birth_date"], AttributeValue::Text("1997-05-10".to_string())),
            (["age_over_18"], AttributeValue::Bool(true)),
        ]),
        SigningKey::random(&mut OsRng).verifying_key(),
        time_generator,
    );

    let metadata = TypeMetadata::example_with_claim_names(
        PID_ATTESTATION_TYPE,
        &[
            ("family_name", JsonSchemaPropertyType::String, None),
            ("given_name", JsonSchemaPropertyType::String, None),
            (
                "birth_date",
                JsonSchemaPropertyType::String,
                Some(JsonSchemaPropertyFormat::Date),
            ),
            ("age_over_18", JsonSchemaPropertyType::Boolean, None),
        ],
    );

    let (attestation_type, _, metadata_documents) = TypeMetadataDocuments::from_single_example(metadata);
    let (normalized_metadata, raw_metadata) = metadata_documents.into_normalized(&attestation_type).unwrap();

    (credential_payload, raw_metadata, normalized_metadata)
}

/// Generate valid `CredentialPreviewData`.
pub fn create_example_preview_data(time_generator: &impl Generator<DateTime<Utc>>) -> NormalizedCredentialPreview {
    let (credential_payload, raw_metadata, normalized_metadata) = create_example_credential_payload(time_generator);

    NormalizedCredentialPreview {
        content: CredentialPreviewContent {
            copies_per_format: IndexMap::from([(Format::MsoMdoc, NonZeroU8::new(1).unwrap())]),
            credential_payload: credential_payload.previewable_payload,
            issuer_certificate: ISSUER_KEY.issuance_key.certificate().clone(),
        },
        normalized_metadata,
        raw_metadata,
    }
}

/// Generates a valid [`VerifiedSdJwt`] that contains a full mdoc PID.
pub fn create_example_pid_sd_jwt() -> (VerifiedSdJwt, NormalizedTypeMetadata) {
    let credential_payload = CredentialPayload::nl_pid_example(&MockTimeGenerator::default());
    let metadata = NormalizedTypeMetadata::nl_pid_example();

    let verified_sd_jwt =
        verified_sd_jwt_from_credential_payload(credential_payload, &metadata, &ISSUER_KEY.issuance_key);

    (verified_sd_jwt, metadata)
}

pub fn verified_sd_jwt_from_credential_payload(
    credential_payload: CredentialPayload,
    metadata: &NormalizedTypeMetadata,
    issuer_keypair: &KeyPair,
) -> VerifiedSdJwt {
    let sd_jwt = credential_payload
        .into_sd_jwt(metadata, issuer_keypair)
        .now_or_never()
        .unwrap()
        .unwrap();

    sd_jwt.into_verified()
}

/// Generates a valid [`Mdoc`] that contains a full mdoc PID.
pub fn create_example_pid_mdoc() -> Mdoc {
    let preview_payload = PreviewableCredentialPayload::nl_pid_example(&MockTimeGenerator::default());

    mdoc_from_credential_payload(preview_payload, &ISSUER_KEY.issuance_key)
}

/// Generates a valid [`Mdoc`], based on an [`PreviewableCredentialPayload`] and issuer key.
pub fn mdoc_from_credential_payload(preview_payload: PreviewableCredentialPayload, issuer_keypair: &KeyPair) -> Mdoc {
    let private_key_id = crypto::utils::random_string(16);
    let holder_privkey = SigningKey::random(&mut OsRng);

    preview_payload
        .into_signed_mdoc_unverified(
            Integrity::from(""),
            private_key_id,
            holder_privkey.verifying_key(),
            issuer_keypair,
        )
        .now_or_never()
        .unwrap()
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

fn create_wallet_configuration() -> WalletConfiguration {
    // Override public key material in the `Configuration`.
    let keys = LazyLock::force(&ACCOUNT_SERVER_KEYS);

    let mut config = default_wallet_config();

    config.account_server.certificate_public_key = (*keys.certificate_signing_key.verifying_key()).into();
    config.account_server.instruction_result_public_key = (*keys.instruction_result_signing_key.verifying_key()).into();

    config.issuer_trust_anchors = vec![ISSUER_KEY.trust_anchor.clone()];

    config
}

/// Generates a valid certificate for the `Wallet`.
pub fn valid_certificate(wallet_id: Option<String>, hw_pubkey: VerifyingKey) -> WalletCertificate {
    SignedJwt::sign_with_sub(
        valid_certificate_claims(wallet_id, hw_pubkey),
        &ACCOUNT_SERVER_KEYS.certificate_signing_key,
    )
    .now_or_never()
    .unwrap()
    .unwrap()
    .into()
}

/// Generates valid certificate claims for the `Wallet`.
pub fn valid_certificate_claims(wallet_id: Option<String>, hw_pubkey: VerifyingKey) -> WalletCertificateClaims {
    let wallet_id = wallet_id.unwrap_or_else(|| crypto::utils::random_string(32));

    WalletCertificateClaims {
        wallet_id,
        hw_pubkey: DerVerifyingKey::from(hw_pubkey),
        pin_pubkey_hash: crypto::utils::random_bytes(32),
        version: 0,
        iss: "wallet_unit_test".to_string(),
        iat: Utc::now(),
    }
}

impl TestStorageRegistration for MockHardwareDatabaseStorage {
    async fn init() -> Self {
        MockHardwareDatabaseStorage::open_in_memory().await
    }

    async fn register(&mut self, registration_data: RegistrationData) {
        self.insert_data::<RegistrationData>(&registration_data).await.unwrap();
    }
}

impl TestStorageRegistration for MockStorage {
    async fn init() -> Self {
        let mut storage = MockStorage::default();
        storage.expect_state().returning(|| Ok(StorageState::Opened));
        storage.expect_fetch_data::<RegistrationData>().returning(|| Ok(None));
        storage.expect_fetch_data::<KeyData>().returning(|| Ok(None));
        storage
    }

    async fn register(&mut self, registration_data: RegistrationData) {
        self.expect_state().returning(|| Ok(StorageState::Opened));
        self.expect_fetch_data::<RegistrationData>()
            .returning(move || Ok(Some(registration_data.clone())));
    }
}

impl<S> TestWallet<S>
where
    S: TestStorageRegistration + Storage,
{
    pub fn mut_storage(&mut self) -> &mut S {
        Arc::get_mut(&mut self.storage).unwrap().get_mut()
    }

    /// Creates an unregistered `Wallet` with mock dependencies.
    pub async fn new_unregistered(vendor: WalletDeviceVendor) -> Self {
        let config_server_config = default_config_server_config();
        let config_repository = UpdatingConfigurationRepository::new(
            LocalConfigurationRepository::new(create_wallet_configuration()),
            config_server_config,
        )
        .await;

        Wallet::new(
            config_repository,
            MockUpdatePolicyRepository::default(),
            S::init().await,
            generate_key_holder(vendor),
            WalletClients::default(),
            RegistrationStatus::Unregistered,
        )
    }

    pub async fn new_init_registration(vendor: WalletDeviceVendor) -> Result<Self, WalletInitError> {
        let config_server_config = default_config_server_config();
        let config_repository =
            UpdatingConfigurationRepository::new(LocalConfigurationRepository::default(), config_server_config).await;

        Wallet::init_registration(
            config_repository,
            MockUpdatePolicyRepository::default(),
            S::init().await,
            generate_key_holder(vendor),
            WalletClients::default(),
        )
        .await
    }

    pub async fn new_registered_and_unlocked(vendor: WalletDeviceVendor) -> Self {
        let mut wallet = Self::new_unregistered(vendor).await;

        // Generate registration data.
        let (registration_data, attested_key) = wallet.registration_data();

        // Store the registration in `Storage`, populate the field on `Wallet` and set the wallet to unlocked.
        S::register(wallet.mut_storage(), registration_data.clone()).await;

        wallet.registration = WalletRegistration::Registered {
            attested_key: Arc::new(attested_key),
            data: registration_data,
        };

        wallet.lock.unlock();

        wallet
    }

    fn registration_data(&self) -> (RegistrationData, MockAttestedKey) {
        let (attested_key, attested_key_identifier) = self.key_holder.random_key();
        let verifying_key = match &attested_key {
            AttestedKey::Apple(key) => *key.verifying_key(),
            AttestedKey::Google(key) => *key.verifying_key(),
        };
        let wallet_certificate = valid_certificate(None, verifying_key);
        let wallet_id = wallet_certificate.dangerous_parse_unverified().unwrap().1.wallet_id;

        // Generate registration data.
        let registration_data = RegistrationData {
            attested_key_identifier,
            pin_salt: pin_key::new_pin_salt(),
            wallet_id,
            wallet_certificate,
        };

        (registration_data, attested_key)
    }
}

impl TestWalletMockStorage {
    /// Allows for overriding the `MockStorage` and `MockHardwareAttestedKeyHolder` instances.
    pub async fn new_init_registration_with_mock_storage(
        key_holder: MockHardwareAttestedKeyHolder,
        storage: MockStorage,
    ) -> Result<Self, WalletInitError> {
        let config_server_config = default_config_server_config();
        let config_repository =
            UpdatingConfigurationRepository::new(LocalConfigurationRepository::default(), config_server_config).await;

        Wallet::init_registration(
            config_repository,
            MockUpdatePolicyRepository::default(),
            storage,
            key_holder,
            WalletClients::default(),
        )
        .await
    }
}

pub async fn setup_mock_attestations_callback<S>(
    wallet: &mut TestWallet<S>,
) -> Result<
    Arc<Mutex<Vec<Vec<AttestationPresentation>>>>,
    (Arc<Mutex<Vec<Vec<AttestationPresentation>>>>, AttestationsError),
>
where
    S: Storage,
{
    // Wrap a `Vec<Attestation>` in both a `Mutex` and `Arc`,
    // so we can write to it from the closure.
    let attestations = Arc::new(Mutex::new(Vec::<Vec<AttestationPresentation>>::with_capacity(1)));
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

pub async fn setup_mock_recent_history_callback<S>(
    wallet: &mut TestWallet<S>,
) -> Result<Arc<Mutex<Vec<Vec<WalletEvent>>>>, (Arc<Mutex<Vec<Vec<WalletEvent>>>>, HistoryError)>
where
    S: Storage,
{
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

pub fn create_wp_result<T>(result: T) -> UnverifiedJwt<InstructionResultClaims<T>>
where
    T: Serialize + DeserializeOwned,
{
    let result_claims = InstructionResultClaims {
        result,
        iss: "wallet_unit_test".to_string(),
        iat: Utc::now(),
    };
    SignedJwt::sign_with_sub(result_claims, &ACCOUNT_SERVER_KEYS.instruction_result_signing_key)
        .now_or_never()
        .unwrap()
        .expect("could not sign instruction result")
        .into()
}
