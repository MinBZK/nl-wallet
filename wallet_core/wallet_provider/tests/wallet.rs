use std::{
    net::{IpAddr, TcpListener},
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use assert_matches::assert_matches;
use base64::{engine::general_purpose::STANDARD, Engine};
use sea_orm::{Database, DatabaseConnection, EntityTrait, PaginatorTrait};
use serial_test::serial;
use tokio::{sync::Mutex, time::sleep};
use tracing_subscriber::FmtSubscriber;
use url::Url;

use nl_wallet_mdoc::{
    holder::{CborHttpClient, Wallet as MdocWallet},
    utils::mdocs_map::MdocsMap,
};
use pid_issuer::{
    app::{
        mock::{MockAttributesLookup, MockBsnLookup},
        AttributesLookup, BsnLookup,
    },
    server as PidServer,
    settings::Settings as PidSettings,
};
use platform_support::{hw_keystore::PlatformEcdsaKey, utils::software::SoftwareUtilities};
use wallet::{
    mock::{MockConfigurationRepository, MockDigidAuthenticator, MockPidRetriever, MockStorage},
    wallet::{
        AccountServerClient, ConfigurationRepository, DigidAuthenticator, InstructionError, PidRetriever, Storage,
        Wallet, WalletUnlockError,
    },
    wallet_deps::{PidIssuerClient, RemoteAccountServerClient},
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

fn local_pid_base_url(port: u16) -> Url {
    Url::parse(&format!("http://localhost:{}/", port)).expect("Could not create url")
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
    pid_base_url: Option<Url>,
    public_key: EcdsaDecodingKey,
    instruction_result_public_key: EcdsaDecodingKey,
    digid_authenticator: MockDigidAuthenticator,
    pid_retriever: impl PidRetriever,
) -> Wallet<
    MockConfigurationRepository,
    RemoteAccountServerClient,
    MockStorage,
    SoftwareEcdsaKey,
    MockDigidAuthenticator,
    impl PidRetriever,
> {
    // Create mock Wallet from settings
    let mut config = MockConfigurationRepository::default();
    config.0.account_server.base_url = base_url;
    config.0.account_server.certificate_public_key = public_key;
    config.0.account_server.instruction_result_public_key = instruction_result_public_key;

    if let Some(pid_base_url) = pid_base_url {
        config.0.pid_issuance.pid_issuer_url = pid_base_url;
    }

    Wallet::init_wp_and_storage::<SoftwareUtilities>(config, digid_authenticator, pid_retriever)
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

fn settings() -> (Settings, u16) {
    let mut settings = Settings::new().expect("Could not read settings");
    let port = find_listener_port();
    settings.webserver.ip = IpAddr::from_str("127.0.0.1").unwrap();
    settings.webserver.port = port;
    settings.pin_policy.timeouts_in_ms = vec![200, 400, 600];
    (settings, port)
}

fn start_wallet_provider(settings: Settings) {
    tokio::spawn(async { server::serve(settings).await.expect("Could not start server") });
    let _ = tracing::subscriber::set_global_default(FmtSubscriber::new());
}

fn pid_issuer_settings() -> (PidSettings, u16) {
    let port = find_listener_port();

    let mut settings = PidSettings::new().expect("Could not read settings");
    settings.webserver.ip = IpAddr::from_str("127.0.0.1").unwrap();
    settings.webserver.port = port;
    settings.public_url = format!("http://localhost:{}/", port).parse().unwrap();

    (settings, port)
}

fn start_pid_issuer<A, B>(settings: PidSettings, attributes_lookup: A, bsn_lookup: B)
where
    A: AttributesLookup + Send + Sync + 'static,
    B: BsnLookup + Send + Sync + 'static,
{
    tokio::spawn(async {
        PidServer::serve::<A, B>(settings, attributes_lookup, bsn_lookup)
            .await
            .expect("Could not start server")
    });

    let _ = tracing::subscriber::set_global_default(FmtSubscriber::new());
}

async fn test_wallet_registration<C, A, S, K, D, P>(mut wallet: Wallet<C, A, S, K, D, P>)
where
    C: ConfigurationRepository,
    A: AccountServerClient + Sync,
    S: Storage + Send + Sync,
    K: PlatformEcdsaKey,
    D: DigidAuthenticator,
    P: PidRetriever,
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
    let (settings, port) = settings();
    let (public_key, instruction_result_public_key) = public_key_from_settings(&settings);
    let connection = database_connection(&settings).await;
    start_wallet_provider(settings);
    let wallet = create_test_wallet(
        local_base_url(port),
        None,
        public_key,
        instruction_result_public_key,
        MockDigidAuthenticator::new(),
        MockPidRetriever {},
    )
    .await;

    let before = wallet_user_count(&connection).await;
    test_wallet_registration(wallet).await;
    let after = wallet_user_count(&connection).await;

    assert_eq!(before + 1, after);
}

#[tokio::test]
#[cfg_attr(not(feature = "live_test"), ignore)]
async fn test_wallet_registration_live() {
    let base_url = Url::parse("http://localhost:3000/api/v1/").unwrap();
    let pub_key = EcdsaDecodingKey::from_sec1(&STANDARD.decode("").unwrap());
    let instr_pub_key = EcdsaDecodingKey::from_sec1(&STANDARD.decode("").unwrap());
    let wallet = create_test_wallet(
        base_url,
        None,
        pub_key,
        instr_pub_key,
        MockDigidAuthenticator::new(),
        MockPidRetriever {},
    )
    .await;

    test_wallet_registration(wallet).await;
}

#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "db_test"), ignore)]
async fn test_unlock_ok() {
    let (settings, port) = settings();
    let (public_key, instruction_result_public_key) = public_key_from_settings(&settings);
    start_wallet_provider(settings);
    let mut wallet = create_test_wallet(
        local_base_url(port),
        None,
        public_key,
        instruction_result_public_key,
        MockDigidAuthenticator::new(),
        MockPidRetriever {},
    )
    .await;

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
async fn test_block() {
    let (mut settings, port) = settings();
    settings.pin_policy.rounds = 1;
    settings.pin_policy.attempts_per_round = 2;
    settings.pin_policy.timeouts_in_ms = vec![];

    let (public_key, instruction_result_public_key) = public_key_from_settings(&settings);
    start_wallet_provider(settings);
    let mut wallet = create_test_wallet(
        local_base_url(port),
        None,
        public_key,
        instruction_result_public_key,
        MockDigidAuthenticator::new(),
        MockPidRetriever {},
    )
    .await;

    wallet
        .register("112234".to_string())
        .await
        .expect("Could not register wallet");

    assert!(wallet.has_registration());

    wallet.lock();
    assert!(wallet.is_locked());

    let result = wallet
        .unlock("555555".to_string())
        .await
        .expect_err("invalid pin should return error");
    assert_matches!(
        result,
        WalletUnlockError::Instruction(InstructionError::IncorrectPin {
            leftover_attempts: 1,
            is_final_attempt: true
        })
    );
    assert!(wallet.is_locked());

    let result = wallet
        .unlock("555556".to_string())
        .await
        .expect_err("invalid pin should block wallet");
    assert_matches!(result, WalletUnlockError::Instruction(InstructionError::Blocked));
    assert!(wallet.is_locked());

    let result = wallet
        .unlock("112234".to_string())
        .await
        .expect_err("wallet should be blocked");
    assert_matches!(result, WalletUnlockError::Instruction(InstructionError::Blocked));
    assert!(wallet.is_locked());
}

#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "db_test"), ignore)]
async fn test_unlock_error() {
    let (settings, port) = settings();
    let (public_key, instruction_result_public_key) = public_key_from_settings(&settings);
    start_wallet_provider(settings);
    let mut wallet = create_test_wallet(
        local_base_url(port),
        None,
        public_key,
        instruction_result_public_key,
        MockDigidAuthenticator::new(),
        MockPidRetriever {},
    )
    .await;

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
        WalletUnlockError::Instruction(InstructionError::IncorrectPin {
            leftover_attempts: 3,
            is_final_attempt: false
        })
    );
    assert!(wallet.is_locked());

    let r2 = wallet
        .unlock("555556".to_string())
        .await
        .expect_err("invalid pin should return error");
    assert_matches!(
        r2,
        WalletUnlockError::Instruction(InstructionError::IncorrectPin {
            leftover_attempts: 2,
            is_final_attempt: false
        })
    );
    assert!(wallet.is_locked());

    let r3 = wallet
        .unlock("555557".to_string())
        .await
        .expect_err("invalid pin should return error");
    assert_matches!(
        r3,
        WalletUnlockError::Instruction(InstructionError::IncorrectPin {
            leftover_attempts: 1,
            is_final_attempt: false
        })
    );
    assert!(wallet.is_locked());

    // Sleeping before a timeout is expected influence timeout.
    sleep(Duration::from_millis(200)).await;

    let r4 = wallet
        .unlock("555557".to_string())
        .await
        .expect_err("invalid pin should return error");
    assert_matches!(
        r4,
        WalletUnlockError::Instruction(InstructionError::Timeout { timeout_millis: 200 })
    );
    assert!(wallet.is_locked());

    let r5 = wallet
        .unlock("555557".to_string())
        .await
        .expect_err("invalid pin should return error");
    assert_matches!(r5, WalletUnlockError::Instruction(InstructionError::Timeout { timeout_millis: t }) if t < 200);
    assert!(wallet.is_locked());

    sleep(Duration::from_millis(200)).await;

    let r6 = wallet
        .unlock("555557".to_string())
        .await
        .expect_err("invalid pin should return error");
    assert_matches!(
        r6,
        WalletUnlockError::Instruction(InstructionError::IncorrectPin {
            leftover_attempts: 3,
            is_final_attempt: false
        })
    );
    assert!(wallet.is_locked());

    let r7 = wallet
        .unlock("555557".to_string())
        .await
        .expect_err("invalid pin should return error");
    assert_matches!(
        r7,
        WalletUnlockError::Instruction(InstructionError::IncorrectPin {
            leftover_attempts: 2,
            is_final_attempt: false
        })
    );
    assert!(wallet.is_locked());

    let r8 = wallet
        .unlock("555557".to_string())
        .await
        .expect_err("invalid pin should return error");
    assert_matches!(
        r8,
        WalletUnlockError::Instruction(InstructionError::IncorrectPin {
            leftover_attempts: 1,
            is_final_attempt: false
        })
    );
    assert!(wallet.is_locked());

    let r8 = wallet
        .unlock("555557".to_string())
        .await
        .expect_err("invalid pin should return error");
    assert_matches!(
        r8,
        WalletUnlockError::Instruction(InstructionError::Timeout { timeout_millis: 400 })
    );
    assert!(wallet.is_locked());

    sleep(Duration::from_millis(400)).await;

    wallet.unlock("112234".to_string()).await.expect("should unlock wallet");
    assert!(!wallet.is_locked());
}

#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "db_test"), ignore)]
async fn test_pid_ok() {
    let (settings, port) = settings();
    let (pid_settings, pid_port) = pid_issuer_settings();
    let (public_key, instruction_result_public_key) = public_key_from_settings(&settings);
    start_wallet_provider(settings);
    start_pid_issuer(pid_settings, MockAttributesLookup, MockBsnLookup);

    let digid_client = {
        let mut digid_client = MockDigidAuthenticator::new();

        digid_client
            .expect_start_session()
            .returning(|_, _, _| Ok(Url::parse("http://localhost/").unwrap()));

        // Return a mock access token from the mock DigiD client that the `MockBsnLookup` always accepts.
        digid_client
            .expect_get_access_token()
            .returning(|_| Ok("mock_token".to_string()));

        digid_client
    };

    let client = CborHttpClient(reqwest::Client::new());
    let mdoc_wallet = Arc::new(Mutex::new(MdocWallet::new(MdocsMap::new(), client)));
    let pid_issuer_client = PidIssuerClient::new(Arc::clone(&mdoc_wallet));

    let mut wallet = create_test_wallet(
        local_base_url(port),
        Some(local_pid_base_url(pid_port)),
        public_key,
        instruction_result_public_key,
        digid_client,
        pid_issuer_client,
    )
    .await;

    wallet
        .register("112234".to_string())
        .await
        .expect("Could not register wallet");

    assert!(wallet.has_registration());

    let redirect_url = wallet.create_pid_issuance_redirect_uri().await.unwrap();
    let unsigned_modcs = wallet.continue_pid_issuance(&redirect_url).await.unwrap();
    dbg!(&unsigned_modcs);

    wallet.accept_pid_issuance("112234".to_string()).await.unwrap();

    let mdocs = mdoc_wallet.lock().await.list_mdocs();
    dbg!(&mdocs);

    let pid_mdocs = mdocs.first().unwrap().1;
    let namespace = pid_mdocs.first().unwrap();
    let attrs = namespace.first().unwrap().1;
    assert!(!attrs.is_empty());
}
