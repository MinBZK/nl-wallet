use std::assert_matches;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;

use axum::Router;
use axum::extract::State;
use axum::routing::get;
use chrono::TimeDelta;
use chrono::Utc;
use crypto::PublicKey;
use http_utils::client::TlsPinningConfig;
use http_utils::health::create_health_router;
use http_utils::urls::BaseUrl;
use tests_integration::common::*;
use tokio::net::TcpListener;
use tokio::sync::Notify;
use tokio::time;
use utils::vec_nonempty;
use wallet::errors::ConfigurationError;
use wallet::test::CONFIG_EXPIRY_LEEWAY;
use wallet::test::HttpConfigurationRepository;
use wallet::test::ObservableConfigExpiry;
use wallet::test::Repository;
use wallet::test::UpdateableRepository;
use wallet::test::UpdatingConfigurationRepository;
use wallet::test::default_config_server_config;
use wallet::test::default_wallet_config;
use wallet_configuration::config_server_config::ConfigServerConfiguration;
use wallet_configuration::wallet_config::WalletConfiguration;

/// Serves a wallet configuration JWT, which can be replaced while the server is running. This makes it possible to
/// observe how a wallet responds to its configuration expiring and to a fresh one becoming available afterwards,
/// without having to restart the server.
struct SwappableConfigServer {
    jwt: Arc<RwLock<String>>,
}

impl SwappableConfigServer {
    async fn start(config: &WalletConfiguration) -> (Self, ConfigServerConfiguration) {
        let (settings, trust_anchor) = static_server_settings();

        let jwt = Arc::new(RwLock::new(config_jwt(config).await.to_string()));
        let listener = TcpListener::bind("localhost:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let app = Router::new().merge(create_health_router([])).nest(
            "/config/v1",
            Router::new()
                .route("/wallet-config", get(serve_configuration))
                .with_state(Arc::clone(&jwt)),
        );

        let tls_config = settings.tls_config.into_rustls_config().unwrap();
        tokio::spawn(async move {
            axum_server::from_tcp_rustls(listener.into_std().unwrap(), tls_config)
                .expect("TCP listener should not be in blocking mode")
                .serve(app.into_make_service())
                .await
                .unwrap();
        });

        let base_url = local_config_base_url(port);
        let root_url: BaseUrl = format!("https://localhost:{port}/")
            .parse()
            .expect("hardcoded values should always parse successfully");

        wait_for_server(root_url, Some(vec_nonempty![trust_anchor.clone().into_certificate()])).await;

        let config_server_config = ConfigServerConfiguration {
            http_config: TlsPinningConfig::try_new(base_url, vec_nonempty![trust_anchor]).unwrap(),
            ..default_config_server_config()
        };

        (Self { jwt }, config_server_config)
    }

    /// Replaces the configuration served from this point onwards.
    async fn serve(&self, config: &WalletConfiguration) {
        let jwt = config_jwt(config).await.to_string();
        *self.jwt.write().unwrap() = jwt;
    }
}

/// Note that no `ETag` is served, so that every request results in the configuration being sent and verified again,
/// instead of the wallet being told that nothing changed.
async fn serve_configuration(State(jwt): State<Arc<RwLock<String>>>) -> String {
    jwt.read().unwrap().clone()
}

/// A duration far enough in the past for a configuration to be expired, taking into account that the leeway applied
/// to `exp` means that a configuration is still accepted for a while after the moment it expires.
fn expired_beyond_leeway() -> TimeDelta {
    -TimeDelta::from_std(CONFIG_EXPIRY_LEEWAY).unwrap() - TimeDelta::minutes(1)
}

/// A copy of the embedded configuration, expiring after the provided duration and carrying the provided version.
fn wallet_config(expires_in: TimeDelta, version: u64) -> WalletConfiguration {
    WalletConfiguration {
        expires: (Utc::now() + expires_in).into(),
        version,
        ..default_wallet_config()
    }
}

/// A configuration that has already expired should be rejected outright, rather than being adopted and only noticed
/// later on.
#[tokio::test]
async fn test_expired_configuration_is_rejected() {
    let embedded_config = wallet_config(TimeDelta::hours(1), 1);
    let (_server, config_server_config) =
        SwappableConfigServer::start(&wallet_config(expired_beyond_leeway(), 2)).await;

    let storage_dir = tempfile::tempdir().unwrap();
    let repository = HttpConfigurationRepository::new(
        PublicKey::from(*config_server_config.signing_public_key.as_inner()).into(),
        storage_dir.path().to_path_buf(),
        embedded_config,
    )
    .await
    .unwrap();

    let error = repository
        .fetch(&config_server_config.http_config)
        .await
        .expect_err("fetching an expired wallet configuration should fail");

    assert_matches!(error, ConfigurationError::JwtVerify(_));

    // The expired configuration should not have replaced the one the wallet already had.
    assert_eq!(repository.get().version, 1);
}

/// Once the wallet holds an expired configuration and cannot replace it, that should be reported so the user
/// interface can block, and reporting should stop as soon as a fresh configuration has been received.
#[tokio::test]
async fn test_expiry_is_reported_and_cleared() {
    // Both the configuration the wallet starts out with and the one on offer are expired.
    let (server, config_server_config) = SwappableConfigServer::start(&wallet_config(expired_beyond_leeway(), 2)).await;

    let storage_dir = tempfile::tempdir().unwrap();
    let repository = UpdatingConfigurationRepository::init(
        storage_dir.path().to_path_buf(),
        config_server_config,
        wallet_config(expired_beyond_leeway(), 1),
    )
    .await
    .unwrap();

    let expired = Arc::new(RwLock::new(Vec::new()));
    let notifier = Arc::new(Notify::new());

    {
        let expired = Arc::clone(&expired);
        let notifier = Arc::clone(&notifier);

        repository.register_config_expiry_callback(Box::new(move |is_expired| {
            expired.write().unwrap().push(is_expired);
            notifier.notify_one();
        }));
    }

    // Since the server has nothing valid to offer, the wallet should report that it is stuck with an expired
    // configuration.
    time::timeout(Duration::from_secs(30), async {
        while !repository.config_expired() {
            notifier.notified().await;
        }
    })
    .await
    .expect("expiry of the wallet configuration should have been reported");

    // Serving a fresh configuration should clear the expiry again. Note that this takes a while, as the wallet backs
    // off between attempts to replace an expired configuration.
    server.serve(&wallet_config(TimeDelta::hours(1), 3)).await;

    time::timeout(Duration::from_secs(60), async {
        while repository.config_expired() {
            notifier.notified().await;
        }
    })
    .await
    .expect("a fresh wallet configuration should have cleared the expiry");

    assert_eq!(repository.get().version, 3);
    {
        let reported = expired.read().unwrap();
        assert!(reported.contains(&true) && reported.contains(&false));
    }
}
