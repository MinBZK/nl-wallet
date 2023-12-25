use std::net::SocketAddr;

use anyhow::Result;
use axum::{routing::get, Router};
use tokio;
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

pub async fn serve_disclosure(settings: &Settings, sessions: SessionStores) -> Result<()> {
    let wallet_socket = SocketAddr::new(settings.wallet_server.ip, settings.wallet_server.port);
    let requester_socket = SocketAddr::new(settings.requester_server.ip, settings.requester_server.port);
    let (wallet_disclosure_router, requester_router) = verifier::create_routers(settings.clone(), sessions.disclosure)?;

    listen(
        wallet_socket,
        requester_socket,
        Router::new()
            .nest("/disclosure/", wallet_disclosure_router)
            .nest("/disclosure/", health_router()),
        Router::new()
            .nest("/disclosure/sessions", requester_router)
            .nest("/disclosure/sessions", health_router()),
    )
    .await?;

    Ok(())
}

#[cfg(feature = "issuance")]
pub async fn serve_full<A>(settings: &Settings, sessions: SessionStores, attr_service: A) -> Result<()>
where
    A: AttributeService,
{
    let wallet_socket = SocketAddr::new(settings.wallet_server.ip, settings.wallet_server.port);
    let requester_socket = SocketAddr::new(settings.requester_server.ip, settings.requester_server.port);
    let (wallet_disclosure_router, requester_router) = verifier::create_routers(settings.clone(), sessions.disclosure)?;

    let wallet_issuance_router =
        crate::issuer::create_issuance_router(settings.clone(), sessions.issuance, attr_service).await?;

    listen(
        wallet_socket,
        requester_socket,
        Router::new()
            .nest("/issuance/", wallet_issuance_router)
            .nest("/issuance/", health_router())
            .nest("/disclosure/", wallet_disclosure_router)
            .nest("/disclosure/", health_router()),
        Router::new()
            .nest("/disclosure/sessions", requester_router)
            .nest("/disclosure/sessions", health_router()),
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
    let server = tokio::spawn(async move {
        axum::Server::bind(&requester_socket)
            .serve(requester_router.into_make_service())
            .await
    });

    debug!("listening for wallet on {}", wallet_socket);
    axum::Server::bind(&wallet_socket)
        .serve(wallet_router.into_make_service())
        .await?;

    server.await??;

    Ok(())
}
