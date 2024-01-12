use std::{
    net::{IpAddr, TcpListener},
    process,
    str::FromStr,
    time::Duration,
};

use async_trait::async_trait;
use ctor::ctor;
use openid4vc::{
    issuer::{AttributeService, Created},
    token::TokenRequest,
};
use sea_orm::{Database, DatabaseConnection, EntityTrait, PaginatorTrait};
use tokio::time;
use url::Url;

use nl_wallet_mdoc::{
    basic_sa_ext::UnsignedMdoc,
    server_state::{MemorySessionStore, SessionState},
};
use platform_support::utils::{software::SoftwareUtilities, PlatformUtilities};
use wallet::{
    mock::{default_configuration, MockDigidSession, MockStorage},
    wallet_deps::{
        HttpAccountProviderClient, HttpConfigurationRepository, HttpOpenidPidIssuerClient,
        UpdateableConfigurationRepository,
    },
    Wallet,
};
use wallet_common::{config::wallet_config::WalletConfiguration, keys::software::SoftwareEcdsaKey};
use wallet_provider::{server as wp_server, settings::Settings as WpSettings};
use wallet_provider_persistence::entity::wallet_user;
use wallet_server::{
    pid::{
        attributes::AttributesLookup as _, mock::MockAttributesLookup as WSMockAttributesLookup,
        mock::MockBsnLookup as WSMockBsnLookup,
    },
    server as ws_server,
    settings::{Server, Settings as WsSettings},
    store::{SessionStores, WalletServerSessionStore},
};

#[ctor]
fn init_logging() {
    let _ = tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_test_writer()
            .finish(),
    );
}

pub fn local_wp_base_url(port: &u16) -> Url {
    Url::parse(&format!("http://localhost:{}/api/v1/", port)).expect("Could not create url")
}

pub fn local_config_base_url(port: &u16) -> Url {
    Url::parse(&format!("http://localhost:{}/config/v1/", port)).expect("Could not create url")
}

pub fn local_pid_base_url(port: &u16) -> Url {
    Url::parse(&format!("http://localhost:{}/issuance/", port)).expect("Could not create url")
}

pub async fn database_connection(settings: &WpSettings) -> DatabaseConnection {
    Database::connect(settings.database.connection_string())
        .await
        .expect("Could not open database connection")
}

pub type WalletWithMocks = Wallet<
    HttpConfigurationRepository,
    MockStorage,
    SoftwareEcdsaKey,
    HttpAccountProviderClient,
    MockDigidSession,
    HttpOpenidPidIssuerClient,
>;

pub async fn setup_wallet_and_default_env() -> WalletWithMocks {
    setup_wallet_and_env(wallet_provider_settings(), wallet_server_settings()).await
}

/// Create an instance of [`Wallet`].
pub async fn setup_wallet_and_env(wp_settings: WpSettings, ws_settings: WsSettings) -> WalletWithMocks {
    let mut wallet_config = default_configuration();
    wallet_config.pid_issuance.pid_issuer_url = local_pid_base_url(&ws_settings.wallet_server.port);
    wallet_config.account_server.base_url = local_wp_base_url(&wp_settings.webserver.port);

    let config_base_url = local_config_base_url(&wp_settings.webserver.port);

    start_wallet_provider(wp_settings, wallet_config.clone()).await;
    start_wallet_server(
        ws_settings,
        SessionStores {
            disclosure: WalletServerSessionStore::Memory(MemorySessionStore::new()),
            issuance: WalletServerSessionStore::Memory(MemorySessionStore::new()),
        },
        MockAttributeService,
    )
    .await;

    let pid_issuer_client = HttpOpenidPidIssuerClient::default();

    let config_repository = HttpConfigurationRepository::new(
        config_base_url,
        SoftwareUtilities::storage_path().await.unwrap(),
        wallet_config,
    )
    .await
    .unwrap();
    config_repository.fetch().await.unwrap();

    Wallet::init_registration(
        config_repository,
        MockStorage::default(),
        HttpAccountProviderClient::default(),
        pid_issuer_client,
    )
    .await
    .expect("Could not create test wallet")
}

pub async fn wallet_user_count(connection: &DatabaseConnection) -> u64 {
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

pub fn wallet_provider_settings() -> WpSettings {
    let port = find_listener_port();

    let mut settings = WpSettings::new().expect("Could not read settings");
    settings.webserver.ip = IpAddr::from_str("127.0.0.1").unwrap();
    settings.webserver.port = port;
    settings.pin_policy.timeouts_in_ms = vec![200, 400, 600];
    settings
}

pub async fn start_wallet_provider(settings: WpSettings, wallet_config: WalletConfiguration) {
    let base_url = local_wp_base_url(&settings.webserver.port);
    tokio::spawn(async {
        if let Err(error) = wp_server::serve(settings, wallet_config).await {
            println!("Could not start wallet_provider: {:?}", error);

            process::exit(1);
        }
    });

    wait_for_server(base_url).await;
}

pub fn wallet_server_settings() -> WsSettings {
    let mut settings = WsSettings::new().expect("Could not read settings");
    let ws_port = find_listener_port();

    settings.wallet_server.ip = IpAddr::from_str("127.0.0.1").unwrap();
    settings.wallet_server.port = ws_port;

    let requester_port = find_listener_port();
    settings.requester_server = Server {
        ip: IpAddr::from_str("127.0.0.1").unwrap(),
        port: requester_port,
    };

    settings.public_url = Url::parse(&format!("http://localhost:{}/", ws_port)).unwrap();
    settings.internal_url = Url::parse(&format!("http://localhost:{}/", requester_port)).unwrap();
    settings
}

pub async fn start_wallet_server<A>(settings: WsSettings, sessions: SessionStores, attr_service: A)
where
    A: AttributeService,
{
    let public_url = settings.public_url.clone();
    tokio::spawn(async move {
        if let Err(error) = ws_server::serve_full(&settings, sessions, attr_service).await {
            println!("Could not start wallet_server: {:?}", error);

            process::exit(1);
        }
    });

    wait_for_server(public_url.join("disclosure/").unwrap()).await;
}

async fn wait_for_server(base_url: Url) {
    let client = reqwest::Client::new();

    time::timeout(Duration::from_secs(3), async {
        let mut interval = time::interval(Duration::from_millis(10));
        loop {
            match client.get(base_url.join("health").unwrap()).send().await {
                Ok(_) => break,
                _ => {
                    println!("Waiting for wallet_server...");
                    interval.tick().await;
                }
            }
        }
    })
    .await
    .unwrap();
}

pub async fn do_wallet_registration(mut wallet: WalletWithMocks, pin: String) -> WalletWithMocks {
    // No registration should be loaded initially.
    assert!(!wallet.has_registration());

    // Register with a valid PIN.
    wallet.register(pin.clone()).await.expect("Could not register wallet");

    // The registration should now be loaded.
    assert!(wallet.has_registration());

    // Registering again should result in an error.
    assert!(wallet.register(pin).await.is_err());

    wallet
}

pub async fn do_pid_issuance(mut wallet: WalletWithMocks, pin: String) -> WalletWithMocks {
    let redirect_url = wallet
        .create_pid_issuance_auth_url()
        .await
        .expect("Could not create pid issuance auth url");
    let _unsigned_mdocs = wallet
        .continue_pid_issuance(&redirect_url)
        .await
        .expect("Could not continue pid issuance");
    wallet
        .accept_pid_issuance(pin)
        .await
        .expect("Could not accept pid issuance");
    wallet
}

pub struct MockAttributeService;

#[async_trait]
impl AttributeService for MockAttributeService {
    type Error = wallet_server::verifier::Error; // arbitrary type that implements the required trait bounds
    type Settings = ();

    async fn new(_settings: &Self::Settings) -> Result<Self, Self::Error> {
        Ok(MockAttributeService)
    }

    async fn attributes(
        &self,
        _session: &SessionState<Created>,
        _token_request: TokenRequest,
    ) -> Result<Vec<UnsignedMdoc>, Self::Error> {
        let mock_bsn = WSMockBsnLookup::default().bsn("access_token").await.unwrap();
        Ok(WSMockAttributesLookup::default().attributes(&mock_bsn))
    }
}
