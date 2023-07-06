use std::{
    net::{IpAddr, TcpListener},
    str::FromStr,
};

use base64::{engine::general_purpose::STANDARD, Engine};
use p256::{ecdsa::SigningKey, pkcs8::DecodePrivateKey};
use sea_orm::{Database, DatabaseConnection, EntityTrait, PaginatorTrait};
use url::Url;

use platform_support::hw_keystore::{software::SoftwareEcdsaKey, PlatformEcdsaKey};
use wallet::{
    mock::{MockConfigurationRepository, MockStorage, RemoteAccountServerClient},
    wallet::{AccountServerClient, ConfigurationRepository, Storage, Wallet},
};
use wallet_common::account::jwt::EcdsaDecodingKey;
use wallet_provider::{server, settings::Settings};
use wallet_provider_persistence::{entity::wallet_user, postgres};

fn public_key_from_settings(settings: &Settings) -> EcdsaDecodingKey {
    EcdsaDecodingKey::from_sec1(
        SigningKey::from_pkcs8_der(&settings.signing_private_key.0)
            .expect("Could not decode private key")
            .verifying_key()
            .to_encoded_point(false)
            .as_bytes(),
    )
}

fn local_base_url(port: u16) -> Url {
    Url::parse(&format!("http://127.0.0.1:{}", port)).expect("Could not create url")
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
) -> Wallet<MockConfigurationRepository, RemoteAccountServerClient, MockStorage, SoftwareEcdsaKey> {
    tracing_subscriber::fmt::init();

    // Create mock Wallet from settings
    let mut config = MockConfigurationRepository::default();
    config.0.account_server.base_url = base_url;
    config.0.account_server.public_key = public_key;

    Wallet::new(config).await.expect("Could not create test wallet")
}

async fn wallet_user_count(connection: &DatabaseConnection) -> u64 {
    wallet_user::Entity::find()
        .count(connection)
        .await
        .expect("Could not fetch user count from database")
}

fn find_listener_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("Could not find TCP port")
        .local_addr()
        .expect("Could not get local address from TCP listener")
        .port()
}

fn start_wallet_provider(mut settings: Settings) -> u16 {
    let port = find_listener_port();
    settings.webserver.ip = IpAddr::from_str("127.0.0.1").expect("Could not parse IP address");
    settings.webserver.port = port;
    tokio::spawn(async { server::serve(settings).await.expect("Could not start server") });
    port
}

async fn test_wallet_registration<C, A, S, K>(mut wallet: Wallet<C, A, S, K>)
where
    C: ConfigurationRepository,
    A: AccountServerClient,
    S: Storage + Default,
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
#[cfg_attr(not(feature = "db_test"), ignore)]
async fn test_wallet_registration_in_process() {
    let settings = Settings::new().expect("Could not read settings");
    let public_key = public_key_from_settings(&settings);
    let connection = database_connection(&settings).await;
    let port = start_wallet_provider(settings);
    let wallet = create_test_wallet(local_base_url(port), public_key).await;

    let before = wallet_user_count(&connection).await;
    test_wallet_registration(wallet).await;
    let after = wallet_user_count(&connection).await;

    assert_eq!(before + 1, after);
}

#[tokio::test]
#[cfg_attr(not(feature = "live_test"), ignore)]
async fn test_wallet_registration_live() {
    let base_url = Url::parse("http://localhost:3000").unwrap();
    let pub_key = EcdsaDecodingKey::from_sec1(&STANDARD.decode("").unwrap());
    let wallet = create_test_wallet(base_url, pub_key).await;

    test_wallet_registration(wallet).await;
}
