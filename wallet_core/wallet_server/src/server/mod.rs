#[cfg(feature = "issuance")]
pub mod pid_issuer;

cfg_if::cfg_if! {
    if #[cfg(feature = "disclosure")] {
        pub mod verification_server;

        use tower_http::validate_request::ValidateRequestHeaderLayer;

        use crate::settings::{Authentication, RequesterAuth};
    }
}

#[cfg(all(feature = "disclosure", feature = "issuance"))]
pub mod wallet_server;

use std::future::Future;
use std::io;

use anyhow::Result;
use axum::routing::get;
use axum::Router;
use http::header;
use http::HeaderValue;
use tokio::net::TcpListener;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;
use tracing::debug;
use tracing::error;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use crate::log_requests::log_request_response;
use crate::settings::Server;
use crate::settings::Settings;

fn health_router() -> Router {
    Router::new().route("/health", get(|| async {}))
}

pub fn decorate_router(mut router: Router, log_requests: bool) -> Router {
    router = router.merge(health_router());

    router = router.layer(SetResponseHeaderLayer::overriding(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store"),
    ));

    if log_requests {
        router = router.layer(axum::middleware::from_fn(log_request_response));
    }

    router.layer(TraceLayer::new_for_http())
}

/// Create Wallet listener from [settings].
async fn create_wallet_listener(wallet_server: Server) -> Result<TcpListener, io::Error> {
    TcpListener::bind((wallet_server.ip, wallet_server.port)).await
}

/// Secure [requester_router] with an API key when required by [settings].
#[cfg(feature = "disclosure")]
fn secure_requester_router(requester_server: &RequesterAuth, requester_router: Router) -> Router {
    match requester_server {
        RequesterAuth::Authentication(Authentication::ApiKey(api_key))
        | RequesterAuth::ProtectedInternalEndpoint {
            authentication: Authentication::ApiKey(api_key),
            ..
        } => requester_router.layer(ValidateRequestHeaderLayer::bearer(api_key)),
        RequesterAuth::InternalEndpoint(_) => requester_router,
    }
}

/// Create Requester listener when required by [settings].
#[cfg(feature = "disclosure")]
async fn create_requester_listener(requester_server: &RequesterAuth) -> Result<Option<TcpListener>, io::Error> {
    match requester_server {
        RequesterAuth::Authentication(_) => None,
        RequesterAuth::ProtectedInternalEndpoint { server, .. } | RequesterAuth::InternalEndpoint(server) => {
            TcpListener::bind((server.ip, server.port)).await.into()
        }
    }
    .transpose()
}

#[cfg(feature = "disclosure")]
async fn listen(
    wallet_server: Server,
    requester_server: RequesterAuth,
    mut wallet_router: Router,
    mut requester_router: Router,
    log_requests: bool,
) -> Result<()> {
    let wallet_listener = create_wallet_listener(wallet_server).await?;
    let requester_listener = create_requester_listener(&requester_server).await?;

    requester_router = secure_requester_router(&requester_server, requester_router);

    match requester_listener {
        Some(requester_listener) => {
            wallet_router = decorate_router(wallet_router, log_requests);
            requester_router = decorate_router(requester_router, log_requests);

            debug!(
                "listening for requester on {}",
                requester_listener.local_addr().unwrap()
            );
            let requester_server = tokio::spawn(async move {
                axum::serve(requester_listener, requester_router)
                    .await
                    .expect("requester server should be started");
            });

            debug!("listening for wallet on {}", wallet_listener.local_addr().unwrap());
            let wallet_server = tokio::spawn(async move {
                axum::serve(wallet_listener, wallet_router)
                    .await
                    .expect("wallet server should be started");
            });

            tokio::try_join!(requester_server, wallet_server)?;
        }
        None => {
            wallet_router = decorate_router(wallet_router.merge(requester_router), log_requests);
            debug!(
                "listening for wallet and requester on {}",
                wallet_listener.local_addr().unwrap()
            );
            axum::serve(wallet_listener, wallet_router)
                .await
                .expect("wallet server should be started");
        }
    }

    Ok(())
}

#[cfg(feature = "issuance")]
async fn listen_wallet_only(wallet_server: Server, mut wallet_router: Router, log_requests: bool) -> Result<()> {
    wallet_router = decorate_router(wallet_router, log_requests);

    let wallet_listener = create_wallet_listener(wallet_server).await?;

    debug!("listening for wallet on {}", wallet_listener.local_addr().unwrap());
    axum::serve(wallet_listener, wallet_router)
        .await
        .expect("wallet server should be started");

    Ok(())
}

/// Setup tracing and read settings, then run `app`.
pub async fn wallet_server_main<Fut: Future<Output = Result<()>>>(
    config_file: &str,
    env_prefix: &str,
    app: impl FnOnce(Settings) -> Fut,
) -> Result<()> {
    let settings = Settings::new_custom(config_file, env_prefix)?;

    // Initialize tracing.
    let builder = tracing_subscriber::fmt().with_env_filter(
        EnvFilter::builder()
            .with_default_directive(LevelFilter::INFO.into())
            .from_env_lossy(),
    );
    if settings.structured_logging {
        builder.json().init();
    } else {
        builder.init();
    }

    // Verify the settings here, now that we've setup tracing.
    if let Err(error) = settings.verify_key_pairs() {
        error!("invalid certificate configuration: {error}");
        return Err(error.into());
    }

    app(settings).await
}
