#[cfg(feature = "issuance")]
pub mod pid_issuer;
#[cfg(feature = "disclosure")]
pub mod verification_server;
#[cfg(all(feature = "disclosure", feature = "issuance"))]
pub mod wallet_server;

use std::{future::Future, net::SocketAddr};

use anyhow::Result;
use axum::{routing::get, Router};
use tower_http::{trace::TraceLayer, validate_request::ValidateRequestHeaderLayer};
use tracing::debug;

use crate::{
    log_requests::log_request_response,
    settings::{Authentication, RequesterAuth, Server, Settings},
};

fn health_router() -> Router {
    Router::new().route("/health", get(|| async {}))
}

pub fn decorate_router_with_health_log_and_tracing(prefix: &str, router: Router, log_requests: bool) -> Router {
    let mut router = router.nest(prefix, health_router());

    if log_requests {
        router = router.layer(axum::middleware::from_fn(log_request_response));
    }

    router.layer(TraceLayer::new_for_http())
}

pub fn decorate_router(prefix: &str, router: Router, log_requests: bool) -> Router {
    let router = Router::new().nest(prefix, router);

    decorate_router_with_health_log_and_tracing(prefix, router, log_requests)
}

/// Create Wallet socket from [settings].
fn create_wallet_socket(wallet_server: Server) -> SocketAddr {
    SocketAddr::new(wallet_server.ip, wallet_server.port)
}

/// Secure [requester_router] with an API key when required by [settings].
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

/// Create Requester socket when required by [settings].
fn create_requester_socket(requester_server: &RequesterAuth) -> Option<SocketAddr> {
    match requester_server {
        RequesterAuth::Authentication(_) => None,
        RequesterAuth::ProtectedInternalEndpoint { server, .. } | RequesterAuth::InternalEndpoint(server) => {
            Some(SocketAddr::new(server.ip, server.port))
        }
    }
}

async fn listen(
    wallet_server: Server,
    requester_server: Option<RequesterAuth>,
    wallet_router: Router,
    requester_router: Option<Router>,
) -> Result<()> {
    let wallet_socket = create_wallet_socket(wallet_server);
    let requester_socket = requester_server.as_ref().and_then(create_requester_socket);

    let requester_router = requester_router.and_then(|router| {
        requester_server
            .as_ref()
            .map(|server| secure_requester_router(server, router))
    });

    match (requester_socket, requester_router) {
        (Some(requester_socket), Some(requester_router)) => {
            debug!("listening for requester on {}", requester_socket);
            let requester_server = tokio::spawn(async move {
                axum::Server::bind(&requester_socket)
                    .serve(requester_router.into_make_service())
                    .await
                    .expect("requester server should be started")
            });

            debug!("listening for wallet on {}", wallet_socket);
            let wallet_server = tokio::spawn(async move {
                axum::Server::bind(&wallet_socket)
                    .serve(wallet_router.into_make_service())
                    .await
                    .expect("wallet server should be started")
            });

            tokio::try_join!(requester_server, wallet_server)?;
        }
        (None, Some(requester_router)) => {
            debug!("listening for wallet and requester on {}", wallet_socket);
            axum::Server::bind(&wallet_socket)
                .serve(wallet_router.merge(requester_router).into_make_service())
                .await
                .expect("wallet server should be started");
        }
        (_, None) => {
            debug!("listening for wallet on {}", wallet_socket);
            axum::Server::bind(&wallet_socket)
                .serve(wallet_router.into_make_service())
                .await
                .expect("wallet server should be started");
        }
    }

    Ok(())
}

/// Setup tracing, read settings and setup Sentry if configured, then run `app` on the tokio runtime.
pub fn wallet_server_main<Fut: Future<Output = Result<()>>>(
    config_file: &str,
    env_prefix: &str,
    app: impl FnOnce(Settings) -> Fut,
) -> Result<()> {
    // Initialize tracing.
    tracing_subscriber::fmt::init();

    let settings = Settings::new_custom(config_file, env_prefix)?;

    // Retain [`ClientInitGuard`]
    let _guard = settings
        .sentry
        .as_ref()
        .map(|sentry| sentry.init(sentry::release_name!()));

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async { app(settings).await })
}
