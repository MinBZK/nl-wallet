use std::sync::Arc;
use std::time::Duration;

use audit_log::model::DbAuditLog;
use audit_log::repository::PostgresAuditLogRepository;
use crypto::keys::EcdsaKey;
use crypto::kid::Kid;
use crypto::server_keys::KeyPair;
use crypto::server_keys::generate::Ca;
use crypto::trust_anchor::TrustAnchors;
use crypto::utils::random_string;
use crypto::x509::CertificateConfiguration;
use crypto::x509::DistinguishedName;
use db_test::DbName;
use db_test::DbSetup;
use hsm::keys::HsmEcdsaKey;
use hsm::model::TestHsm;
use hsm::service::Pkcs11Hsm;
use hsm::test::HsmSetup;
use http_utils::reqwest::test::get_test_trust_anchor;
use p256::pkcs8::EncodePublicKey;
use status_lists::config::StatusListConfig;
use status_lists::postgres::PostgresStatusListService;
use status_lists::publish::PublishDir;
use utils::generator::TimeGenerator;
use utils::num::NonZeroU31;
use utils::num::U31;
use utils::vec_nonempty;
use wallet_provider_domain::keys::KidPair;
use wallet_provider_persistence::database::ConnectionOptions;
use wallet_provider_persistence::database::Db;
use wallet_provider_persistence::repositories::Repositories;
use wallet_provider_persistence::test::db_from_setup;
use wallet_provider_service::account_server::UserState;
use wallet_provider_service::admin_portal::UuidV7Generator;
use wallet_provider_service::flags::WalletRepoFlags;
use wallet_provider_service::status_list::KeyStorageStatusList;
use wallet_provider_service::wia_issuer::WiaIssuer;
use wscd::payload::wia::WiaWalletInfo;

use crate::admin_portal::AdminPortalState;
use crate::settings::AdminPortalSettings;

/// Keep the concrete HTTP state's database, token directory, and published files alive together.
pub struct TestState {
    pub state: Arc<AdminPortalState>,
    pub repositories: Repositories,
    _db_setup: DbSetup,
    _hsm_setup: HsmSetup,
    _publish_dir: tempfile::TempDir,
}

impl std::ops::Deref for TestState {
    type Target = Arc<AdminPortalState>;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

type TestFlags = WalletRepoFlags<Repositories>;
type TestStatusList = PostgresStatusListService<HsmEcdsaKey, TestFlags>;
type TestUserState = UserState<Repositories, TestFlags, Pkcs11Hsm, HsmEcdsaKey, TestStatusList>;

pub async fn setup_state() -> TestState {
    let hsm_setup = HsmSetup::new();
    let db_setup = DbSetup::create_clean_only([DbName::WalletProvider, DbName::WalletProviderAuditLog]).await;
    let publish_dir = tempfile::tempdir().unwrap();

    let db = db_from_setup(&db_setup).await;
    let repositories = Repositories::from(db.clone());
    let flags = WalletRepoFlags::try_new(repositories.clone(), Duration::from_secs(300))
        .await
        .unwrap();

    let hsm = setup_hsm(&hsm_setup);
    let key_pair = generate_key_pair(&hsm).await;
    let wia_status_list = setup_wia_status_list(&db, &publish_dir, key_pair.clone(), flags.clone()).await;
    let user_state = setup_user_state(repositories.clone(), flags, hsm, key_pair, wia_status_list);

    let state = Arc::new(
        AdminPortalState::try_new(
            admin_portal_settings(),
            repositories.clone(),
            setup_audit_log(&db_setup).await,
            Arc::new(user_state),
        )
        .unwrap(),
    );

    TestState {
        state,
        repositories,
        _db_setup: db_setup,
        _hsm_setup: hsm_setup,
        _publish_dir: publish_dir,
    }
}

fn setup_hsm(hsm_setup: &HsmSetup) -> Pkcs11Hsm {
    // Read only the HSM connection settings; all test keys and certificates are generated here.
    let config = config::Config::builder()
        .set_default("hsm.max_sessions", 3)
        .unwrap()
        .set_default("hsm.max_session_lifetime_in_sec", 900)
        .unwrap()
        .add_source(config::File::from(std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/wallet_provider.toml"
        ))))
        .build()
        .unwrap();
    hsm_setup.pkcs11_hsm(config.get("hsm").unwrap()).unwrap()
}

/// Generate a fresh signing key in the HSM, with a certificate issued by a mock CA.
async fn generate_key_pair(hsm: &Pkcs11Hsm) -> KeyPair<HsmEcdsaKey> {
    let key_id = random_string(24);
    TestHsm::generate_signing_key_pair(hsm, &key_id).await.unwrap();
    let key = HsmEcdsaKey::new(key_id, hsm.clone());
    let public_key = key.verifying_key().await.unwrap().to_public_key_der().unwrap();

    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let certificate = ca
        .generate_certificate(
            public_key.as_bytes(),
            DistinguishedName::create_mock("Test wallet"),
            CertificateConfiguration::default(),
        )
        .unwrap();

    KeyPair::new(key, certificate).await.unwrap()
}

/// Create the WIA status list service and initialize its lists, publishing them into `publish_dir`.
async fn setup_wia_status_list(
    db: &Db,
    publish_dir: &tempfile::TempDir,
    key_pair: KeyPair<HsmEcdsaKey>,
    flags: TestFlags,
) -> TestStatusList {
    let config = StatusListConfig {
        list_size: NonZeroU31::try_new(100).unwrap(),
        create_threshold: U31::ZERO,
        expiry: Duration::from_secs(3600),
        refresh_threshold: Duration::from_secs(600),
        ttl: None,
        base_url: "https://example.com/".parse().unwrap(),
        context_path: "wia".to_string(),
        publish_dir: PublishDir::try_new(publish_dir.path().to_path_buf()).unwrap(),
        key_pair,
    };
    let status_list = PostgresStatusListService::try_new("wia", db.to_connection(), config, flags)
        .await
        .unwrap();

    futures::future::try_join_all(status_list.initialize_lists().await.unwrap())
        .await
        .unwrap();

    status_list
}

/// Assemble the user state, using `key_pair` for both WIA issuance and key attestations.
fn setup_user_state(
    repositories: Repositories,
    flags: TestFlags,
    hsm: Pkcs11Hsm,
    key_pair: KeyPair<HsmEcdsaKey>,
    wia_status_list: TestStatusList,
) -> TestUserState {
    let wallet_info = WiaWalletInfo {
        wallet_name: "Test wallet".to_string(),
        wallet_version: "test".to_string(),
        wallet_link: None,
        wallet_solution_certification_information: "test".to_string(),
    };
    let key_storage_status_list = KeyStorageStatusList::try_new(
        "https://example.com/key-storage".parse().unwrap(),
        Duration::from_secs(3600),
        Duration::from_secs(300),
        key_pair.clone(),
        flags.clone(),
    )
    .unwrap();

    UserState {
        repositories,
        flags,
        wallet_user_hsm: hsm,
        wia_issuer: WiaIssuer::new(key_pair.clone(), "wallet".to_string(), wallet_info),
        wia_status_tracking_validity: Duration::from_secs(86400),
        key_attestation_key_pair: key_pair,
        key_attestation_certification: "https://example.com/".parse().unwrap(),
        key_storage_status_tracking_validity: Duration::from_secs(86400),
        attestation_wrapping_kids: KidPair {
            current: Kid::try_from("test").unwrap(),
            previous: None,
        },
        pid_issuer_trust_anchors: TrustAnchors::empty(),
        wia_status_list,
        key_storage_status_list,
    }
}

async fn setup_audit_log(db_setup: &DbSetup) -> DbAuditLog<PostgresAuditLogRepository, UuidV7Generator> {
    let db_connection = Db::new_connection(
        db_setup.audit_log_url(),
        ConnectionOptions {
            acquire_timeout: Duration::from_secs(5),
            connect_timeout: Duration::from_secs(1),
            max_connections: 5,
        },
    )
    .await
    .unwrap();

    DbAuditLog {
        repository: PostgresAuditLogRepository { db_connection },
        time_generator: TimeGenerator,
        uuid_generator: UuidV7Generator,
    }
}

fn admin_portal_settings() -> AdminPortalSettings {
    AdminPortalSettings {
        keycloak_url: "https://keycloak.invalid".parse().unwrap(),
        keycloak_realm: "test-realm".to_string(),
        keycloak_client_id: "test-client".to_string(),
        public_url: "https://portal.example.org/".parse().unwrap(),
        keycloak_trust_anchors: vec_nonempty![get_test_trust_anchor()],
        session_ttl: Duration::from_mins(15),
        login_attempt_ttl: Duration::from_mins(10),
    }
}
