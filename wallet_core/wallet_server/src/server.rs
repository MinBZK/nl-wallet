use std::net::SocketAddr;

use axum::{routing::get, Router};
use tower_http::{trace::TraceLayer, validate_request::ValidateRequestHeaderLayer};
use tracing::debug;

#[cfg(feature = "disclosure")]
use nl_wallet_mdoc::{server_state::SessionStore, verifier::DisclosureData};

use crate::{
    log_requests::log_request_response,
    settings::{Authentication, RequesterAuth, Settings},
};

#[cfg(all(feature = "issuance", feature = "disclosure"))]
use openid4vc::issuer::AttributeService;

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

pub fn create_wallet_socket(settings: &Settings) -> SocketAddr {
    SocketAddr::new(settings.wallet_server.ip, settings.wallet_server.port)
}

pub fn secure_router(settings: &Settings, requester_router: Router) -> Router {
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

pub fn get_requester_socket(settings: &Settings) -> Option<SocketAddr> {
    match &settings.requester_server {
        RequesterAuth::Authentication(Authentication::ApiKey(_)) => None,
        RequesterAuth::ProtectedInternalEndpoint {
            authentication: Authentication::ApiKey(_),
            server,
        } => Some(SocketAddr::new(server.ip, server.port)),
        RequesterAuth::InternalEndpoint(server) => Some(SocketAddr::new(server.ip, server.port)),
    }
}

#[cfg(feature = "disclosure")]
pub fn setup_disclosure<S>(
    settings: Settings,
    disclosure_sessions: S,
) -> anyhow::Result<(Option<SocketAddr>, Router, Router)>
where
    S: SessionStore<DisclosureData> + Send + Sync + 'static,
{
    let (wallet_disclosure_router, mut requester_router) =
        crate::verifier::create_routers(settings.clone(), disclosure_sessions)?;

    let requester_socket = get_requester_socket(&settings);
    requester_router = secure_router(&settings, requester_router);

    Ok((requester_socket, wallet_disclosure_router, requester_router))
}

#[cfg(feature = "disclosure")]
pub async fn serve_disclosure<S>(settings: Settings, disclosure_sessions: S) -> anyhow::Result<()>
where
    S: SessionStore<DisclosureData> + Send + Sync + 'static,
{
    let log_requests = settings.log_requests;

    let wallet_socket = create_wallet_socket(&settings);
    let (requester_socket, wallet_disclosure_router, requester_router) =
        setup_disclosure(settings, disclosure_sessions)?;

    listen(
        wallet_socket,
        requester_socket,
        decorate_router("/disclosure/", wallet_disclosure_router, log_requests),
        decorate_router("/disclosure/sessions", requester_router, log_requests),
    )
    .await?;

    Ok(())
}

#[cfg(all(feature = "issuance", feature = "disclosure"))]
pub async fn serve_full<A, DS, IS>(
    attr_service: A,
    settings: Settings,
    disclosure_sessions: DS,
    issuance_sessions: IS,
) -> anyhow::Result<()>
where
    A: AttributeService + Send + Sync + 'static,
    DS: SessionStore<DisclosureData> + Send + Sync + 'static,
    IS: SessionStore<openid4vc::issuer::IssuanceData> + Send + Sync + 'static,
{
    let log_requests = settings.log_requests;

    let wallet_socket = create_wallet_socket(&settings);
    let (requester_socket, wallet_disclosure_router, requester_router) =
        setup_disclosure(settings.clone(), disclosure_sessions)?;

    let wallet_issuance_router =
        crate::issuer::create_issuance_router(settings, issuance_sessions, attr_service).await?;

    listen(
        wallet_socket,
        requester_socket,
        decorate_router("/issuance/", wallet_issuance_router, log_requests).merge(decorate_router(
            "/disclosure/",
            wallet_disclosure_router,
            log_requests,
        )),
        decorate_router("/disclosure/sessions", requester_router, log_requests),
    )
    .await?;

    Ok(())
}

pub async fn listen(
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
