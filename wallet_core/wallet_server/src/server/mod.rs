cfg_if::cfg_if! {
    if #[cfg(all(feature = "disclosure", feature = "issuance"))] {
    pub mod wallet_server;
    } else if #[cfg(feature = "issuance")] {
    pub mod pid_issuer;
    } else if #[cfg(feature = "disclosure")] {
    pub mod wallet_server_verifier;
    }
}

use std::net::SocketAddr;

use anyhow::Result;
use axum::{routing::get, Router};
use tower_http::{trace::TraceLayer, validate_request::ValidateRequestHeaderLayer};
use tracing::debug;

use crate::{
    log_requests::log_request_response,
    settings::{Authentication, RequesterAuth, Settings},
};

#[cfg(feature = "disclosure")]
pub mod disclosure;

fn health_router() -> Router {
    Router::new().route("/health", get(|| async {}))
}

pub fn decorate_router(prefix: &str, router: Router, log_requests: bool) -> Router {
    let mut router = Router::new().nest(prefix, router).nest(prefix, health_router());

    if log_requests {
        router = router.layer(axum::middleware::from_fn(log_request_response));
    }

    router.layer(TraceLayer::new_for_http())
}

fn create_wallet_socket(settings: &Settings) -> SocketAddr {
    SocketAddr::new(settings.wallet_server.ip, settings.wallet_server.port)
}

fn secure_router(settings: &Settings, requester_router: Router) -> Router {
    match &settings.requester_server {
        RequesterAuth::Authentication(Authentication::ApiKey(api_key)) => {
            requester_router.layer(ValidateRequestHeaderLayer::bearer(api_key))
        }
        RequesterAuth::ProtectedInternalEndpoint {
            authentication: Authentication::ApiKey(api_key),
            ..
        } => requester_router.layer(ValidateRequestHeaderLayer::bearer(api_key)),
        RequesterAuth::InternalEndpoint(_) => requester_router,
    }
}

fn create_requester_socket(settings: &Settings) -> Option<SocketAddr> {
    match &settings.requester_server {
        RequesterAuth::Authentication(Authentication::ApiKey(_)) => None,
        RequesterAuth::ProtectedInternalEndpoint {
            authentication: Authentication::ApiKey(_),
            server,
        } => Some(SocketAddr::new(server.ip, server.port)),
        RequesterAuth::InternalEndpoint(server) => Some(SocketAddr::new(server.ip, server.port)),
    }
}

async fn listen(
    wallet_socket: SocketAddr,
    requester_socket: Option<SocketAddr>,
    wallet_router: Router,
    requester_router: Router,
) -> Result<()> {
    if let Some(requester_socket) = requester_socket {
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
    } else {
        debug!("listening for wallet and requester on {}", wallet_socket);
        axum::Server::bind(&wallet_socket)
            .serve(wallet_router.merge(requester_router).into_make_service())
            .await
            .expect("wallet server should be started")
    }

    Ok(())
}
