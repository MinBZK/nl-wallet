use std::env;
use std::sync::Arc;

use async_trait::async_trait;
use axum_test_helper::TestClient;
use p256::{ecdsa::SigningKey, pkcs8::DecodePrivateKey};
use platform_support::hw_keystore::PlatformEcdsaKey;
use sea_orm::{Database, DatabaseConnection, EntityTrait, PaginatorTrait};
use tokio::sync::OnceCell;
use url::Url;

use platform_support::hw_keystore::software::SoftwareEcdsaKey;
use wallet::mock::{ConfigurationRepository, RemoteAccountServerClient};
use wallet::wallet::Storage;
use wallet::{
    mock::{MockConfigurationRepository, MockStorage},
    wallet::{AccountServerClient, AccountServerClientError, Wallet},
    AccountServerConfiguration, Configuration, LockTimeoutConfiguration,
};
use wallet_common::account::{
    auth::{Certificate, Challenge, Registration, WalletCertificate},
    jwt::EcdsaDecodingKey,
    signed::SignedDouble,
};
use wallet_provider::{app, app_dependencies::AppDependencies, settings::Settings};
use wallet_provider_persistence::{entity::wallet_user, postgres};

/// A global [`TestClient`] that is only initialized once.
static TEST_CLIENT: OnceCell<TestClient> = OnceCell::const_new();

/// This struct acts as a client for [`Wallet`] by implementing [`AccountServerClient`]
/// and using [`TestClient`]. It can access the routes of the Wallet Provider without
/// actually needing a HTTP server.
struct WalletTestClient {
    client: &'static TestClient,
}

#[async_trait]
impl AccountServerClient for WalletTestClient {
    fn new(_base_url: &Url) -> Self
    where
        Self: Sized,
    {
        WalletTestClient {
            client: TEST_CLIENT.get().expect("TEST_CLIENT not initialized"),
        }
    }

    async fn registration_challenge(&self) -> Result<Vec<u8>, AccountServerClientError> {
        let challenge = self
            .client
            .post("/api/v1/enroll")
            .send()
            .await
            .json::<Challenge>()
            .await
            .challenge
            .0;

        Ok(challenge)
    }

    async fn register(
        &self,
        registration_message: SignedDouble<Registration>,
    ) -> Result<WalletCertificate, AccountServerClientError> {
        let cert = self
            .client
            .post("/api/v1/createwallet")
            .json(&registration_message)
            .send()
            .await
            .json::<Certificate>()
            .await
            .certificate;

        Ok(cert)
    }
}

/// Create an instance of [`Wallet`] with appropriate mocks, including [`WalletTestClient`].
/// Also create a new connection to the database.
async fn create_test_wallet_and_db_connection() -> (
    Wallet<MockConfigurationRepository, WalletTestClient, MockStorage, SoftwareEcdsaKey>,
    DatabaseConnection,
    EcdsaDecodingKey,
) {
    // Make sure TEST_CLIENT is initialized
    _ = TEST_CLIENT
        .get_or_init(|| async {
            let settings = Settings::new().expect("Could not read settings");
            let deps = Arc::new(
                AppDependencies::new_from_settings(settings)
                    .await
                    .expect("Could not create app dependencies"),
            );

            TestClient::new(app::router(deps))
        })
        .await;

    // Read settings again
    let settings = Settings::new().expect("Could not read settings");

    // Create mock Wallet from settings
    let mut config = MockConfigurationRepository::default();
    let pubkey = EcdsaDecodingKey::from_sec1(
        SigningKey::from_pkcs8_der(&settings.signing_private_key.0)
            .expect("Could not decode private key")
            .verifying_key()
            .to_encoded_point(false)
            .as_bytes(),
    );
    config.0.account_server.public_key = pubkey.clone();
    let wallet = Wallet::new(config).await.expect("Could not create test wallet");

    // Create database connection from settings
    let connection = Database::connect(postgres::connection_string(
        &settings.database.host,
        &settings.database.name,
        settings.database.username.as_deref(),
        settings.database.password.as_deref(),
    ))
    .await
    .expect("Could not open database connection");

    (wallet, connection, pubkey)
}

async fn wallet_user_count(connection: &DatabaseConnection) -> u64 {
    wallet_user::Entity::find()
        .count(connection)
        .await
        .expect("Could not fetch user count from database")
}

async fn test_wallet_registration<C, A, S, K>(mut wallet: Wallet<C, A, S, K>, conn: &DatabaseConnection)
where
    C: ConfigurationRepository,
    A: AccountServerClient,
    S: Storage + Default,
    K: PlatformEcdsaKey + Clone + Send + 'static,
{
    // No registration should be loaded initially.
    assert!(!wallet.has_registration());

    let before = wallet_user_count(conn).await;

    // Register with a valid PIN.
    wallet
        .register("112233".to_string())
        .await
        .expect("Could not register wallet");

    // The registration should now be loaded.
    assert!(wallet.has_registration());

    let after = wallet_user_count(conn).await;

    assert_eq!(before + 1, after);

    // Registering again should result in an error.
    assert!(wallet.register("112233".to_owned()).await.is_err());
}

#[tokio::test]
#[cfg_attr(not(feature = "db_test"), ignore)]
async fn test_wallet_registration_direct() {
    let (wallet, connection, _) = create_test_wallet_and_db_connection().await;
    test_wallet_registration(wallet, &connection).await;
}

#[tokio::test]
#[cfg_attr(not(feature = "http_test"), ignore)]
async fn test_wallet_registration_via_http() {
    let (_, connection, public_key) = create_test_wallet_and_db_connection().await;
    let base_url = &env::var("WALLET_PROVIDER_BASE_URL").unwrap_or("http://localhost:3000".to_string());

    let config = MockConfigurationRepository(Configuration {
        lock_timeouts: LockTimeoutConfiguration {
            inactive_timeout: 60,
            background_timeout: 2 * 60,
        },
        account_server: AccountServerConfiguration {
            base_url: Url::parse(base_url).unwrap(),
            public_key,
        },
    });

    let wallet: Wallet<MockConfigurationRepository, RemoteAccountServerClient, MockStorage, SoftwareEcdsaKey> =
        Wallet::new_without_registration(config);

    test_wallet_registration(wallet, &connection).await;
}
