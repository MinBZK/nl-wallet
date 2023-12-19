use std::net::SocketAddr;

use anyhow::Result;
use axum::{routing::get, Router};
use tokio;
use tracing::debug;

use nl_wallet_mdoc::{
    server_state::{SessionState, SessionStore},
    verifier::DisclosureData,
};

use crate::{settings::Settings, verifier::create_routers};

fn health_router() -> Router {
    Router::new().route("/health", get(|| async {}))
}

pub async fn serve<S>(settings: &Settings, sessions: S) -> Result<()>
where
    S: SessionStore<Data = SessionState<DisclosureData>> + Send + Sync + 'static,
{
    let wallet_socket = SocketAddr::new(settings.wallet_server.ip, settings.wallet_server.port);
    let requestor_socket = SocketAddr::new(settings.requester_server.ip, settings.requester_server.port);

    let (wallet_router, requester_router) = create_routers(settings.clone(), sessions)?;

    debug!("listening for requester on {}", requestor_socket);
    let server = tokio::spawn(async move {
        axum::Server::bind(&requestor_socket)
            .serve(
                Router::new()
                    .nest("/sessions", requester_router)
                    .nest("/sessions", health_router())
                    .into_make_service(),
            )
            .await
    });

    debug!("listening for wallet on {}", wallet_socket);
    axum::Server::bind(&wallet_socket)
        .serve(
            Router::new()
                .nest("/", wallet_router)
                .nest("/", health_router())
                .into_make_service(),
        )
        .await?;

    server.await??;

    Ok(())
}
