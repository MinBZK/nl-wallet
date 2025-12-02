use std::future::Future;
use std::io;

use anyhow::Result;
use axum::Router;
use axum::routing::get;
use http::HeaderValue;
use http::header;
use tokio::net::TcpListener;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;
use tower_http::validate_request::ValidateRequestHeaderLayer;
use tracing::error;
use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use utils::built_info::version_string;

use crate::log_requests::log_request_response;
use crate::settings::Authentication;
use crate::settings::Server;
use crate::settings::ServerAuth;
use crate::settings::ServerSettings;

fn health_router() -> Router {
    Router::new().route("/health", get(|| async {}))
}

pub fn add_cache_layer(router: Router) -> Router {
    router.layer(SetResponseHeaderLayer::overriding(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store"),
    ))
}

pub fn decorate_router(mut router: Router, log_requests: bool) -> Router {
    if log_requests {
        router = router.layer(axum::middleware::from_fn(log_request_response));
    }

    router.layer(TraceLayer::new_for_http()).merge(health_router())
}

/// Create Wallet listener from [settings].
pub async fn create_wallet_listener(wallet_server: &Server) -> Result<TcpListener, io::Error> {
    TcpListener::bind((wallet_server.ip, wallet_server.port)).await
}

/// Sanity check to see if [internal_listener] is set conform [settings].
pub fn check_internal_listener_with_settings(internal_listener: &Option<TcpListener>, internal_server: &ServerAuth) {
    match internal_server {
        ServerAuth::Authentication(_) => {
            assert!(
                internal_listener.is_none(),
                "no internal listener should be provided for authentication only"
            );
        }
        ServerAuth::ProtectedInternalEndpoint { .. } | ServerAuth::InternalEndpoint(_) => {
            assert!(
                internal_listener.is_some(),
                "an internal listener should be provided for internal endpoint"
            );
        }
    }
}

/// Create internal listener when required by [settings].
pub async fn create_internal_listener(internal_server: &ServerAuth) -> Result<Option<TcpListener>, io::Error> {
    match internal_server {
        ServerAuth::Authentication(_) => None,
        ServerAuth::ProtectedInternalEndpoint { server, .. } | ServerAuth::InternalEndpoint(server) => {
            TcpListener::bind((server.ip, server.port)).await.into()
        }
    }
    .transpose()
}

/// Secure [internal_router] with an API key when required by [settings].
pub fn secure_internal_router(internal_server: &ServerAuth, internal_router: Router) -> Router {
    match internal_server {
        ServerAuth::Authentication(Authentication::ApiKey(api_key))
        | ServerAuth::ProtectedInternalEndpoint {
            authentication: Authentication::ApiKey(api_key),
            ..
        } => internal_router.layer(ValidateRequestHeaderLayer::bearer(api_key)),
        ServerAuth::InternalEndpoint(_) => internal_router,
    }
}

pub async fn listen(
    wallet_listener: TcpListener,
    internal_listener: Option<TcpListener>,
    mut wallet_router: Router,
    mut internal_router: Router,
    log_requests: bool,
) -> Result<()> {
    info!("{}", version_string());

    match internal_listener {
        Some(internal_listener) => {
            wallet_router = decorate_router(wallet_router, log_requests);
            internal_router = decorate_router(internal_router, log_requests);

            info!("listening for internal on {}", internal_listener.local_addr()?);
            let internal_server = tokio::spawn(async move {
                axum::serve(internal_listener, internal_router)
                    .await
                    .expect("internal server should be started");
            });

            info!("listening for wallet on {}", wallet_listener.local_addr()?);
            let wallet_server = tokio::spawn(async move {
                axum::serve(wallet_listener, wallet_router)
                    .await
                    .expect("wallet server should be started");
            });

            tokio::try_join!(internal_server, wallet_server)?;
        }
        None => {
            wallet_router = decorate_router(wallet_router.merge(internal_router), log_requests);
            info!(
                "listening for wallet and requester on {}",
                wallet_listener.local_addr()?
            );
            axum::serve(wallet_listener, wallet_router)
                .await
                .expect("wallet server should be started");
        }
    }

    Ok(())
}

/// Setup tracing and read settings, then run `app`.
pub async fn wallet_server_main<S: ServerSettings, Fut: Future<Output = Result<()>>>(
    config_file: &str,
    env_prefix: &str,
    app: impl FnOnce(S) -> Fut,
) -> Result<()> {
    let settings = S::new(config_file, env_prefix)?;

    // Initialize tracing.
    let builder = tracing_subscriber::fmt().with_env_filter(
        EnvFilter::builder()
            .with_default_directive(LevelFilter::INFO.into())
            .from_env_lossy(),
    );
    if settings.server_settings().structured_logging {
        builder.json().init();
    } else {
        builder.init();
    }

    // Verify the settings here, now that we've setup tracing.
    if let Err(error) = settings.validate() {
        error!("invalid configuration: {error}");
        return Err(error.into());
    }

    app(settings).await
}
