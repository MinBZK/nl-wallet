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
use tracing::error;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use openid4vc_server::log_requests::log_request_response;

use crate::settings::Server;
use crate::settings::ServerSettings;

fn health_router() -> Router {
    Router::new().route("/health", get(|| async {}))
}

pub fn decorate_router(mut router: Router, log_requests: bool) -> Router {
    router = router.layer(SetResponseHeaderLayer::overriding(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store"),
    ));

    if log_requests {
        router = router.layer(axum::middleware::from_fn(log_request_response));
    }

    router.layer(TraceLayer::new_for_http()).merge(health_router())
}

/// Create Wallet listener from [settings].
pub async fn create_wallet_listener(wallet_server: Server) -> Result<TcpListener, io::Error> {
    TcpListener::bind((wallet_server.ip, wallet_server.port)).await
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
