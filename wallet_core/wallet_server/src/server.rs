use std::net::SocketAddr;

use anyhow::Result;
use axum::{routing::get, Router};
use nl_wallet_mdoc::{server_state::SessionStore, verifier::DisclosureData};
use tower_http::{trace::TraceLayer, validate_request::ValidateRequestHeaderLayer};
use tracing::debug;

#[cfg(feature = "issuance")]
use openid4vc::issuer::AttributeService;

use crate::{
    settings::{Authentication, RequesterAuth, Settings},
    verifier,
};

fn health_router() -> Router {
    Router::new().route("/health", get(|| async {}))
}

fn decorate_router(prefix: &str, router: Router) -> Router {
    let router = Router::new().nest(prefix, router).nest(prefix, health_router());

    #[cfg(feature = "log_requests")]
    let router = router.layer(axum::middleware::from_fn(crate::log_requests::log_request_response));

    router.layer(TraceLayer::new_for_http())
}

fn setup_disclosure<S>(
    settings: Settings,
    disclosure_sessions: S,
) -> Result<(SocketAddr, Option<SocketAddr>, Router, Router)>
where
    S: SessionStore<DisclosureData> + Send + Sync + 'static,
{
    let wallet_socket = SocketAddr::new(settings.wallet_server.ip, settings.wallet_server.port);
    let (wallet_disclosure_router, mut requester_router) =
        verifier::create_routers(settings.clone(), disclosure_sessions)?;

    let requester_socket = match settings.requester_server {
        RequesterAuth::Authentication(Authentication::ApiKey(api_key)) => {
            requester_router = requester_router.layer(ValidateRequestHeaderLayer::bearer(&api_key));
            None
        }
        RequesterAuth::ProtectedInternalEndpoint {
            authentication: Authentication::ApiKey(api_key),
            server,
        } => {
            requester_router = requester_router.layer(ValidateRequestHeaderLayer::bearer(&api_key));
            Some(SocketAddr::new(server.ip, server.port))
        }
        RequesterAuth::InternalEndpoint(server) => Some(SocketAddr::new(server.ip, server.port)),
    };

    Ok((
        wallet_socket,
        requester_socket,
        wallet_disclosure_router,
        requester_router,
    ))
}

pub async fn serve_disclosure<S>(settings: Settings, disclosure_sessions: S) -> Result<()>
where
    S: SessionStore<DisclosureData> + Send + Sync + 'static,
{
    let (wallet_socket, requester_socket, wallet_disclosure_router, requester_router) =
        setup_disclosure(settings, disclosure_sessions)?;

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
pub async fn serve_full<A, DS, IS>(
    attr_service: A,
    settings: Settings,
    disclosure_sessions: DS,
    issuance_sessions: IS,
) -> Result<()>
where
    A: AttributeService + Send + Sync + 'static,
    DS: SessionStore<DisclosureData> + Send + Sync + 'static,
    IS: SessionStore<openid4vc::issuer::IssuanceData> + Send + Sync + 'static,
{
    let (wallet_socket, requester_socket, wallet_disclosure_router, requester_router) =
        setup_disclosure(settings.clone(), disclosure_sessions)?;

    let wallet_issuance_router =
        crate::issuer::create_issuance_router(settings, issuance_sessions, attr_service).await?;

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
    requester_socket: Option<SocketAddr>,
    wallet_router: Router,
    requester_router: Router,
) -> anyhow::Result<()> {
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
