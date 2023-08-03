use std::{
    net::{IpAddr, TcpListener},
    str::FromStr,
    time::Duration,
};

use assert_matches::assert_matches;
use base64::{engine::general_purpose::STANDARD, Engine};
use sea_orm::{Database, DatabaseConnection, EntityTrait, PaginatorTrait};
use serial_test::serial;
use tokio::time::sleep;
use tracing_subscriber::FmtSubscriber;
use url::Url;

use platform_support::{hw_keystore::PlatformEcdsaKey, utils::software::SoftwareUtilities};
use wallet::{
    mock::{MockConfigurationRepository, MockStorage, RemoteAccountServerClient},
    wallet::{AccountServerClient, ConfigurationRepository, Storage, Wallet, WalletUnlockError},
};
use wallet_common::{account::jwt::EcdsaDecodingKey, keys::software::SoftwareEcdsaKey};
use wallet_provider::{server, settings::Settings};
use wallet_provider_persistence::{entity::wallet_user, postgres};

fn public_key_from_settings(settings: &Settings) -> (EcdsaDecodingKey, EcdsaDecodingKey) {
    (
        (*settings.certificate_private_key.0.verifying_key()).into(),
        (*settings.instruction_result_private_key.0.verifying_key()).into(),
    )
}

fn local_base_url(port: u16) -> Url {
    Url::parse(&format!("http://localhost:{}/api/v1/", port)).expect("Could not create url")
}

async fn database_connection(settings: &Settings) -> DatabaseConnection {
    Database::connect(postgres::connection_string(
        &settings.database.host,
        &settings.database.name,
        settings.database.username.as_deref(),
        settings.database.password.as_deref(),
    ))
    .await
    .expect("Could not open database connection")
}

/// Create an instance of [`Wallet`].
async fn create_test_wallet(
    base_url: Url,
    public_key: EcdsaDecodingKey,
    instruction_result_public_key: EcdsaDecodingKey,
) -> Wallet<MockConfigurationRepository, RemoteAccountServerClient, MockStorage, SoftwareEcdsaKey> {
    // Create mock Wallet from settings
    let mut config = MockConfigurationRepository::default();
    config.0.account_server.base_url = base_url;
    config.0.account_server.certificate_public_key = public_key;
    config.0.account_server.instruction_result_public_key = instruction_result_public_key;

    Wallet::init::<SoftwareUtilities>(config)
        .await
        .expect("Could not create test wallet")
}

async fn wallet_user_count(connection: &DatabaseConnection) -> u64 {
    wallet_user::Entity::find()
        .count(connection)
        .await
        .expect("Could not fetch user count from database")
}

fn find_listener_port() -> u16 {
    TcpListener::bind("localhost:0")
        .expect("Could not find TCP port")
        .local_addr()
        .expect("Could not get local address from TCP listener")
        .port()
}

fn start_wallet_provider(mut settings: Settings) -> u16 {
    let port = find_listener_port();
    settings.webserver.ip = IpAddr::from_str("127.0.0.1").expect("Could not parse IP address");
    settings.webserver.port = port;
    settings.pin_policy.timeouts_in_ms = vec![200, 400, 600];

    tokio::spawn(async { server::serve(settings).await.expect("Could not start server") });

    let _ = tracing::subscriber::set_global_default(FmtSubscriber::new());

    port
}

async fn test_wallet_registration<C, A, S, K>(mut wallet: Wallet<C, A, S, K>)
where
    C: ConfigurationRepository,
    A: AccountServerClient,
    S: Storage,
    K: PlatformEcdsaKey + Clone + Send + 'static,
{
    // No registration should be loaded initially.
    assert!(!wallet.has_registration());

    // Register with a valid PIN.
    wallet
        .register("112233".to_string())
        .await
        .expect("Could not register wallet");

    // The registration should now be loaded.
    assert!(wallet.has_registration());

    // Registering again should result in an error.
    assert!(wallet.register("112233".to_owned()).await.is_err());
}

#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "db_test"), ignore)]
async fn test_wallet_registration_in_process() {
    let settings = Settings::new().expect("Could not read settings");
    let (public_key, instruction_result_public_key) = public_key_from_settings(&settings);
    let connection = database_connection(&settings).await;
    let port = start_wallet_provider(settings);
    let wallet = create_test_wallet(local_base_url(port), public_key, instruction_result_public_key).await;

    let before = wallet_user_count(&connection).await;
    test_wallet_registration(wallet).await;
    let after = wallet_user_count(&connection).await;

    assert_eq!(before + 1, after);
}

#[tokio::test]
#[cfg_attr(not(feature = "live_test"), ignore)]
async fn test_wallet_registration_live() {
    let base_url = Url::parse("http://localhost:3000/api/v1").unwrap();
    let pub_key = EcdsaDecodingKey::from_sec1(&STANDARD.decode("").unwrap());
    let instr_pub_key = EcdsaDecodingKey::from_sec1(&STANDARD.decode("").unwrap());
    let wallet = create_test_wallet(base_url, pub_key, instr_pub_key).await;

    test_wallet_registration(wallet).await;
}

#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "db_test"), ignore)]
async fn test_unlock_ok() {
    let settings = Settings::new().expect("Could not read settings");
    let (public_key, instruction_result_public_key) = public_key_from_settings(&settings);
    let port = start_wallet_provider(settings);
    let mut wallet = create_test_wallet(local_base_url(port), public_key, instruction_result_public_key).await;

    wallet
        .register("112234".to_string())
        .await
        .expect("Could not register wallet");

    assert!(wallet.has_registration());

    wallet.lock();
    assert!(wallet.is_locked());

    wallet.unlock("112234".to_string()).await.expect("Should unlock wallet");
    assert!(!wallet.is_locked());

    // Test multiple instructions
    wallet.unlock("112234".to_string()).await.expect("Should unlock wallet");
    assert!(!wallet.is_locked());
}

#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "db_test"), ignore)]
async fn test_unlock_error() {
    let settings = Settings::new().expect("Could not read settings");
    let (public_key, instruction_result_public_key) = public_key_from_settings(&settings);
    let port = start_wallet_provider(settings);
    let mut wallet = create_test_wallet(local_base_url(port), public_key, instruction_result_public_key).await;

    wallet
        .register("112234".to_string())
        .await
        .expect("Could not register wallet");

    assert!(wallet.has_registration());

    wallet.lock();
    assert!(wallet.is_locked());

    let r1 = wallet
        .unlock("555555".to_string())
        .await
        .expect_err("invalid pin should return error");
    assert_matches!(
        r1,
        WalletUnlockError::IncorrectPin {
            leftover_attempts: 3,
            is_final_attempt: false
        }
    );
    assert!(wallet.is_locked());

    let r2 = wallet
        .unlock("555556".to_string())
        .await
        .expect_err("invalid pin should return error");
    assert_matches!(
        r2,
        WalletUnlockError::IncorrectPin {
            leftover_attempts: 2,
            is_final_attempt: false
        }
    );
    assert!(wallet.is_locked());

    let r3 = wallet
        .unlock("555557".to_string())
        .await
        .expect_err("invalid pin should return error");
    assert_matches!(
        r3,
        WalletUnlockError::IncorrectPin {
            leftover_attempts: 1,
            is_final_attempt: false
        }
    );
    assert!(wallet.is_locked());

    // Sleeping before a timeout is expected influence timeout.
    sleep(Duration::from_millis(200)).await;

    let r4 = wallet
        .unlock("555557".to_string())
        .await
        .expect_err("invalid pin should return error");
    assert_matches!(r4, WalletUnlockError::Timeout { timeout_millis: 200 });
    assert!(wallet.is_locked());

    let r5 = wallet
        .unlock("555557".to_string())
        .await
        .expect_err("invalid pin should return error");
    assert_matches!(r5, WalletUnlockError::Timeout { timeout_millis: 200 });
    assert!(wallet.is_locked());

    sleep(Duration::from_millis(200)).await;

    let r6 = wallet
        .unlock("555557".to_string())
        .await
        .expect_err("invalid pin should return error");
    assert_matches!(
        r6,
        WalletUnlockError::IncorrectPin {
            leftover_attempts: 3,
            is_final_attempt: false
        }
    );
    assert!(wallet.is_locked());

    let r7 = wallet
        .unlock("555557".to_string())
        .await
        .expect_err("invalid pin should return error");
    assert_matches!(
        r7,
        WalletUnlockError::IncorrectPin {
            leftover_attempts: 2,
            is_final_attempt: false
        }
    );
    assert!(wallet.is_locked());

    let r8 = wallet
        .unlock("555557".to_string())
        .await
        .expect_err("invalid pin should return error");
    assert_matches!(
        r8,
        WalletUnlockError::IncorrectPin {
            leftover_attempts: 1,
            is_final_attempt: false
        }
    );
    assert!(wallet.is_locked());

    let r8 = wallet
        .unlock("555557".to_string())
        .await
        .expect_err("invalid pin should return error");
    assert_matches!(r8, WalletUnlockError::Timeout { timeout_millis: 400 });
    assert!(wallet.is_locked());

    sleep(Duration::from_millis(400)).await;

    wallet.unlock("112234".to_string()).await.expect("should unlock wallet");
    assert!(!wallet.is_locked());
}
