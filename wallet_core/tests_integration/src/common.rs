use std::io;
use std::net::IpAddr;
use std::process;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use axum::Json;
use axum::Router;
use axum::routing::post;
use ctor::ctor;
use p256::ecdsa::SigningKey;
use p256::pkcs8::DecodePrivateKey;
use reqwest::Certificate;
use rustls::crypto::CryptoProvider;
use rustls::crypto::ring;
use sea_orm::Database;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::PaginatorTrait;
use tokio::net::TcpListener;
use tokio::time;
use url::Url;
use wiremock::MockServer;

use android_attest::android_crl::RevocationStatusList;
use android_attest::root_public_key::RootPublicKey;
use apple_app_attest::AppIdentifier;
use apple_app_attest::AttestationEnvironment;
use attestation_data::issuable_document::IssuableDocument;
use configuration_server::settings::Settings as CsSettings;
use crypto::trust_anchor::BorrowingTrustAnchor;
use gba_hc_converter::settings::Settings as GbaSettings;
use hsm::service::Pkcs11Hsm;
use http_utils::reqwest::ReqwestTrustAnchor;
use http_utils::reqwest::default_reqwest_client_builder;
use http_utils::reqwest::trusted_reqwest_client_builder;
use http_utils::tls::pinning::TlsPinningConfig;
use http_utils::tls::server::TlsServerConfig;
use http_utils::urls::BaseUrl;
use issuance_server::disclosure::AttributesFetcher;
use issuance_server::disclosure::HttpAttributesFetcher;
use issuance_server::settings::IssuanceServerSettings;
use jwt::SignedJwt;
use openid4vc::disclosure_session::VpDisclosureClient;
use openid4vc::issuance_session::HttpIssuanceSession;
use openid4vc::issuer::AttributeService;
use openid4vc::token::TokenRequest;
use pid_issuer::pid::mock::MockAttributeService;
use pid_issuer::pid::mock::mock_issuable_document_address;
use pid_issuer::pid::mock::mock_issuable_document_pid;
use pid_issuer::settings::PidIssuerSettings;
use pid_issuer::wua_tracker::WuaTrackerVariant;
use platform_support::attested_key::mock::KeyHolderType;
use platform_support::attested_key::mock::MockHardwareAttestedKeyHolder;
use server_utils::settings::RequesterAuth;
use server_utils::settings::Server;
use server_utils::settings::ServerSettings;
use server_utils::store::SessionStoreVariant;
use update_policy_server::settings::Settings as UpsSettings;
use utils::vec_at_least::VecNonEmpty;
use verification_server::settings::VerifierSettings;
use wallet::Wallet;
use wallet::WalletClients;
use wallet::test::HttpAccountProviderClient;
use wallet::test::HttpConfigurationRepository;
use wallet::test::InMemoryDatabaseStorage;
use wallet::test::MockDigidClient;
use wallet::test::MockDigidSession;
use wallet::test::UpdatePolicyRepository;
use wallet::test::UpdateableRepository;
use wallet::test::default_config_server_config;
use wallet::test::default_wallet_config;
use wallet_configuration::config_server_config::ConfigServerConfiguration;
use wallet_configuration::wallet_config::WalletConfiguration;
use wallet_provider::settings::AppleEnvironment;
use wallet_provider::settings::Ios;
use wallet_provider::settings::Settings as WpSettings;
use wallet_provider_persistence::entity::wallet_user;
use wallet_provider_service::account_server::mock_play_integrity::MockPlayIntegrityClient;

use crate::logging::init_logging;
use crate::utils::read_file;
use crate::utils::remove_path;

#[ctor]
fn init() {
    init_logging();
    CryptoProvider::install_default(ring::default_provider()).unwrap();
}

pub fn local_wp_base_url(port: u16) -> BaseUrl {
    format!("https://localhost:{port}/api/v1/")
        .parse()
        .expect("hardcode values should always parse successfully")
}

pub fn local_config_base_url(port: u16) -> BaseUrl {
    format!("https://localhost:{port}/config/v1/")
        .parse()
        .expect("hardcoded values should always parse successfully")
}

pub fn local_ups_base_url(port: u16) -> BaseUrl {
    format!("https://localhost:{port}/update/v1/")
        .parse()
        .expect("hardcoded values should always parse successfully")
}

pub fn local_pid_base_url(port: u16) -> BaseUrl {
    format!("http://localhost:{port}/issuance/")
        .parse()
        .expect("hardcoded values should always parse successfully")
}

pub async fn database_connection(settings: &WpSettings) -> DatabaseConnection {
    Database::connect(settings.database.connection_string())
        .await
        .expect("Could not open database connection")
}

#[derive(Debug, Clone, Copy)]
pub enum WalletDeviceVendor {
    Apple,
    Google,
}

pub type WalletWithStorage = Wallet<
    HttpConfigurationRepository<TlsPinningConfig>,
    UpdatePolicyRepository,
    InMemoryDatabaseStorage,
    MockHardwareAttestedKeyHolder,
    HttpAccountProviderClient,
    MockDigidClient<TlsPinningConfig>,
    HttpIssuanceSession,
    VpDisclosureClient,
>;

pub async fn setup_wallet_and_default_env(vendor: WalletDeviceVendor) -> WalletWithStorage {
    let (wallet, _, _) = setup_wallet_and_env(
        vendor,
        config_server_settings(),
        update_policy_server_settings(),
        wallet_provider_settings(),
        verification_server_settings(),
        pid_issuer_settings(),
        issuance_server_settings(),
    )
    .await;

    wallet
}

pub struct DisclosureParameters {
    pub verifier_url: BaseUrl,
    pub verifier_internal_url: BaseUrl,
}

pub struct IssuanceParameters {
    pub attestation_server: MockServer,
    pub url: BaseUrl,
}

/// Create an instance of [`Wallet`].
#[expect(clippy::too_many_arguments, reason = "Test setup function")]
pub async fn setup_wallet_and_env(
    vendor: WalletDeviceVendor,
    (mut cs_settings, cs_root_ca): (CsSettings, ReqwestTrustAnchor),
    (ups_settings, ups_root_ca): (UpsSettings, ReqwestTrustAnchor),
    (mut wp_settings, wp_root_ca): (WpSettings, ReqwestTrustAnchor),
    verifier_settings: VerifierSettings,
    (issuer_settings, pid_issuable_documents): (PidIssuerSettings, VecNonEmpty<IssuableDocument>),
    (issuance_server_settings, issuable_documents, di_root_ca, di_tls_config): (
        IssuanceServerSettings,
        Vec<IssuableDocument>,
        ReqwestTrustAnchor,
        TlsServerConfig,
    ),
) -> (WalletWithStorage, DisclosureParameters, BaseUrl) {
    let key_holder = match vendor {
        WalletDeviceVendor::Apple => MockHardwareAttestedKeyHolder::generate_apple(
            AttestationEnvironment::Development,
            AppIdentifier::new_mock(),
        ),
        WalletDeviceVendor::Google => MockHardwareAttestedKeyHolder::generate_google(),
    };

    match &key_holder.holder_type {
        KeyHolderType::Apple {
            ca,
            environment,
            app_identifier,
        } => {
            let apple_environment = match environment {
                AttestationEnvironment::Development => AppleEnvironment::Development,
                AttestationEnvironment::Production => AppleEnvironment::Production,
            };

            wp_settings.ios = Ios {
                team_identifier: app_identifier.prefix().to_string(),
                bundle_identifier: app_identifier.bundle_identifier().to_string(),
                environment: apple_environment,
                root_certificates: vec![BorrowingTrustAnchor::from_der(ca.as_certificate_der().as_ref()).unwrap()],
            };
        }
        KeyHolderType::Google { ca_chain } => {
            wp_settings.android.root_public_keys = vec![RootPublicKey::Rsa(ca_chain.root_public_key.clone()).into()]
        }
    }

    let ups_port = start_update_policy_server(ups_settings, ups_root_ca.clone()).await;

    assert_eq!(Some(wp_settings.hsm.clone()), verifier_settings.server_settings.hsm);
    assert_eq!(
        Some(wp_settings.hsm.clone()),
        issuer_settings.issuer_settings.server_settings.hsm
    );

    let hsm = Pkcs11Hsm::from_settings(wp_settings.hsm.clone()).expect("Could not initialize HSM");
    let wp_port = start_wallet_provider(wp_settings, hsm.clone(), wp_root_ca).await;

    let attestation_server_url =
        start_mock_attestation_server(issuable_documents, di_tls_config, di_root_ca.clone()).await;
    let attributes_fetcher = HttpAttributesFetcher::try_new(
        issuance_server_settings
            .disclosure_settings
            .keys()
            .map(|id| {
                (
                    id.to_string(),
                    TlsPinningConfig {
                        base_url: attestation_server_url.clone(),
                        trust_anchors: vec![di_root_ca.clone()],
                    },
                )
            })
            .collect(),
    )
    .unwrap();

    let wallet_urls = start_verification_server(verifier_settings, Some(hsm.clone())).await;
    let pid_issuer_port = start_pid_issuer_server(
        issuer_settings,
        Some(hsm.clone()),
        MockAttributeService::new(pid_issuable_documents),
    )
    .await;
    let issuance_server_url = start_issuance_server(issuance_server_settings, Some(hsm), attributes_fetcher).await;

    let config_bytes = read_file("wallet-config.json");
    let mut served_wallet_config: WalletConfiguration = serde_json::from_slice(&config_bytes).unwrap();
    served_wallet_config.pid_issuance.pid_issuer_url = local_pid_base_url(pid_issuer_port);
    served_wallet_config.account_server.http_config.base_url = local_wp_base_url(wp_port);
    served_wallet_config.update_policy_server.http_config.base_url = local_ups_base_url(ups_port);
    served_wallet_config.update_policy_server.http_config.trust_anchors = vec![ups_root_ca.clone()];

    cs_settings.wallet_config_jwt = config_jwt(&served_wallet_config).await.into();

    let cs_port = start_config_server(cs_settings, cs_root_ca.clone()).await;
    let config_server_config = ConfigServerConfiguration {
        http_config: TlsPinningConfig {
            base_url: local_config_base_url(cs_port),
            trust_anchors: vec![cs_root_ca],
        },
        ..default_config_server_config()
    };

    let mut wallet_config = default_wallet_config();
    wallet_config.pid_issuance.pid_issuer_url = local_pid_base_url(pid_issuer_port);
    wallet_config.account_server.http_config.base_url = local_wp_base_url(wp_port);
    wallet_config.update_policy_server.http_config.base_url = local_ups_base_url(ups_port);
    wallet_config.update_policy_server.http_config.trust_anchors = vec![ups_root_ca];

    let config_repository = HttpConfigurationRepository::new(
        config_server_config.signing_public_key.as_inner().into(),
        tempfile::tempdir().unwrap().into_path(),
        wallet_config,
    )
    .await
    .unwrap();
    config_repository
        .fetch(&config_server_config.http_config)
        .await
        .unwrap();

    let update_policy_repository = UpdatePolicyRepository::init();
    let mut wallet_clients = WalletClients::new_http(default_reqwest_client_builder()).unwrap();
    setup_mock_digid_client(&mut wallet_clients.digid_client);

    let storage = InMemoryDatabaseStorage::open().await;

    let wallet = Wallet::init_registration(
        config_repository,
        update_policy_repository,
        storage,
        key_holder,
        wallet_clients,
    )
    .await
    .expect("Could not create test wallet");

    (wallet, wallet_urls, issuance_server_url)
}

pub async fn wallet_user_count(connection: &DatabaseConnection) -> u64 {
    wallet_user::Entity::find()
        .count(connection)
        .await
        .expect("Could not fetch user count from database")
}

pub fn config_server_settings() -> (CsSettings, ReqwestTrustAnchor) {
    let mut settings = CsSettings::new().expect("Could not read settings");
    settings.ip = IpAddr::from_str("127.0.0.1").unwrap();
    settings.port = 0;

    let root_ca = read_file("cs.ca.crt.der").try_into().unwrap();

    (settings, root_ca)
}

pub fn update_policy_server_settings() -> (UpsSettings, ReqwestTrustAnchor) {
    let mut settings = UpsSettings::new().expect("Could not read settings");
    settings.ip = IpAddr::from_str("127.0.0.1").unwrap();
    settings.port = 0;

    let root_ca = read_file("ups.ca.crt.der").try_into().unwrap();

    (settings, root_ca)
}

pub async fn config_jwt(wallet_config: &WalletConfiguration) -> SignedJwt<WalletConfiguration> {
    let key = read_file("config_signing.pem");

    SignedJwt::sign(
        wallet_config,
        &SigningKey::from_pkcs8_pem(&String::from_utf8_lossy(&key)).unwrap(),
    )
    .await
    .unwrap()
}

pub fn wallet_provider_settings() -> (WpSettings, ReqwestTrustAnchor) {
    let mut settings = WpSettings::new().expect("Could not read settings");
    settings.webserver.ip = IpAddr::from_str("127.0.0.1").unwrap();
    settings.webserver.port = 0;
    settings.pin_policy.timeouts = vec![200, 400, 600].into_iter().map(Duration::from_millis).collect();

    let root_ca = read_file("wp.ca.crt.der").try_into().unwrap();

    (settings, root_ca)
}

pub async fn start_config_server(settings: CsSettings, trust_anchor: ReqwestTrustAnchor) -> u16 {
    let listener = TcpListener::bind("localhost:0").await.unwrap().into_std().unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async {
        if let Err(error) = configuration_server::server::serve_with_listener(listener, settings).await {
            println!("Could not start config_server: {error:?}");
            process::exit(1);
        }
    });

    let base_url = local_config_base_url(port);
    wait_for_server(remove_path(&base_url), std::iter::once(trust_anchor.into_certificate())).await;
    port
}

pub async fn start_update_policy_server(settings: UpsSettings, trust_anchor: ReqwestTrustAnchor) -> u16 {
    let listener = TcpListener::bind("localhost:0").await.unwrap().into_std().unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async {
        if let Err(error) = update_policy_server::server::serve_with_listener(listener, settings).await {
            println!("Could not start update_policy_server: {error:?}");
            process::exit(1);
        }
    });

    let base_url = local_ups_base_url(port);
    wait_for_server(remove_path(&base_url), std::iter::once(trust_anchor.into_certificate())).await;
    port
}

pub async fn start_wallet_provider(settings: WpSettings, hsm: Pkcs11Hsm, trust_anchor: ReqwestTrustAnchor) -> u16 {
    let listener = TcpListener::bind("localhost:0").await.unwrap().into_std().unwrap();
    let port = listener.local_addr().unwrap().port();

    let play_integrity_client = MockPlayIntegrityClient::new(
        settings.android.package_name.clone(),
        settings.android.play_store_certificate_hashes.clone(),
    );

    tokio::spawn(async {
        if let Err(error) = wallet_provider::server::serve_with_listener(
            listener,
            settings,
            hsm,
            RevocationStatusList::default(),
            play_integrity_client,
        )
        .await
        {
            println!("Could not start wallet_provider: {error:?}");

            process::exit(1);
        }
    });

    let base_url = local_wp_base_url(port);
    wait_for_server(remove_path(&base_url), std::iter::once(trust_anchor.into_certificate())).await;
    port
}

pub fn pid_issuer_settings() -> (PidIssuerSettings, VecNonEmpty<IssuableDocument>) {
    let mut settings = PidIssuerSettings::new("pid_issuer.toml", "pid_issuer").expect("Could not read settings");

    settings.issuer_settings.server_settings.wallet_server.ip = IpAddr::from_str("127.0.0.1").unwrap();
    settings.issuer_settings.server_settings.wallet_server.port = 0;

    (
        settings,
        vec![mock_issuable_document_pid(), mock_issuable_document_address()]
            .try_into()
            .unwrap(),
    )
}

pub fn issuance_server_settings() -> (
    IssuanceServerSettings,
    Vec<IssuableDocument>,
    ReqwestTrustAnchor,
    TlsServerConfig,
) {
    let mut settings =
        IssuanceServerSettings::new("issuance_server.toml", "issuance_server").expect("Could not read settings");

    settings.issuer_settings.server_settings.wallet_server.ip = IpAddr::from_str("127.0.0.1").unwrap();
    settings.issuer_settings.server_settings.wallet_server.port = 0;

    let root_ca = read_file("di.ca.crt.der").try_into().unwrap();
    let tls_config = TlsServerConfig {
        cert: read_file("di.crt.der"),
        key: read_file("di.key.der"),
    };

    (settings, vec![IssuableDocument::new_mock()], root_ca, tls_config)
}

pub fn verification_server_settings() -> VerifierSettings {
    let mut settings =
        VerifierSettings::new("verification_server.toml", "verification_server").expect("Could not read settings");

    settings.server_settings.wallet_server.ip = IpAddr::from_str("127.0.0.1").unwrap();
    settings.server_settings.wallet_server.port = 0;

    settings.requester_server = RequesterAuth::InternalEndpoint(Server {
        ip: IpAddr::from_str("127.0.0.1").unwrap(),
        port: 0,
    });

    settings
}

fn internal_url(settings: &VerifierSettings) -> BaseUrl {
    match settings.requester_server {
        RequesterAuth::ProtectedInternalEndpoint {
            server: Server { port, .. },
            ..
        }
        | RequesterAuth::InternalEndpoint(Server { port, .. }) => format!("http://localhost:{port}/").parse().unwrap(),
        RequesterAuth::Authentication(_) => settings.server_settings.public_url.clone(),
    }
}

async fn start_mock_attestation_server(
    issuable_documents: Vec<IssuableDocument>,
    tls_server_config: TlsServerConfig,
    trust_anchor: ReqwestTrustAnchor,
) -> BaseUrl {
    let listener = TcpListener::bind("localhost:0").await.unwrap().into_std().unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        axum_server::from_tcp_rustls(listener, tls_server_config.into_rustls_config().await.unwrap())
            .serve(
                Router::new()
                    .route("/", post(|| async { Json(issuable_documents) }))
                    .into_make_service(),
            )
            .await
            .expect("issuance server should be started");
    });

    let url: BaseUrl = format!("https://localhost:{port}/").as_str().parse().unwrap();
    wait_for_server(url.clone(), std::iter::once(trust_anchor.into_certificate())).await;
    url
}

pub async fn start_issuance_server(
    mut settings: IssuanceServerSettings,
    hsm: Option<Pkcs11Hsm>,
    attributes_fetcher: impl AttributesFetcher + Sync + 'static,
) -> BaseUrl {
    let listener = TcpListener::bind("localhost:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let public_url = BaseUrl::from_str(format!("http://localhost:{port}/").as_str()).unwrap();
    settings.issuer_settings.server_settings.public_url = public_url.clone();

    let storage_settings = &settings.issuer_settings.server_settings.storage;

    let db_connection = server_utils::store::DatabaseConnection::try_new(storage_settings.url.clone())
        .await
        .unwrap();

    let issuance_sessions = Arc::new(SessionStoreVariant::new(db_connection.clone(), storage_settings.into()));
    let disclosure_settings = Arc::new(SessionStoreVariant::new(db_connection.clone(), storage_settings.into()));

    tokio::spawn(async move {
        if let Err(error) = issuance_server::server::serve_with_listener(
            listener,
            settings,
            hsm,
            issuance_sessions,
            disclosure_settings,
            attributes_fetcher,
        )
        .await
        {
            println!("Could not start issuance_server: {error:?}");

            process::exit(1);
        }
    });

    wait_for_server(public_url.clone(), std::iter::empty()).await;
    public_url
}

pub async fn start_pid_issuer_server<A: AttributeService + Send + Sync + 'static>(
    mut settings: PidIssuerSettings,
    hsm: Option<Pkcs11Hsm>,
    attr_service: A,
) -> u16 {
    let listener = TcpListener::bind("localhost:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let public_url = BaseUrl::from_str(format!("http://localhost:{port}/").as_str()).unwrap();

    let storage_settings = &settings.issuer_settings.server_settings.storage;
    settings.issuer_settings.server_settings.public_url = public_url.clone();

    let db_connection = server_utils::store::DatabaseConnection::try_new(storage_settings.url.clone())
        .await
        .unwrap();

    let issuance_sessions = Arc::new(SessionStoreVariant::new(db_connection.clone(), storage_settings.into()));
    let wua_tracker = WuaTrackerVariant::new(db_connection);

    tokio::spawn(async move {
        if let Err(error) = pid_issuer::server::serve_with_listener(
            listener,
            attr_service,
            settings.issuer_settings,
            hsm,
            issuance_sessions,
            settings.wua_issuer_pubkey.into_inner(),
            wua_tracker,
        )
        .await
        {
            println!("Could not start pid_issuer: {error:?}");

            process::exit(1);
        }
    });

    wait_for_server(public_url, std::iter::empty()).await;
    port
}

pub async fn start_verification_server(mut settings: VerifierSettings, hsm: Option<Pkcs11Hsm>) -> DisclosureParameters {
    let listener = TcpListener::bind("localhost:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let requester_listener = match &mut settings.requester_server {
        RequesterAuth::Authentication(_) => None,
        RequesterAuth::ProtectedInternalEndpoint { server, .. } | RequesterAuth::InternalEndpoint(server) => {
            let listener = TcpListener::bind(("localhost", 0)).await.unwrap();
            server.port = listener.local_addr().unwrap().port();
            Some(listener)
        }
    };

    let public_url = BaseUrl::from_str(format!("http://localhost:{port}/").as_str()).unwrap();
    let internal_url = internal_url(&settings);

    let storage_settings = &settings.server_settings.storage;
    settings.server_settings.public_url = public_url.clone();

    let db_connection = server_utils::store::DatabaseConnection::try_new(storage_settings.url.clone())
        .await
        .unwrap();

    let disclosure_sessions = Arc::new(SessionStoreVariant::new(db_connection.clone(), storage_settings.into()));

    tokio::spawn(async move {
        if let Err(error) = verification_server::server::serve_with_listeners(
            listener,
            requester_listener,
            settings,
            hsm,
            disclosure_sessions,
        )
        .await
        {
            println!("Could not start verification_server: {error:?}");

            process::exit(1);
        }
    });

    wait_for_server(public_url.clone(), std::iter::empty()).await;
    DisclosureParameters {
        verifier_url: public_url,
        verifier_internal_url: internal_url,
    }
}

pub async fn wait_for_server(base_url: BaseUrl, trust_anchors: impl Iterator<Item = Certificate>) {
    let client = trusted_reqwest_client_builder(trust_anchors).build().unwrap();

    time::timeout(Duration::from_secs(3), async {
        let mut interval = time::interval(Duration::from_millis(10));
        loop {
            match client.get(base_url.join("health")).send().await {
                Ok(_) => break,
                Err(e) => {
                    println!("Server not yet up: {e:?}");
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
            if let Some(io_error) = error.downcast_ref::<io::Error>()
                && io_error.kind() == io::ErrorKind::AddrInUse
            {
                println!(
                    "TCP address/port for gba_hc_converter is already in use, assuming you started it yourself, \
                     continuing..."
                );
                return;
            }
            println!("Could not start gba_hc_converter: {error:?}");
            process::exit(1);
        }
    });

    wait_for_server(base_url, std::iter::empty()).await;
}

pub async fn do_wallet_registration(mut wallet: WalletWithStorage, pin: &str) -> WalletWithStorage {
    // No registration should be loaded initially.
    assert!(!wallet.has_registration());

    // Register with a valid PIN.
    wallet.register(pin).await.expect("Could not register wallet");

    // The registration should now be loaded.
    assert!(wallet.has_registration());

    // Registering again should result in an error.
    assert!(wallet.register(pin).await.is_err());

    wallet
}

pub async fn do_pid_issuance(mut wallet: WalletWithStorage, pin: String) -> WalletWithStorage {
    let redirect_url = wallet
        .create_pid_issuance_auth_url()
        .await
        .expect("Could not create pid issuance auth url");
    let _attestations = wallet
        .continue_pid_issuance(redirect_url)
        .await
        .expect("Could not continue pid issuance");
    wallet
        .accept_issuance(pin)
        .await
        .expect("Could not accept pid issuance");
    wallet
}

/// Configure [`MockDigidClient`] to return a [`MockDigidClient`] that returns some arbitrary token.
pub fn setup_mock_digid_client(digid_client: &mut MockDigidClient<TlsPinningConfig>) {
    digid_client
        .expect_start_session()
        .returning(|_digid_config, _http_config, _redirect_uri| {
            let mut session = MockDigidSession::new();

            session
                .expect_auth_url()
                .return_const(Url::parse("http://localhost/").unwrap());

            session
                .expect_into_token_request()
                .times(1)
                .return_once(|_http_config, _redirect_uri| {
                    let token_request = TokenRequest {
                        grant_type: openid4vc::token::TokenRequestGrantType::PreAuthorizedCode {
                            pre_authorized_code: crypto::utils::random_string(32).into(),
                        },
                        code_verifier: Some("my_code_verifier".to_string()),
                        client_id: Some("my_client_id".to_string()),
                        redirect_uri: Some("redirect://here".parse().unwrap()),
                    };

                    Ok(token_request)
                });

            Ok(session)
        });
}
