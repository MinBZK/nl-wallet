use std::convert::Infallible;
use std::io;
use std::net::IpAddr;
use std::path::PathBuf;
use std::process;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use android_attest::android_crl::RevocationStatusList;
use android_attest::mock_chain::MockCaChain;
use android_attest::root_public_key::RootPublicKey;
use apple_app_attest::AppIdentifier;
use apple_app_attest::AttestationEnvironment;
use apple_app_attest::MockAttestationCa;
use attestation_types::credential_format::Format;
use axum::Json;
use axum::Router;
use axum::routing::post;
use crypto::trust_anchor::BorrowingTrustAnchor;
use ctor::ctor;
use db_test::DbSetup;
use gba_hc_converter::settings::Settings as GbaSettings;
use hsm::service::Pkcs11Hsm;
use http_utils::client::TlsPinningConfig;
use http_utils::health::create_health_router;
use http_utils::reqwest::ReqwestTrustAnchor;
use http_utils::reqwest::default_reqwest_client_builder;
use http_utils::reqwest::tls_reqwest_client_builder;
use http_utils::server::TlsServerConfig;
use http_utils::urls::BaseUrl;
use http_utils::urls::DEFAULT_UNIVERSAL_LINK_BASE;
use http_utils::urls::disclosure_based_issuance_base_uri;
use issuance_server::settings::IssuanceServerSettings;
use issuer_common::state_bridge_store::IssuerStateBridgeStore;
use jwt::SignedJwt;
use openid4vc::disclosure_session::DisclosureUriSource;
use openid4vc::disclosure_session::VpDisclosureClient;
use openid4vc::issuable_document::IssuableDocument;
use openid4vc::issuer::WiaConfig;
use openid4vc::issuer_identifier::IssuerIdentifier;
use openid4vc::openid4vp::ClientId;
use openid4vc::openid4vp::VpRequestUri;
use openid4vc::openid4vp::VpRequestUriMethod;
use openid4vc::openid4vp::VpRequestUriObject;
use openid4vc::verifier::SessionType;
use openid4vc::verifier::VerifierUrlParameters;
use openid4vc::wallet_issuance::discovery::HttpIssuanceDiscovery;
use p256::ecdsa::SigningKey;
use p256::pkcs8::DecodePrivateKey;
use pid_issuer::pid::auth_code_flow::UpstreamOidcAuthorizationCodeFlow;
use pid_issuer::pid::brp::client::BrpClient;
use pid_issuer::pid::digid::DigidClient;
use pid_issuer::pid::mock::MockBrpClient;
use pid_issuer::pid::mock::MockDigidClient;
use pid_issuer::settings::PidIssuerSettings;
use platform_support::attested_key::mock::MockHardwareAttestedKeyHolder;
use reqwest::Certificate;
use reqwest::header;
use reqwest::redirect::Policy;
use rustls::crypto::CryptoProvider;
use rustls::crypto::ring;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::PaginatorTrait;
use server_utils::keys::SecretKeyVariant;
use server_utils::settings::Server;
use server_utils::settings::ServerAuth;
use server_utils::settings::ServerSettings;
use server_utils::store::SessionStoreVariant;
use server_utils::store::StoreConnection;
pub use server_utils::store::postgres::new_connection;
use static_server::settings::Settings as StaticSettings;
use token_status_list::verification::reqwest::HttpStatusListClient;
use tokio::net::TcpListener;
use tokio::time;
use tracing::Instrument;
use tracing::info_span;
use update_policy_server::settings::Settings as UpsSettings;
use url::Url;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_nonempty;
use verification_server::settings::VerifierSettings;
use wallet::AttestationPresentation;
use wallet::PidIssuancePurpose;
use wallet::Wallet;
use wallet::WalletClients;
use wallet::WalletRepositories;
use wallet::test::HttpAccountProviderClient;
use wallet::test::HttpConfigurationRepository;
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

pub fn local_http_base_url(port: u16) -> BaseUrl {
    format!("http://localhost:{port}/")
        .parse()
        .expect("hardcoded values should always parse successfully")
}

pub fn local_https_base_url(port: u16) -> BaseUrl {
    format!("https://localhost:{port}/")
        .parse()
        .expect("hardcoded values should always parse successfully")
}

pub fn local_http_issuer_identifier(port: u16) -> IssuerIdentifier {
    format!("http://localhost:{port}/")
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
    HttpIssuanceDiscovery,
    VpDisclosureClient,
>;

pub async fn setup_wallet_and_default_env(
    db_setup: &DbSetup,
    vendor: WalletDeviceVendor,
) -> (WalletWithStorage, DisclosureUrls, IssuerData) {
    setup_wallet_and_env(
        db_setup,
        vendor,
        update_policy_server_settings(),
        wallet_provider_settings(db_setup.wallet_provider_url(), db_setup.audit_log_url()),
        pid_issuer_settings(db_setup.pid_issuer_url()),
        issuance_server_settings(db_setup.issuance_server_url()),
    )
    .await
}

pub struct DegreeClientIds {
    mdoc: ClientId,
    sd_jwt: ClientId,
}

impl DegreeClientIds {
    fn from_settings(settings: &IssuanceServerSettings) -> Self {
        Self {
            mdoc: ClientId::x509_hash_from_certificate(
                &settings.verifier_settings.disclosure_settings["university_mdoc"]
                    .key_pair
                    .certificate,
            ),
            sd_jwt: ClientId::x509_hash_from_certificate(
                &settings.verifier_settings.disclosure_settings["university_sd_jwt"]
                    .key_pair
                    .certificate,
            ),
        }
    }

    pub fn for_format(&self, format: Format) -> &ClientId {
        match format {
            Format::MsoMdoc => &self.mdoc,
            Format::SdJwt => &self.sd_jwt,
        }
    }
}

pub struct DisclosureUrls {
    pub verifier_url: BaseUrl,
    pub verifier_internal_url: BaseUrl,
}

#[derive(Debug)]
pub struct IssuerUrl {
    pub internal: BaseUrl,
    pub public: IssuerIdentifier,
}

pub struct IssuerData {
    pub pid_issuer: IssuerUrl,
    pub issuance_server: IssuerUrl,
    pub degree_client_ids: DegreeClientIds,
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

pub async fn setup_env_default(
    db_setup: &DbSetup,
) -> (
    ConfigServerConfiguration,
    MockDeviceConfig,
    WalletConfiguration,
    IssuerData,
    DisclosureUrls,
) {
    setup_env(
        static_server_settings(),
        update_policy_server_settings(),
        wallet_provider_settings(db_setup.wallet_provider_url(), db_setup.audit_log_url()),
        verification_server_settings(db_setup.verification_server_url()),
        pid_issuer_settings(db_setup.pid_issuer_url()),
        issuance_server_settings(db_setup.issuance_server_url()),
    )
    .await
}

pub async fn setup_env(
    (mut static_settings, static_root_ca): (StaticSettings, ReqwestTrustAnchor),
    (ups_settings, ups_root_ca): (UpsSettings, ReqwestTrustAnchor),
    (mut wp_settings, wp_root_ca): (WpSettings, ReqwestTrustAnchor),
    verifier_settings: VerifierSettings,
    issuer_settings: PidIssuerSettings,
    (mut issuance_server_settings, issuable_documents, di_root_ca, di_tls_config): (
        IssuanceServerSettings,
        Vec<IssuableDocument>,
        ReqwestTrustAnchor,
        TlsServerConfig,
    ),
) -> (
    ConfigServerConfiguration,
    MockDeviceConfig,
    WalletConfiguration,
    IssuerData,
    DisclosureUrls,
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

    for config in issuance_server_settings
        .verifier_settings
        .disclosure_settings
        .values_mut()
    {
        config.attestation_url_config =
            TlsPinningConfig::try_new(attestation_server_url.clone(), vec_nonempty![di_root_ca.clone()]).unwrap();
    }

    let verifier_server_urls = start_verification_server(verifier_settings, Some(hsm.clone())).await;

    let degree_client_ids = DegreeClientIds::from_settings(&issuance_server_settings);

    let issuance_server_url = start_issuance_server(issuance_server_settings, Some(hsm.clone())).await;

    let recovery_code_secret_key =
        SecretKeyVariant::from_settings(issuer_settings.recovery_code.clone(), Some(hsm.clone()))
            .expect("could not initialize recovery code secret key");

    let pid_issuer_url = start_pid_issuer_server(issuer_settings, Some(hsm), |public_url| {
        // The production `UpstreamOidcAuthorizationCodeFlow` (state bridge, callback handler, BRP
        // attribute mapping, recovery-code HMAC) with only its two external boundaries mocked:
        // `MockDigidClient` redirects the user-agent straight back to the issuer's own
        // `/digid/callback` and returns a fixed BSN, and `MockBrpClient` serves a bundled
        // haal-centraal person fixture.

        UpstreamOidcAuthorizationCodeFlow::new(
            MockBrpClient::default(),
            MockDigidClient::default(),
            recovery_code_secret_key,
            Arc::new(IssuerStateBridgeStore::new(StoreConnection::Memory)),
            public_url.as_base_url().clone(),
            "mock-digid-client".to_string(),
        )
    })
    .await;

    let issuer_data = IssuerData {
        issuance_server: issuance_server_url,
        pid_issuer: pid_issuer_url,
        degree_client_ids,
    };

    let pid_credential_offer = create_pid_credential_offer(&issuer_data.pid_issuer.public);

    let config_bytes = read_file("wallet-config.json");
    let mut served_wallet_config: WalletConfiguration = serde_json::from_slice(&config_bytes).unwrap();
    served_wallet_config.pid_credential_offer = pid_credential_offer.clone();
    served_wallet_config.account_server.http_config = TlsPinningConfig::try_new(
        local_wp_base_url(wp_port),
        VecNonEmpty::try_from(served_wallet_config.account_server.http_config.trust_anchors().to_vec()).unwrap(),
    )
    .unwrap();
    served_wallet_config.update_policy_server.http_config =
        TlsPinningConfig::try_new(local_ups_base_url(ups_port), vec_nonempty![ups_root_ca.clone()]).unwrap();

    static_settings.wallet_config_jwt = config_jwt(&served_wallet_config).await.into();

    let static_port = start_static_server(static_settings, static_root_ca.clone()).await;
    let config_server_config = ConfigServerConfiguration {
        http_config: TlsPinningConfig::try_new(local_config_base_url(static_port), vec_nonempty![static_root_ca])
            .unwrap(),
        ..default_config_server_config()
    };

    let mut wallet_config = default_wallet_config();
    wallet_config.pid_credential_offer = pid_credential_offer.clone();
    wallet_config.account_server.http_config = TlsPinningConfig::try_new(
        local_wp_base_url(wp_port),
        VecNonEmpty::try_from(wallet_config.account_server.http_config.trust_anchors().to_vec()).unwrap(),
    )
    .unwrap();
    wallet_config.update_policy_server.http_config =
        TlsPinningConfig::try_new(local_ups_base_url(ups_port), vec_nonempty![ups_root_ca]).unwrap();

    (
        config_server_config,
        mock_device_config,
        wallet_config,
        issuer_data,
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

    let wallet_clients = WalletClients::new().unwrap();

    Wallet::init_registration(
        storage_generator().await,
        key_holder,
        WalletRepositories {
            config_repository,
            update_policy_repository,
        },
        wallet_clients,
    )
    .await
    .expect("Could not create test wallet")
}

/// Create an instance of [`Wallet`].
pub async fn setup_wallet_and_env(
    db_setup: &DbSetup,
    vendor: WalletDeviceVendor,
    ups_config: (UpsSettings, ReqwestTrustAnchor),
    wp_config: (WpSettings, ReqwestTrustAnchor),
    issuer_config: PidIssuerSettings,
    issuance_config: (
        IssuanceServerSettings,
        Vec<IssuableDocument>,
        ReqwestTrustAnchor,
        TlsServerConfig,
    ),
) -> (WalletWithStorage, DisclosureUrls, IssuerData) {
    let (config_server_config, mock_device_config, wallet_config, issuer_data, verifier_server_urls) = setup_env(
        static_server_settings(),
        ups_config,
        wp_config,
        verification_server_settings(db_setup.verification_server_url()),
        issuer_config,
        issuance_config,
    )
    .await;

    let key_holder = match vendor {
        WalletDeviceVendor::Apple => mock_device_config.apple_key_holder(),
        WalletDeviceVendor::Google => mock_device_config.google_key_holder(),
    };

    let wallet = setup_in_memory_wallet(config_server_config, wallet_config, key_holder).await;

    (wallet, verifier_server_urls, issuer_data)
}

pub async fn wallet_user_count(connection: &DatabaseConnection) -> u64 {
    wallet_user::Entity::find()
        .count(connection)
        .await
        .expect("Could not fetch user count from database")
}

pub async fn get_all_wallet_ids(connection: &DatabaseConnection) -> Vec<String> {
    wallet_user::Entity::find()
        .all(connection)
        .await
        .unwrap()
        .into_iter()
        .map(|user| user.wallet_id)
        .collect()
}

pub async fn get_wallet_recovery_code(connection: &DatabaseConnection, wallet_id: &str) -> String {
    use sea_orm::ColumnTrait;
    use sea_orm::QueryFilter;

    wallet_user::Entity::find()
        .filter(wallet_user::Column::WalletId.eq(wallet_id))
        .one(connection)
        .await
        .unwrap()
        .expect("wallet_user should exist")
        .recovery_code
        .expect("wallet_user should have a recovery code")
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

pub fn wallet_provider_settings(db_url: Url, audit_db_url: Url) -> (WpSettings, ReqwestTrustAnchor) {
    let mut settings = WpSettings::new().expect("Could not read settings");
    settings.database.url = db_url;
    settings.audit_log.url = audit_db_url;

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
            tracing::error!("Could not start config_server: {error:?}");
            process::exit(1);
        }
    });

    let base_url = local_config_base_url(port);
    wait_for_server(
        remove_path(&base_url),
        Some(vec_nonempty![trust_anchor.into_certificate()]),
    )
    .await;
    port
}

pub async fn start_update_policy_server(settings: UpsSettings, trust_anchor: ReqwestTrustAnchor) -> u16 {
    let listener = TcpListener::bind("localhost:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async {
        if let Err(error) = update_policy_server::server::serve_with_listener(listener, settings).await {
            tracing::error!("Could not start update_policy_server: {error:?}");
            process::exit(1);
        }
    });

    let base_url = local_ups_base_url(port);
    wait_for_server(
        remove_path(&base_url),
        Some(vec_nonempty![trust_anchor.into_certificate()]),
    )
    .await;
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
            tracing::error!("Could not start wallet_provider: {error:?}");

            process::exit(1);
        }
    });

    let base_url = local_wp_base_url(port);
    wait_for_server(
        remove_path(&base_url),
        Some(vec_nonempty![trust_anchor.into_certificate()]),
    )
    .await;
    port
}

pub fn pid_issuer_settings(db_url: Url) -> PidIssuerSettings {
    let mut settings = PidIssuerSettings::new("pid_issuer.toml", "pid_issuer").expect("Could not read settings");

    settings.issuer_settings.server_settings.storage.url = db_url;

    settings.issuer_settings.server_settings.wallet_server.ip = IpAddr::from_str("127.0.0.1").unwrap();
    settings.issuer_settings.server_settings.wallet_server.port = 0;

    settings
}

pub fn issuance_server_settings(
    db_url: Url,
) -> (
    IssuanceServerSettings,
    Vec<IssuableDocument>,
    ReqwestTrustAnchor,
    TlsServerConfig,
) {
    let mut settings =
        IssuanceServerSettings::new("issuance_server.toml", "issuance_server").expect("Could not read settings");

    settings.issuer_settings.server_settings.storage.url = db_url;

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

pub fn verification_server_settings(db_url: Url) -> VerifierSettings {
    let mut settings =
        VerifierSettings::new("verification_server.toml", "verification_server").expect("Could not read settings");

    settings.server_settings.storage.url = db_url;

    settings.server_settings.wallet_server.ip = IpAddr::from_str("127.0.0.1").unwrap();
    settings.server_settings.wallet_server.port = 0;

    settings.server_settings.internal_server = ServerAuth::InternalEndpoint(Server {
        ip: IpAddr::from_str("127.0.0.1").unwrap(),
        port: 0,
    });

    settings
}

fn internal_url(settings: &VerifierSettings) -> BaseUrl {
    match settings.server_settings.internal_server {
        ServerAuth::ProtectedInternalEndpoint {
            server: Server { port, .. },
            ..
        }
        | ServerAuth::InternalEndpoint(Server { port, .. }) => local_http_base_url(port),
        ServerAuth::Authentication(_) => settings.public_url.clone(),
    }
}

async fn get_internal_listener(server_settings: &mut server_utils::settings::Settings) -> Option<TcpListener> {
    match &mut server_settings.internal_server {
        ServerAuth::Authentication(_) => None,
        ServerAuth::ProtectedInternalEndpoint { server, .. } | ServerAuth::InternalEndpoint(server) => {
            let listener = TcpListener::bind(("localhost", 0)).await.unwrap();
            server.port = listener.local_addr().unwrap().port();
            Some(listener)
        }
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
        axum_server::from_tcp_rustls(listener, tls_server_config.into_rustls_config().unwrap())
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

    let url = local_https_base_url(port);
    wait_for_server(url.clone(), Some(vec_nonempty![trust_anchor.into_certificate()])).await;
    url
}

pub async fn start_issuance_server(mut settings: IssuanceServerSettings, hsm: Option<Pkcs11Hsm>) -> IssuerUrl {
    let public_listener = TcpListener::bind("localhost:0").await.unwrap();
    let public_port = public_listener.local_addr().unwrap().port();
    let public_url = local_http_issuer_identifier(public_port);
    settings.issuer_settings.public_url = public_url.clone();

    let internal_listener = get_internal_listener(&mut settings.issuer_settings.server_settings).await;
    let internal_port = internal_listener.as_ref().unwrap().local_addr().unwrap().port();
    let internal_url = local_http_base_url(internal_port);

    let status_list_client = HttpStatusListClient::new(default_reqwest_client_builder()).unwrap();
    let revocation_verifier = settings.to_revocation_verifier(status_list_client);

    let serve_status_lists = settings.issuer_settings.status_lists.serve;

    let (issuer, _, store_connection, server_settings) =
        settings.issuer_settings.into_issuer(hsm.clone(), None).await.unwrap();

    let issuer = Arc::new(issuer);

    let disclosure_sessions = SessionStoreVariant::new(store_connection, (&server_settings.storage).into());

    let disclosure_router = settings
        .verifier_settings
        .into_disclosure_router(
            hsm,
            Arc::clone(&issuer),
            disclosure_sessions,
            revocation_verifier,
            &server_settings,
        )
        .await
        .unwrap();

    tokio::spawn(
        async move {
            if let Err(error) = issuance_server::server::serve_with_listeners(
                public_listener,
                internal_listener,
                issuer,
                server_settings,
                serve_status_lists,
                [],
                disclosure_router,
            )
            .await
            {
                tracing::error!("Could not start issuance_server: {error:?}");

                process::exit(1);
            }
        }
        .instrument(info_span!("service", name = "issuance_server")),
    );

    wait_for_server(public_url.as_base_url().clone(), None).await;
    IssuerUrl {
        internal: internal_url,
        public: public_url,
    }
}

pub async fn start_pid_issuer_server<B, O, F>(
    mut settings: PidIssuerSettings,
    hsm: Option<Pkcs11Hsm>,
    flow_builder: F,
) -> IssuerUrl
where
    B: BrpClient + Send + Sync + 'static,
    O: DigidClient + Send + Sync + 'static,
    F: FnOnce(&IssuerIdentifier) -> UpstreamOidcAuthorizationCodeFlow<B, O>,
{
    let public_listener = TcpListener::bind("localhost:0").await.unwrap();
    let public_port = public_listener.local_addr().unwrap().port();
    let public_url = local_http_issuer_identifier(public_port);
    settings.issuer_settings.public_url = public_url.clone();

    let internal_listener = get_internal_listener(&mut settings.issuer_settings.server_settings).await;
    let internal_port = internal_listener.as_ref().unwrap().local_addr().unwrap().port();
    let internal_url = local_http_base_url(internal_port);

    let serve_status_lists = settings.issuer_settings.status_lists.serve;

    let wia_config = WiaConfig {
        wia_trust_anchors: settings.wia_trust_anchors,
    };

    let wallet_redirect_uris = settings.wallet_redirect_uris;

    let flow = flow_builder(&public_url);

    let (issuer, _, _, server_settings) = settings
        .issuer_settings
        .into_authorizing_issuer(hsm, Some(wia_config), wallet_redirect_uris, |_| {
            Ok::<_, Infallible>(flow)
        })
        .await
        .unwrap();

    let authorizing_issuer = Arc::new(issuer);

    tokio::spawn(
        async move {
            if let Err(error) = pid_issuer::server::serve_with_listeners(
                public_listener,
                internal_listener,
                authorizing_issuer,
                server_settings,
                serve_status_lists,
                [],
            )
            .await
            {
                tracing::error!("Could not start pid_issuer: {error:?}");

                process::exit(1);
            }
        }
        .instrument(info_span!("service", name = "pid_issuer")),
    );

    wait_for_server(public_url.as_base_url().clone(), None).await;
    IssuerUrl {
        internal: internal_url,
        public: public_url,
    }
}

pub async fn start_verification_server(mut settings: VerifierSettings, hsm: Option<Pkcs11Hsm>) -> DisclosureUrls {
    let wallet_listener = TcpListener::bind("localhost:0").await.unwrap();
    let port = wallet_listener.local_addr().unwrap().port();

    let requester_listener = get_internal_listener(&mut settings.server_settings).await;

    let public_url = BaseUrl::from_str(format!("http://localhost:{port}/").as_str()).unwrap();
    let internal_url = internal_url(&settings);
    settings.public_url = public_url.clone();

    let storage_settings = &settings.server_settings.storage;
    let store_connection = StoreConnection::try_new(storage_settings.url.clone()).await.unwrap();

    let disclosure_sessions = Arc::new(SessionStoreVariant::new(
        store_connection.clone(),
        storage_settings.into(),
    ));

    let status_list_client = HttpStatusListClient::new(default_reqwest_client_builder()).unwrap();

    tokio::spawn(
        async move {
            if let Err(error) = verification_server::server::serve_with_listeners(
                wallet_listener,
                requester_listener,
                settings,
                hsm,
                disclosure_sessions,
                status_list_client,
            )
            .await
            {
                tracing::error!("Could not start verification_server: {error:?}");

                process::exit(1);
            }
        }
        .instrument(info_span!("service", name = "verification_server")),
    );

    wait_for_server(public_url.clone(), None).await;
    DisclosureUrls {
        verifier_url: public_url,
        verifier_internal_url: internal_url,
    }
}

pub async fn wait_for_server(base_url: BaseUrl, trust_anchors: Option<VecNonEmpty<Certificate>>) {
    let client = match trust_anchors {
        None => default_reqwest_client_builder(),
        Some(anchors) => tls_reqwest_client_builder(anchors),
    }
    .connect_timeout(Duration::from_secs(1))
    .build()
    .unwrap();

    time::timeout(Duration::from_secs(3), async {
        let mut interval = time::interval(Duration::from_millis(100));
        loop {
            interval.tick().await;
            match client
                .get(base_url.join("health"))
                .send()
                .await
                .and_then(|r| r.error_for_status())
            {
                Ok(_) => break,
                Err(e) => tracing::info!("Server not yet up: {e:?}"),
            }
        }
    })
    .await
    .unwrap_or_else(|e| panic!("Server not up: {base_url}: {e}"));
}

pub fn gba_hc_converter_settings() -> GbaSettings {
    // We cannot use a random port here, since the BRP proxy needs to connect to the converter on a set port.
    let mut settings = GbaSettings::new().expect("Could not read settings");
    settings.ip = IpAddr::from_str("127.0.0.1").unwrap();
    settings
}

pub async fn start_gba_hc_converter(settings: GbaSettings) {
    let base_url = local_http_base_url(settings.port);

    tokio::spawn(async {
        if let Err(error) = gba_hc_converter::app::serve_from_settings(settings).await {
            if let Some(io_error) = error.downcast_ref::<io::Error>()
                && io_error.kind() == io::ErrorKind::AddrInUse
            {
                tracing::warn!(
                    "TCP address/port for gba_hc_converter is already in use, assuming you started it yourself, \
                     continuing..."
                );
                return;
            }
            tracing::error!("Could not start gba_hc_converter: {error:?}");
            process::exit(1);
        }
    });

    wait_for_server(base_url, None).await;
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

/// Act as the user's browser for the in-process pid_issuer: follow the redirect chain starting at
/// `/authorize` (`/authorize` → `/digid/callback` → wallet redirect, since the mocked DigiD
/// client sends the user-agent straight back to the issuer's own callback) for as long as it
/// stays on the issuer's origin, returning the first off-origin redirect URL, which is the wallet-facing
/// redirect with the issuer-generated `code` and the wallet's original `state`.
pub async fn follow_authorization_redirects(authorization_url: Url) -> Url {
    let http_client = default_reqwest_client_builder()
        .redirect(Policy::none())
        .build()
        .unwrap();

    let issuer_origin = authorization_url.origin();
    let mut url = authorization_url;

    loop {
        let response = http_client.get(url).send().await.unwrap();
        let location: Url = response
            .headers()
            .get(header::LOCATION)
            .expect("response should be a redirect carrying a Location header")
            .to_str()
            .unwrap()
            .parse()
            .unwrap();

        if location.origin() != issuer_origin {
            return location;
        }
        url = location;
    }
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
    let redirect_uri = wallet
        .create_pid_issuance_auth_url(purpose)
        .await
        .expect("Could not create pid issuance auth url");

    let redirect_url = follow_authorization_redirects(redirect_uri).await;

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

pub async fn do_pin_recovery(mut wallet: WalletWithStorage, new_pin: String) -> WalletWithStorage {
    let redirect_uri = wallet
        .create_pin_recovery_redirect_uri()
        .await
        .expect("Could not create pin recovery redirect URI");

    let redirect_url = follow_authorization_redirects(redirect_uri).await;

    wallet
        .continue_pin_recovery(redirect_url)
        .await
        .expect("Could not continue pin recovery");
    wallet
        .complete_pin_recovery(new_pin)
        .await
        .expect("Could not complete pin recovery");
    wallet
}

pub async fn do_degree_issuance(
    wallet: &mut WalletWithStorage,
    pin: String,
    issuance_server_url: &IssuerIdentifier,
    client_ids: &DegreeClientIds,
    format: Format,
) -> Vec<AttestationPresentation> {
    let _proposal = wallet
        .start_disclosure(
            &universal_link(issuance_server_url.as_base_url(), client_ids, format),
            DisclosureUriSource::Link,
        )
        .await
        .unwrap();

    let attestation_previews = wallet
        .continue_disclosure_based_issuance(&[0], pin.clone())
        .await
        .unwrap();

    wallet.accept_issuance(pin).await.unwrap();

    attestation_previews
}

pub fn universal_link(issuance_server_url: &BaseUrl, client_ids: &DegreeClientIds, format: Format) -> Url {
    let params = serde_qs::to_string(&VerifierUrlParameters {
        session_type: SessionType::SameDevice,
        ephemeral_id_params: None,
    })
    .unwrap();

    let issuance_path = match format {
        Format::MsoMdoc => "/disclosure/university_mdoc/request_uri",
        Format::SdJwt => "/disclosure/university_sd_jwt/request_uri",
    };
    let mut issuance_server_url = issuance_server_url.join_base_url(issuance_path).into_inner();
    issuance_server_url.set_query(Some(&params));

    let query = serde_qs::to_string(&VpRequestUri {
        client_id: client_ids.for_format(format).clone(),
        object: VpRequestUriObject::AsReference {
            request_uri: issuance_server_url.try_into().unwrap(),
            request_uri_method: Some(VpRequestUriMethod::POST),
        },
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

pub fn create_pid_credential_offer(issuer_identifier: &IssuerIdentifier) -> Url {
    let credential_offer_uri = issuer_identifier
        .as_base_url()
        .join_base_url("/issuance/credential_offer")
        .into_inner();

    let mut pid_credential_offer = Url::parse("eu-eaa-offer://").unwrap();
    pid_credential_offer
        .query_pairs_mut()
        .append_pair("credential_offer_uri", credential_offer_uri.as_str());

    pid_credential_offer
}
