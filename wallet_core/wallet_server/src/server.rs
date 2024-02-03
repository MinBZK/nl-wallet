use std::net::SocketAddr;

use anyhow::Result;
use axum::{routing::get, Router};
use tracing::debug;

#[cfg(feature = "issuance")]
use openid4vc::issuer::AttributeService;

use crate::{
    settings::Settings,
    store::SessionStores,
    verifier::{self},
};

fn health_router() -> Router {
    Router::new().route("/health", get(|| async {}))
}

fn decorate_router(prefix: &str, router: Router) -> Router {
    let router = Router::new().nest(prefix, router).nest(prefix, health_router());

    #[cfg(feature = "log_requests")]
    let router = router.layer(axum::middleware::from_fn(crate::log_requests::log_request_response));

    #[allow(clippy::let_and_return)] // See https://github.com/rust-lang/rust-clippy/issues/9150
    router
}

pub async fn serve_disclosure(settings: &Settings, sessions: SessionStores) -> Result<()> {
    let wallet_socket = SocketAddr::new(settings.wallet_server.ip, settings.wallet_server.port);
    let requester_socket = SocketAddr::new(settings.requester_server.ip, settings.requester_server.port);
    let (wallet_disclosure_router, requester_router) = verifier::create_routers(settings.clone(), sessions.disclosure)?;

    listen(
        wallet_socket,
        requester_socket,
        decorate_router("/disclosure/", wallet_disclosure_router),
        decorate_router("/disclosure/sessions", requester_router),
    )
    .await?;

    Ok(())
}

#[cfg(feature = "issuance")]
pub async fn serve_full<A>(settings: &Settings, sessions: SessionStores, attr_service: A) -> Result<()>
where
    A: AttributeService + Send + Sync + 'static,
{
    let wallet_socket = SocketAddr::new(settings.wallet_server.ip, settings.wallet_server.port);
    let requester_socket = SocketAddr::new(settings.requester_server.ip, settings.requester_server.port);
    let (wallet_disclosure_router, requester_router) = verifier::create_routers(settings.clone(), sessions.disclosure)?;

    let wallet_issuance_router =
        crate::issuer::create_issuance_router(settings.clone(), sessions.issuance, attr_service).await?;

    listen(
        wallet_socket,
        requester_socket,
        decorate_router("/issuance/", wallet_issuance_router)
            .merge(decorate_router("/disclosure/", wallet_disclosure_router)),
        decorate_router("/disclosure/sessions", requester_router),
    )
    .await?;

    Ok(())
}

async fn listen(
    wallet_socket: SocketAddr,
    requester_socket: SocketAddr,
    wallet_router: Router,
    requester_router: Router,
) -> anyhow::Result<()> {
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

    Ok(())
}
