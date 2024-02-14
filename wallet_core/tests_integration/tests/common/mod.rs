use std::{
    net::{IpAddr, TcpListener},
    process,
    str::FromStr,
    time::Duration,
};

use ctor::ctor;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use reqwest::Certificate;
use sea_orm::{Database, DatabaseConnection, EntityTrait, PaginatorTrait};
use tokio::time;
use url::Url;

use configuration_server::settings::Settings as CsSettings;
use nl_wallet_mdoc::{basic_sa_ext::UnsignedMdoc, server_state::SessionState};
use openid4vc::{
    issuance_client::HttpIssuerClient,
    issuer::{AttributeService, Created},
    token::TokenRequest,
};
use platform_support::utils::{software::SoftwareUtilities, PlatformUtilities};
use wallet::{
    mock::{default_configuration, MockDigidSession, MockStorage},
    wallet_deps::{
        ConfigServerConfiguration, HttpAccountProviderClient, HttpConfigurationRepository,
        UpdateableConfigurationRepository,
    },
    Wallet,
};
use wallet_common::{config::wallet_config::WalletConfiguration, keys::software::SoftwareEcdsaKey};
use wallet_provider::settings::Settings as WpSettings;
use wallet_provider_persistence::entity::wallet_user;
use wallet_server::{
    pid::mock::{MockAttributesLookup as WSMockAttributesLookup, MockBsnLookup as WSMockBsnLookup},
    settings::{Server, Settings as WsSettings},
    store::SessionStores,
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
    Url::parse(&format!("https://localhost:{}/config/v1/", port)).expect("Could not create url")
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
    HttpIssuerClient,
>;

pub async fn setup_wallet_and_default_env() -> WalletWithMocks {
    setup_wallet_and_env(
        config_server_settings(),
        wallet_provider_settings(),
        wallet_server_settings(),
    )
    .await
}

/// Create an instance of [`Wallet`].
pub async fn setup_wallet_and_env(
    mut cs_settings: CsSettings,
    wp_settings: WpSettings,
    ws_settings: WsSettings,
) -> WalletWithMocks {
    let config_server_config = ConfigServerConfiguration {
        base_url: local_config_base_url(&cs_settings.port),
        ..Default::default()
    };

    let mut wallet_config = default_configuration();
    wallet_config.pid_issuance.pid_issuer_url = local_pid_base_url(&ws_settings.wallet_server.port);
    wallet_config.account_server.base_url = local_wp_base_url(&wp_settings.webserver.port);

    let config_bytes = configuration_server::read_file("wallet-config.json");
    let mut served_wallet_config: WalletConfiguration = serde_json::from_slice(&config_bytes).unwrap();
    served_wallet_config.pid_issuance.pid_issuer_url = local_pid_base_url(&ws_settings.wallet_server.port);
    served_wallet_config.account_server.base_url = local_wp_base_url(&wp_settings.webserver.port);

    cs_settings.wallet_config_jwt = config_jwt(&served_wallet_config);

    start_config_server(cs_settings).await;
    start_wallet_provider(wp_settings).await;
    start_wallet_server(ws_settings).await;

    let config_repository = HttpConfigurationRepository::new(
        config_server_config.base_url,
        config_server_config.trust_anchors,
        (&config_server_config.signing_public_key).into(),
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

pub fn config_server_settings() -> CsSettings {
    let port = find_listener_port();

    let mut settings = CsSettings::new().expect("Could not read settings");
    settings.ip = IpAddr::from_str("127.0.0.1").unwrap();
    settings.port = port;
    settings
}

pub fn config_jwt(wallet_config: &WalletConfiguration) -> String {
    let key = configuration_server::read_file("config_signing.pem");

    jsonwebtoken::encode(
        &Header {
            alg: Algorithm::ES256,
            ..Default::default()
        },
        wallet_config,
        &EncodingKey::from_ec_pem(&key).unwrap(),
    )
    .unwrap()
}

pub fn wallet_provider_settings() -> WpSettings {
    let port = find_listener_port();

    let mut settings = WpSettings::new().expect("Could not read settings");
    settings.webserver.ip = IpAddr::from_str("127.0.0.1").unwrap();
    settings.webserver.port = port;
    settings.pin_policy.timeouts = vec![200, 400, 600].into_iter().map(Duration::from_millis).collect();
    settings
}

pub async fn start_config_server(settings: CsSettings) {
    let base_url = local_config_base_url(&settings.port);
    let root_ca = Certificate::from_pem(&configuration_server::read_file("ca.crt.pem")).unwrap();

    tokio::spawn(async {
        if let Err(error) = configuration_server::server::serve(settings).await {
            println!("Could not start config_server: {:?}", error);
            process::exit(1);
        }
    });

    wait_for_server(base_url, vec![root_ca]).await;
}

pub async fn start_wallet_provider(settings: WpSettings) {
    let base_url = local_wp_base_url(&settings.webserver.port);
    tokio::spawn(async {
        if let Err(error) = wallet_provider::server::serve(settings).await {
            println!("Could not start wallet_provider: {:?}", error);

            process::exit(1);
        }
    });

    wait_for_server(base_url, vec![]).await;
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

pub async fn start_wallet_server(settings: WsSettings) {
    let public_url = settings.public_url.clone();
    let sessions = SessionStores::init(settings.store_url.clone()).await.unwrap();
    tokio::spawn(async move {
        if let Err(error) = wallet_server::server::serve_full(MockAttributeService, settings, sessions).await {
            println!("Could not start wallet_server: {:?}", error);

            process::exit(1);
        }
    });

    wait_for_server(public_url.join("disclosure/").unwrap(), vec![]).await;
}

async fn wait_for_server(base_url: Url, trust_anchors: Vec<Certificate>) {
    let client = trust_anchors
        .into_iter()
        .fold(reqwest::Client::builder(), |builder, anchor| {
            builder.add_root_certificate(anchor)
        })
        .build()
        .unwrap();

    time::timeout(Duration::from_secs(3), async {
        let mut interval = time::interval(Duration::from_millis(10));
        loop {
            match client.get(base_url.join("health").unwrap()).send().await {
                Ok(_) => break,
                Err(e) => {
                    println!("Server not yet up: {e}");
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

impl AttributeService for MockAttributeService {
    type Error = wallet_server::verifier::Error; // arbitrary type that implements the required trait bounds

    async fn attributes(
        &self,
        _session: &SessionState<Created>,
        _token_request: TokenRequest,
    ) -> Result<Vec<UnsignedMdoc>, Self::Error> {
        let mock_bsn = WSMockBsnLookup::default().bsn("access_token").await.unwrap();
        Ok(WSMockAttributesLookup::default().attributes(&mock_bsn).unwrap())
    }
}
