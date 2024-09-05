use std::{
    any::Any,
    io,
    net::{IpAddr, TcpListener},
    process,
    str::FromStr,
    time::Duration,
};

use ctor::ctor;
use indexmap::IndexMap;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use reqwest::Certificate;
use sea_orm::{Database, DatabaseConnection, EntityTrait, PaginatorTrait};
use tokio::time;
use url::Url;
use uuid::Uuid;

use configuration_server::settings::Settings as CsSettings;
use gba_hc_converter::settings::Settings as GbaSettings;
use nl_wallet_mdoc::utils::x509;
use openid4vc::{
    disclosure_session::{DisclosureSession, HttpVpMessageClient},
    issuance_session::HttpIssuanceSession,
    issuer::{AttributeService, Created},
    oidc,
    server_state::SessionState,
    token::{CredentialPreview, TokenRequest},
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
use wallet_common::{
    config::wallet_config::WalletConfiguration, keys::software::SoftwareEcdsaKey, nonempty::NonEmpty,
    reqwest::trusted_reqwest_client_builder, urls::BaseUrl, utils,
};
use wallet_provider::settings::Settings as WpSettings;
use wallet_provider_persistence::entity::wallet_user;
use wallet_server::{
    pid::mock::MockAttributesLookup,
    settings::{RequesterAuth, Server, Settings as WsSettings},
    store::SessionStoreVariant,
};

use crate::logging::init_logging;

#[ctor]
fn init() {
    init_logging();
}

pub fn local_wp_base_url(port: &u16) -> BaseUrl {
    format!("http://localhost:{}/api/v1/", port)
        .parse()
        .expect("hardcode values should always parse successfully")
}

pub fn local_config_base_url(port: &u16) -> BaseUrl {
    format!("https://localhost:{}/config/v1/", port)
        .parse()
        .expect("hardcode values should always parse successfully")
}

pub fn local_pid_base_url(port: &u16) -> BaseUrl {
    format!("http://localhost:{}/issuance/", port)
        .parse()
        .expect("hardcode values should always parse successfully")
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
    HttpIssuanceSession,
    DisclosureSession<HttpVpMessageClient, Uuid>,
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

    let certificates = ws_settings.issuer.certificates();

    start_config_server(cs_settings).await;
    start_wallet_provider(wp_settings).await;
    start_wallet_server(ws_settings, MockAttributeService(certificates)).await;

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

pub fn find_listener_port() -> u16 {
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
    settings.requester_server = RequesterAuth::InternalEndpoint(Server {
        ip: IpAddr::from_str("127.0.0.1").unwrap(),
        port: requester_port,
    });

    settings.urls.public_url = format!("http://localhost:{}/", ws_port).parse().unwrap();
    settings
}

pub fn wallet_server_internal_url(auth: &RequesterAuth, public_url: &BaseUrl) -> BaseUrl {
    match auth {
        RequesterAuth::ProtectedInternalEndpoint {
            server: Server { port, .. },
            ..
        }
        | RequesterAuth::InternalEndpoint(Server { port, .. }) => format!("http://localhost:{port}/").parse().unwrap(),
        RequesterAuth::Authentication(_) => public_url.clone(),
    }
}

pub async fn start_wallet_server<A: AttributeService + Send + Sync + 'static>(settings: WsSettings, attr_service: A) {
    let storage_settings = &settings.storage;
    let public_url = settings.urls.public_url.clone();
    let disclosure_sessions = SessionStoreVariant::new(storage_settings.url.clone(), storage_settings.into())
        .await
        .unwrap();
    let issuance_sessions = disclosure_sessions.clone_into();
    tokio::spawn(async move {
        if let Err(error) =
            wallet_server::server::wallet_server::serve(attr_service, settings, disclosure_sessions, issuance_sessions)
                .await
        {
            println!("Could not start wallet_server: {:?}", error);

            process::exit(1);
        }
    });

    wait_for_server(public_url.join_base_url("disclosure/"), vec![]).await;
}

pub async fn wait_for_server(base_url: BaseUrl, trust_anchors: Vec<Certificate>) {
    let client = trusted_reqwest_client_builder(trust_anchors).build().unwrap();

    time::timeout(Duration::from_secs(3), async {
        let mut interval = time::interval(Duration::from_millis(10));
        loop {
            match client.get(base_url.join("health")).send().await {
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

pub fn gba_hc_converter_settings() -> GbaSettings {
    // We cannot use a random port here, since the BRP proxy needs to connect to the converter on a set port.
    let mut settings = GbaSettings::new().expect("Could not read settings");
    settings.ip = IpAddr::from_str("127.0.0.1").unwrap();
    settings
}

pub async fn start_gba_hc_converter(settings: GbaSettings) {
    let base_url = format!("http://localhost:{}/", settings.port)
        .parse()
        .expect("hardcode values should always parse successfully");

    tokio::spawn(async {
        if let Err(error) = gba_hc_converter::app::serve_from_settings(settings).await {
            if let Some(io_error) = error.downcast_ref::<io::Error>() {
                if io_error.kind() == io::ErrorKind::AddrInUse {
                    println!(
                        "TCP address/port for gba_hc_converter is already in use, assuming you started it yourself, \
                         continuing..."
                    );
                    return;
                }
            }
            println!("Could not start gba_hc_converter: {:?}", error);
            process::exit(1);
        }
    });

    wait_for_server(base_url, vec![]).await;
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
        .continue_pid_issuance(redirect_url)
        .await
        .expect("Could not continue pid issuance");
    wallet
        .accept_pid_issuance(pin)
        .await
        .expect("Could not accept pid issuance");
    wallet
}

pub struct MockAttributeService(pub IndexMap<String, x509::Certificate>);

impl AttributeService for MockAttributeService {
    type Error = std::convert::Infallible;

    async fn attributes(
        &self,
        _session: &SessionState<Created>,
        _token_request: TokenRequest,
    ) -> Result<NonEmpty<Vec<CredentialPreview>>, Self::Error> {
        let attributes = MockAttributesLookup::default()
            .attributes("999991772")
            .unwrap()
            .into_iter()
            .map(|unsigned_mdoc| CredentialPreview::MsoMdoc {
                issuer: self.0[&unsigned_mdoc.doc_type].clone(),
                unsigned_mdoc,
            })
            .collect::<Vec<_>>();
        Ok(attributes.try_into().unwrap())
    }

    async fn oauth_metadata(&self, issuer_url: &BaseUrl) -> Result<oidc::Config, Self::Error> {
        Ok(oidc::Config::new_mock(issuer_url))
    }
}

// The type of MockDigidSession::Context is too complex, but keeping ownership is important.
pub fn setup_digid_context() -> Box<dyn Any> {
    let digid_context = MockDigidSession::start_context();
    digid_context.expect().return_once(|_, _| {
        let mut session = MockDigidSession::default();

        session.expect_into_token_request().return_once(|_url| {
            Ok(TokenRequest {
                grant_type: openid4vc::token::TokenRequestGrantType::PreAuthorizedCode {
                    pre_authorized_code: utils::random_string(32).into(),
                },
                code_verifier: Some("my_code_verifier".to_string()),
                client_id: Some("my_client_id".to_string()),
                redirect_uri: Some("redirect://here".parse().unwrap()),
            })
        });

        Ok((session, Url::parse("http://localhost/").unwrap()))
    });
    Box::new(digid_context)
}
