use once_cell::sync::Lazy;
use p256::{ecdsa::SigningKey, elliptic_curve::rand_core::OsRng};

use nl_wallet_mdoc::{
    basic_sa_ext::UnsignedMdoc,
    holder::{Mdoc, TrustAnchor},
    issuer::PrivateKey,
    mock as mdoc_mock,
    utils::x509::OwnedTrustAnchor,
    IssuerSigned,
};
use wallet_common::{
    account::{
        jwt::Jwt,
        messages::auth::{WalletCertificate, WalletCertificateClaims},
    },
    generator::TimeGenerator,
    keys::{software::SoftwareEcdsaKey, ConstructibleWithIdentifier, EcdsaKey},
    utils,
};

use crate::{
    account_provider::MockAccountProviderClient,
    config::LocalConfigurationRepository,
    digid::MockDigidSession,
    document,
    pid_issuer::MockPidIssuerClient,
    pin::key as pin_key,
    storage::{KeyedData, MockStorage, RegistrationData, StorageState},
    Configuration,
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
    pub trust_anchor: OwnedTrustAnchor,
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
    instruction_result_signing_key: SigningKey::random(&mut OsRng),
});

/// The issuer key material, generated once for testing.
pub static ISSUER_KEY: Lazy<IssuerKey> = Lazy::new(|| {
    let (issuance_key, ca) = mdoc_mock::generate_issuance_key_and_ca().unwrap();
    let trust_anchor: TrustAnchor<'_> = (&ca).try_into().unwrap();

    IssuerKey {
        issuance_key,
        trust_anchor: (&trust_anchor).into(),
    }
});

/// Generates a valid `Mdoc` that contains a full PID.
pub async fn create_full_pid_mdoc() -> Mdoc {
    let private_key_id = utils::random_string(16);
    let unsigned_mdoc = document::create_full_unsigned_pid_mdoc();

    mdoc_from_unsigned(unsigned_mdoc, private_key_id).await
}

/// Generates a valid `Mdoc`, based on an `UnsignedMdoc` and key identifier.
pub async fn mdoc_from_unsigned(unsigned_mdoc: UnsignedMdoc, private_key_id: String) -> Mdoc {
    let mdoc_public_key = (&SoftwareEcdsaKey::new(&private_key_id).verifying_key().await.unwrap())
        .try_into()
        .unwrap();
    let (issuer_signed, _) = IssuerSigned::sign(unsigned_mdoc, mdoc_public_key, &ISSUER_KEY.issuance_key)
        .await
        .unwrap();

    Mdoc::new::<SoftwareEcdsaKey>(
        private_key_id,
        issuer_signed,
        &TimeGenerator,
        &[(&ISSUER_KEY.trust_anchor).into()],
    )
    .unwrap()
}

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

        // Override public key material in the `Configuration`.
        let config = {
            let mut config = Configuration::default();
            config.account_server.certificate_public_key = (*keys.certificate_signing_key.verifying_key()).into();
            config.account_server.instruction_result_public_key =
                (*keys.instruction_result_signing_key.verifying_key()).into();
            config.mdoc_trust_anchors = vec![ISSUER_KEY.trust_anchor.clone()];

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
