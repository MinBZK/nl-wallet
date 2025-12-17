use std::collections::HashMap;
use std::io;
use std::net::IpAddr;
use std::path::PathBuf;
use std::process;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use axum::Json;
use axum::Router;
use axum::routing::post;
use ctor::ctor;
use futures::future;
use p256::ecdsa::SigningKey;
use p256::pkcs8::DecodePrivateKey;
use reqwest::Certificate;
use rustls::crypto::CryptoProvider;
use rustls::crypto::ring;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::PaginatorTrait;
use tokio::net::TcpListener;
use tokio::time;
use url::Url;

use android_attest::android_crl::RevocationStatusList;
use android_attest::mock_chain::MockCaChain;
use android_attest::root_public_key::RootPublicKey;
use apple_app_attest::AppIdentifier;
use apple_app_attest::AttestationEnvironment;
use apple_app_attest::MockAttestationCa;
use attestation_data::issuable_document::IssuableDocument;
use crypto::server_keys::KeyPair;
use crypto::trust_anchor::BorrowingTrustAnchor;
use dcql::CredentialFormat;
use gba_hc_converter::settings::Settings as GbaSettings;
use hsm::service::Pkcs11Hsm;
use http_utils::health::create_health_router;
use http_utils::reqwest::ReqwestTrustAnchor;
use http_utils::reqwest::default_reqwest_client_builder;
use http_utils::reqwest::trusted_reqwest_client_builder;
use http_utils::tls::pinning::TlsPinningConfig;
use http_utils::tls::server::TlsServerConfig;
use http_utils::urls::BaseUrl;
use http_utils::urls::DEFAULT_UNIVERSAL_LINK_BASE;
use http_utils::urls::disclosure_based_issuance_base_uri;
use issuance_server::disclosure::AttributesFetcher;
use issuance_server::disclosure::HttpAttributesFetcher;
use issuance_server::settings::IssuanceServerSettings;
use issuer_settings::settings::IssuerSettings;
use jwt::SignedJwt;
use openid4vc::disclosure_session::VpDisclosureClient;
use openid4vc::issuance_session::HttpIssuanceSession;
use openid4vc::issuer::AttributeService;
use openid4vc::openid4vp::RequestUriMethod;
use openid4vc::openid4vp::VpRequestUriObject;
use openid4vc::token::TokenRequest;
use openid4vc::verifier::SessionType;
use openid4vc::verifier::VerifierUrlParameters;
use pid_issuer::pid::mock::MockAttributeService;
use pid_issuer::pid::mock::mock_issuable_document_address;
use pid_issuer::pid::mock::mock_issuable_document_pid;
use pid_issuer::settings::PidIssuerSettings;
use platform_support::attested_key::mock::MockHardwareAttestedKeyHolder;
use server_utils::keys::PrivateKeyVariant;
use server_utils::settings::Server;
use server_utils::settings::ServerAuth;
use server_utils::settings::ServerSettings;
use server_utils::settings::Settings;
use server_utils::store::SessionStoreVariant;
use static_server::settings::Settings as StaticSettings;
use token_status_list::status_list_service::mock::MockStatusListServices;
use token_status_list::verification::client::mock::MockStatusListServicesClient;
use update_policy_server::settings::Settings as UpsSettings;
use utils::vec_at_least::VecNonEmpty;
use verification_server::settings::VerifierSettings;
use wallet::AttestationPresentation;
use wallet::PidIssuancePurpose;
use wallet::Wallet;
use wallet::WalletClients;
use wallet::test::HttpAccountProviderClient;
use wallet::test::HttpConfigurationRepository;
use wallet::test::MockDigidClient;
use wallet::test::MockDigidSession;
use wallet::test::MockHardwareDatabaseStorage;
use wallet::test::UpdatePolicyRepository;
use wallet::test::UpdateableRepository;
use wallet::test::default_config_server_config;
use wallet::test::default_wallet_config;
use wallet_configuration::config_server_config::ConfigServerConfiguration;
use wallet_configuration::wallet_config::WalletConfiguration;
use wallet_provider::settings::AndroidRootPublicKey;
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

#[derive(Debug, Clone, Copy)]
pub enum WalletDeviceVendor {
    Apple,
    Google,
}

pub type WalletWithStorage = Wallet<
    HttpConfigurationRepository<TlsPinningConfig>,
    UpdatePolicyRepository,
    MockHardwareDatabaseStorage,
    MockHardwareAttestedKeyHolder,
    HttpAccountProviderClient,
    MockDigidClient<TlsPinningConfig>,
    HttpIssuanceSession,
    VpDisclosureClient,
>;

pub async fn setup_wallet_and_default_env(
    vendor: WalletDeviceVendor,
) -> (WalletWithStorage, DisclosureParameters, BaseUrl) {
    setup_wallet_and_env(
        vendor,
        update_policy_server_settings(),
        wallet_provider_settings(),
        pid_issuer_settings(),
        issuance_server_settings(),
    )
    .await
}

pub struct DisclosureParameters {
    pub verifier_url: BaseUrl,
    pub verifier_internal_url: BaseUrl,
}

pub struct MockDeviceConfig {
    pub app_identifier: AppIdentifier,
    pub environment: AttestationEnvironment,
    pub apple_ca: MockAttestationCa,
    pub google_ca: MockCaChain,
}

impl Default for MockDeviceConfig {
    fn default() -> Self {
        Self {
            app_identifier: AppIdentifier::new_mock(),
            environment: AttestationEnvironment::Development,
            apple_ca: MockAttestationCa::generate(),
            google_ca: MockCaChain::generate(1),
        }
    }
}

impl MockDeviceConfig {
    pub fn ios_wp_settings(&self) -> Ios {
        let apple_environment = match self.environment {
            AttestationEnvironment::Development => AppleEnvironment::Development,
            AttestationEnvironment::Production => AppleEnvironment::Production,
        };

        Ios {
            team_identifier: self.app_identifier.prefix().to_string(),
            bundle_identifier: self.app_identifier.bundle_identifier().to_string(),
            environment: apple_environment,
            root_certificates: vec![BorrowingTrustAnchor::from_der(self.apple_ca.certificate().as_ref()).unwrap()],
        }
    }

    pub fn android_root_public_keys(&self) -> Vec<AndroidRootPublicKey> {
        vec![RootPublicKey::Rsa(self.google_ca.root_public_key.clone()).into()]
    }

    pub fn apple_key_holder(&self) -> MockHardwareAttestedKeyHolder {
        MockHardwareAttestedKeyHolder::generate_apple_for_ca(
            self.apple_ca.clone(),
            self.environment,
            self.app_identifier.clone(),
        )
    }

    pub fn google_key_holder(&self) -> MockHardwareAttestedKeyHolder {
        MockHardwareAttestedKeyHolder::generate_google_for_ca(self.google_ca.clone())
    }
}

pub async fn setup_env_default() -> (
    ConfigServerConfiguration,
    MockDeviceConfig,
    WalletConfiguration,
    BaseUrl,
    DisclosureParameters,
) {
    setup_env(
        static_server_settings(),
        update_policy_server_settings(),
        wallet_provider_settings(),
        verification_server_settings(),
        pid_issuer_settings(),
        issuance_server_settings(),
    )
    .await
}

pub async fn setup_env(
    (mut static_settings, static_root_ca): (StaticSettings, ReqwestTrustAnchor),
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
) -> (
    ConfigServerConfiguration,
    MockDeviceConfig,
    WalletConfiguration,
    BaseUrl,
    DisclosureParameters,
) {
    let mock_device_config = MockDeviceConfig::default();
    wp_settings.ios = mock_device_config.ios_wp_settings();
    wp_settings.android.root_public_keys = mock_device_config.android_root_public_keys();

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

    let attestation_settings = attestation_settings(
        &issuer_settings.issuer_settings,
        &issuance_server_settings.issuer_settings,
        Some(hsm.clone()),
    )
    .await;

    let verifier_server_urls =
        start_verification_server(verifier_settings, attestation_settings.clone(), Some(hsm.clone())).await;

    let issuance_server_url = start_issuance_server(
        issuance_server_settings,
        Some(hsm.clone()),
        attributes_fetcher,
        attestation_settings,
    )
    .await;

    let pid_issuer_port = start_pid_issuer_server(
        issuer_settings,
        Some(hsm),
        MockAttributeService::new(pid_issuable_documents),
    )
    .await;

    let config_bytes = read_file("wallet-config.json");
    let mut served_wallet_config: WalletConfiguration = serde_json::from_slice(&config_bytes).unwrap();
    served_wallet_config.pid_issuance.pid_issuer_url = local_pid_base_url(pid_issuer_port);
    served_wallet_config.account_server.http_config.base_url = local_wp_base_url(wp_port);
    served_wallet_config.update_policy_server.http_config.base_url = local_ups_base_url(ups_port);
    served_wallet_config.update_policy_server.http_config.trust_anchors = vec![ups_root_ca.clone()];

    static_settings.wallet_config_jwt = config_jwt(&served_wallet_config).await.into();

    let static_port = start_static_server(static_settings, static_root_ca.clone()).await;
    let config_server_config = ConfigServerConfiguration {
        http_config: TlsPinningConfig {
            base_url: local_config_base_url(static_port),
            trust_anchors: vec![static_root_ca],
        },
        ..default_config_server_config()
    };

    let mut wallet_config = default_wallet_config();
    wallet_config.pid_issuance.pid_issuer_url = local_pid_base_url(pid_issuer_port);
    wallet_config.account_server.http_config.base_url = local_wp_base_url(wp_port);
    wallet_config.update_policy_server.http_config.base_url = local_ups_base_url(ups_port);
    wallet_config.update_policy_server.http_config.trust_anchors = vec![ups_root_ca];

    (
        config_server_config,
        mock_device_config,
        wallet_config,
        issuance_server_url,
        verifier_server_urls,
    )
}

/// Create an instance of [`Wallet`] having temporary file storage.
pub async fn setup_file_wallet(
    config_server_config: ConfigServerConfiguration,
    wallet_config: WalletConfiguration,
    key_holder: MockHardwareAttestedKeyHolder,
    path: PathBuf,
) -> WalletWithStorage {
    setup_wallet(config_server_config, wallet_config, key_holder, async move || {
        MockHardwareDatabaseStorage::open_file(path).await
    })
    .await
}

/// Create an instance of [`Wallet`] having in-memory storage.
pub async fn setup_in_memory_wallet(
    config_server_config: ConfigServerConfiguration,
    wallet_config: WalletConfiguration,
    key_holder: MockHardwareAttestedKeyHolder,
) -> WalletWithStorage {
    setup_wallet(config_server_config, wallet_config, key_holder, async || {
        MockHardwareDatabaseStorage::open_in_memory().await
    })
    .await
}

/// Create an instance of [`Wallet`] having temporary file storage.
pub async fn setup_wallet<F>(
    config_server_config: ConfigServerConfiguration,
    wallet_config: WalletConfiguration,
    key_holder: MockHardwareAttestedKeyHolder,
    storage_generator: F,
) -> WalletWithStorage
where
    F: AsyncFnOnce() -> MockHardwareDatabaseStorage,
{
    let config_repository = HttpConfigurationRepository::new(
        config_server_config.signing_public_key.as_inner().into(),
        tempfile::tempdir().unwrap().keep(),
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

    Wallet::init_registration(
        config_repository,
        update_policy_repository,
        storage_generator().await,
        key_holder,
        wallet_clients,
    )
    .await
    .expect("Could not create test wallet")
}

/// Create an instance of [`Wallet`].
pub async fn setup_wallet_and_env(
    vendor: WalletDeviceVendor,
    ups_config: (UpsSettings, ReqwestTrustAnchor),
    wp_config: (WpSettings, ReqwestTrustAnchor),
    issuer_config: (PidIssuerSettings, VecNonEmpty<IssuableDocument>),
    issuance_config: (
        IssuanceServerSettings,
        Vec<IssuableDocument>,
        ReqwestTrustAnchor,
        TlsServerConfig,
    ),
) -> (WalletWithStorage, DisclosureParameters, BaseUrl) {
    let (config_server_config, mock_device_config, wallet_config, issuance_server_url, verifier_server_urls) =
        setup_env(
            static_server_settings(),
            ups_config,
            wp_config,
            verification_server_settings(),
            issuer_config,
            issuance_config,
        )
        .await;

    let key_holder = match vendor {
        WalletDeviceVendor::Apple => mock_device_config.apple_key_holder(),
        WalletDeviceVendor::Google => mock_device_config.google_key_holder(),
    };

    let wallet = setup_in_memory_wallet(config_server_config, wallet_config, key_holder).await;

    (wallet, verifier_server_urls, issuance_server_url)
}

pub async fn wallet_user_count(connection: &DatabaseConnection) -> u64 {
    wallet_user::Entity::find()
        .count(connection)
        .await
        .expect("Could not fetch user count from database")
}

pub fn static_server_settings() -> (StaticSettings, ReqwestTrustAnchor) {
    let mut settings = StaticSettings::new().expect("Could not read settings");
    settings.ip = IpAddr::from_str("127.0.0.1").unwrap();
    settings.port = 0;

    let root_ca = read_file("static.ca.crt.der").try_into().unwrap();

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

pub async fn start_static_server(settings: StaticSettings, trust_anchor: ReqwestTrustAnchor) -> u16 {
    let listener = TcpListener::bind("localhost:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async {
        if let Err(error) = static_server::server::serve_with_listener(listener, settings).await {
            println!("Could not start config_server: {error:?}");
            process::exit(1);
        }
    });

    let base_url = local_config_base_url(port);
    wait_for_server(remove_path(&base_url), std::iter::once(trust_anchor.into_certificate())).await;
    port
}

pub async fn start_update_policy_server(settings: UpsSettings, trust_anchor: ReqwestTrustAnchor) -> u16 {
    let listener = TcpListener::bind("localhost:0").await.unwrap();
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
    let listener = TcpListener::bind("localhost:0").await.unwrap();
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

    let issuable_documents = vec![
        IssuableDocument::new_mock_degree("BSc".to_string()),
        IssuableDocument::new_mock_degree("MSc".to_string()),
    ];

    (settings, issuable_documents, root_ca, tls_config)
}

pub fn verification_server_settings() -> VerifierSettings {
    let mut settings =
        VerifierSettings::new("verification_server.toml", "verification_server").expect("Could not read settings");

    settings.server_settings.wallet_server.ip = IpAddr::from_str("127.0.0.1").unwrap();
    settings.server_settings.wallet_server.port = 0;

    settings.server_settings.internal_server = ServerAuth::InternalEndpoint(Server {
        ip: IpAddr::from_str("127.0.0.1").unwrap(),
        port: 0,
    });

    settings
}

fn internal_url(server_settings: &Settings) -> BaseUrl {
    match server_settings.internal_server {
        ServerAuth::ProtectedInternalEndpoint {
            server: Server { port, .. },
            ..
        }
        | ServerAuth::InternalEndpoint(Server { port, .. }) => format!("http://localhost:{port}/").parse().unwrap(),
        ServerAuth::Authentication(_) => server_settings.public_url.clone(),
    }
}

async fn attestation_settings(
    pid_issuer_settings: &IssuerSettings,
    disclosure_based_issuance_settings: &IssuerSettings,
    hsm: Option<Pkcs11Hsm>,
) -> HashMap<String, KeyPair<PrivateKeyVariant>> {
    HashMap::from_iter(
        future::join_all(
            pid_issuer_settings
                .attestation_settings
                .as_ref()
                .iter()
                .chain(disclosure_based_issuance_settings.attestation_settings.as_ref().iter())
                .map(|(vct, attestation_type_config_settings)| async {
                    let status_list_attestation_settings = &attestation_type_config_settings.status_list;
                    (
                        vct.clone(),
                        status_list_attestation_settings
                            .keypair
                            .clone()
                            .parse(hsm.clone())
                            .await
                            .unwrap(),
                    )
                }),
        )
        .await,
    )
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
            .unwrap()
            .serve(
                Router::new()
                    .route("/", post(|| async { Json(issuable_documents) }))
                    .merge(create_health_router([]))
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
    attestation_settings: HashMap<String, KeyPair<PrivateKeyVariant>>,
) -> BaseUrl {
    let public_listener = TcpListener::bind("localhost:0").await.unwrap();
    let port = public_listener.local_addr().unwrap().port();
    let public_url = BaseUrl::from_str(format!("http://localhost:{port}/").as_str()).unwrap();
    settings.issuer_settings.server_settings.public_url = public_url.clone();

    let internal_listener = match &mut settings.issuer_settings.server_settings.internal_server {
        ServerAuth::Authentication(_) => None,
        ServerAuth::ProtectedInternalEndpoint { server, .. } | ServerAuth::InternalEndpoint(server) => {
            let listener = TcpListener::bind(("localhost", 0)).await.unwrap();
            server.port = listener.local_addr().unwrap().port();
            Some(listener)
        }
    };

    let storage_settings = &settings.issuer_settings.server_settings.storage;

    let store_connection = server_utils::store::StoreConnection::try_new(storage_settings.url.clone())
        .await
        .unwrap();

    let issuance_sessions = Arc::new(SessionStoreVariant::new(
        store_connection.clone(),
        storage_settings.into(),
    ));
    let disclosure_settings = Arc::new(SessionStoreVariant::new(
        store_connection.clone(),
        storage_settings.into(),
    ));

    tokio::spawn(async move {
        if let Err(error) = issuance_server::server::serve_with_listeners(
            public_listener,
            internal_listener,
            settings,
            hsm,
            issuance_sessions,
            disclosure_settings,
            attributes_fetcher,
            MockStatusListServices::default(),
            None,
            MockStatusListServicesClient::new(attestation_settings),
            create_health_router([]),
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
    let public_listener = TcpListener::bind("localhost:0").await.unwrap();
    let port = public_listener.local_addr().unwrap().port();
    let public_url = BaseUrl::from_str(format!("http://localhost:{port}/").as_str()).unwrap();

    let internal_listener = match &mut settings.issuer_settings.server_settings.internal_server {
        ServerAuth::Authentication(_) => None,
        ServerAuth::ProtectedInternalEndpoint { server, .. } | ServerAuth::InternalEndpoint(server) => {
            let listener = TcpListener::bind(("localhost", 0)).await.unwrap();
            server.port = listener.local_addr().unwrap().port();
            Some(listener)
        }
    };

    let storage_settings = &settings.issuer_settings.server_settings.storage;
    settings.issuer_settings.server_settings.public_url = public_url.clone();

    let store_connection = server_utils::store::StoreConnection::try_new(storage_settings.url.clone())
        .await
        .unwrap();

    let issuance_sessions = Arc::new(SessionStoreVariant::new(
        store_connection.clone(),
        storage_settings.into(),
    ));

    tokio::spawn(async move {
        if let Err(error) = pid_issuer::server::serve_with_listeners(
            public_listener,
            internal_listener,
            attr_service,
            settings.issuer_settings,
            hsm,
            issuance_sessions,
            settings.wua_issuer_pubkey.into_inner(),
            MockStatusListServices::default(),
            None,
            create_health_router([]),
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

pub async fn start_verification_server(
    mut settings: VerifierSettings,
    attestation_settings: HashMap<String, KeyPair<PrivateKeyVariant>>,
    hsm: Option<Pkcs11Hsm>,
) -> DisclosureParameters {
    let wallet_listener = TcpListener::bind("localhost:0").await.unwrap();
    let port = wallet_listener.local_addr().unwrap().port();

    let requester_listener = match &mut settings.server_settings.internal_server {
        ServerAuth::Authentication(_) => None,
        ServerAuth::ProtectedInternalEndpoint { server, .. } | ServerAuth::InternalEndpoint(server) => {
            let listener = TcpListener::bind(("localhost", 0)).await.unwrap();
            server.port = listener.local_addr().unwrap().port();
            Some(listener)
        }
    };

    let public_url = BaseUrl::from_str(format!("http://localhost:{port}/").as_str()).unwrap();
    let internal_url = internal_url(&settings.server_settings);

    let storage_settings = &settings.server_settings.storage;
    settings.server_settings.public_url = public_url.clone();

    let store_connection = server_utils::store::StoreConnection::try_new(storage_settings.url.clone())
        .await
        .unwrap();

    let disclosure_sessions = Arc::new(SessionStoreVariant::new(
        store_connection.clone(),
        storage_settings.into(),
    ));

    tokio::spawn(async move {
        if let Err(error) = verification_server::server::serve_with_listeners(
            wallet_listener,
            requester_listener,
            settings,
            hsm,
            disclosure_sessions,
            MockStatusListServicesClient::new(attestation_settings),
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
        let mut interval = time::interval(Duration::from_millis(100));
        loop {
            match client
                .get(base_url.join("health"))
                .send()
                .await
                .and_then(|r| r.error_for_status())
            {
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

pub async fn do_pid_issuance(wallet: WalletWithStorage, pin: String) -> WalletWithStorage {
    do_pid_issuance_with_purpose(wallet, pin, PidIssuancePurpose::Enrollment).await
}

pub async fn do_pid_renewal(wallet: WalletWithStorage, pin: String) -> WalletWithStorage {
    do_pid_issuance_with_purpose(wallet, pin, PidIssuancePurpose::Renewal).await
}

pub async fn do_pid_issuance_with_purpose(
    mut wallet: WalletWithStorage,
    pin: String,
    purpose: PidIssuancePurpose,
) -> WalletWithStorage {
    let redirect_url = wallet
        .create_pid_issuance_auth_url(purpose)
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

pub fn universal_link(issuance_server_url: &BaseUrl, format: CredentialFormat) -> Url {
    let params = serde_urlencoded::to_string(VerifierUrlParameters {
        session_type: SessionType::SameDevice,
        ephemeral_id_params: None,
    })
    .unwrap();

    let issuance_path = match format {
        CredentialFormat::MsoMdoc => "/disclosure/university_mdoc/request_uri",
        CredentialFormat::SdJwt => "/disclosure/university_sd_jwt/request_uri",
    };
    let mut issuance_server_url = issuance_server_url.join_base_url(issuance_path).into_inner();
    issuance_server_url.set_query(Some(&params));

    let query = serde_urlencoded::to_string(VpRequestUriObject {
        request_uri: issuance_server_url.try_into().unwrap(),
        request_uri_method: Some(RequestUriMethod::POST),
        client_id: "university.example.com".to_string(),
    })
    .unwrap();

    let mut uri = disclosure_based_issuance_base_uri(&DEFAULT_UNIVERSAL_LINK_BASE.parse().unwrap()).into_inner();
    uri.set_query(Some(&query));

    uri
}

pub async fn wallet_attestations(wallet: &mut WalletWithStorage) -> Vec<AttestationPresentation> {
    // Emit attestations into this local variable
    let attestations: Arc<std::sync::Mutex<Vec<AttestationPresentation>>> = Arc::default();

    {
        let attestations = Arc::clone(&attestations);
        wallet
            .set_attestations_callback(Box::new(move |mut a| {
                let mut attestations = attestations.lock().unwrap();
                attestations.append(&mut a);
            }))
            .await
            .unwrap();
    }

    attestations.lock().unwrap().to_vec()
}
