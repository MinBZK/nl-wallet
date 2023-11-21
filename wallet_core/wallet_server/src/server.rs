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
    let socket = SocketAddr::new(settings.wallet_server.ip, settings.wallet_server.port);

    let (wallet_router, requester_router) = create_routers(settings.clone(), sessions)?;

    // if an application server is configured, we need two webservers
    if let Some(server) = &settings.requester_server {
        let app_socket = SocketAddr::new(server.ip, server.port);

        debug!("listening for requester on {}", app_socket);
        let server = tokio::spawn(async move {
            axum::Server::bind(&app_socket)
                .serve(
                    Router::new()
                        .nest("/sessions", requester_router)
                        .nest("/sessions", health_router())
                        .into_make_service(),
                )
                .await
        });

        debug!("listening for wallet on {}", socket);
        axum::Server::bind(&socket)
            .serve(
                Router::new()
                    .nest("/", wallet_router)
                    .nest("/", health_router())
                    .into_make_service(),
            )
            .await?;

        server.await??;
    } else {
        let router = Router::new()
            .nest("/", wallet_router)
            .nest("/sessions", requester_router)
            .nest("/", health_router());

        debug!("listening on {}", socket);
        axum::Server::bind(&socket).serve(router.into_make_service()).await?;
    }

    Ok(())
}
