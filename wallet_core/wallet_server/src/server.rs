use std::net::SocketAddr;

use anyhow::Result;
use axum::{routing::get, Router};
use openid4vc::issuer::AttributeService;
use tokio;
use tracing::debug;

use nl_wallet_mdoc::{
    server_state::{SessionState, SessionStore},
    verifier::DisclosureData,
};

use crate::{issuer::create_issuance_router, settings::Settings, verifier::create_verifier_routers};

fn health_router() -> Router {
    Router::new().route("/health", get(|| async {}))
}

pub async fn serve<A, S>(settings: &Settings, sessions: S, attr_service: A) -> Result<()>
where
    S: SessionStore<Data = SessionState<DisclosureData>> + Send + Sync + 'static,
    A: AttributeService,
{
    let wallet_socket = SocketAddr::new(settings.wallet_server.ip, settings.wallet_server.port);
    let requestor_socket = SocketAddr::new(settings.requester_server.ip, settings.requester_server.port);

    let (wallet_router, requester_router) = create_verifier_routers(settings.clone(), sessions)?;

    let issuance_router = create_issuance_router(settings.clone(), attr_service).await?;

    debug!("listening for requester on {}", requestor_socket);
    let server = tokio::spawn(async move {
        axum::Server::bind(&requestor_socket)
            .serve(
                Router::new()
                    .nest("/disclosure/sessions", requester_router)
                    .nest("/disclosure/sessions", health_router())
                    .into_make_service(),
            )
            .await
    });

    debug!("listening for wallet on {}", wallet_socket);
    axum::Server::bind(&wallet_socket)
        .serve(
            Router::new()
                .nest("/issuance/", issuance_router)
                .nest("/disclosure/", wallet_router)
                .nest("/disclosure/", health_router())
                .into_make_service(),
        )
        .await?;

    server.await??;

    Ok(())
}
