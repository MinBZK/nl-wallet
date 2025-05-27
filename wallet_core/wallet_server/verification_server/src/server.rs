use std::io;
use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use hsm::service::Pkcs11Hsm;
use tokio::net::TcpListener;
use tower_http::validate_request::ValidateRequestHeaderLayer;
use tracing::info;

use crypto::trust_anchor::BorrowingTrustAnchor;
use openid4vc::server_state::SessionStore;
use openid4vc::verifier::DisclosureData;
use openid4vc_server::verifier::VerifierFactory;
use server_utils::server::create_wallet_listener;
use server_utils::server::decorate_router;
use server_utils::settings::Authentication;
use server_utils::settings::RequesterAuth;
use utils::built_info::version_string;

use crate::settings::VerifierSettings;

pub async fn serve<S>(settings: VerifierSettings, hsm: Option<Pkcs11Hsm>, disclosure_sessions: Arc<S>) -> Result<()>
where
    S: SessionStore<DisclosureData> + Send + Sync + 'static,
{
    let wallet_listener = create_wallet_listener(&settings.server_settings.wallet_server).await?;
    let requester_listener = create_requester_listener(&settings.requester_server).await?;
    serve_with_listeners(wallet_listener, requester_listener, settings, hsm, disclosure_sessions).await
}

pub async fn serve_with_listeners<S>(
    wallet_listener: TcpListener,
    requester_listener: Option<TcpListener>,
    settings: VerifierSettings,
    hsm: Option<Pkcs11Hsm>,
    disclosure_sessions: Arc<S>,
) -> Result<()>
where
    S: SessionStore<DisclosureData> + Send + Sync + 'static,
{
    // Needed when called directly
    check_requester_listener_with_settings(&requester_listener, &settings);
    let log_requests = settings.server_settings.log_requests;

    let usecases = settings
        .usecases
        .parse(
            hsm,
            (&settings.ephemeral_id_secret).into(),
            Arc::clone(&disclosure_sessions),
        )
        .await?;

    let (wallet_disclosure_router, requester_router) = VerifierFactory::new(
        settings.server_settings.public_url.join_base_url("disclosure/sessions"),
        settings.universal_link_base_url,
        usecases,
        settings
            .server_settings
            .issuer_trust_anchors
            .iter()
            .map(BorrowingTrustAnchor::to_owned_trust_anchor)
            .collect(),
        settings.wallet_client_ids,
    )
    .create_routers(settings.allow_origins, disclosure_sessions, None);

    let requester_router = secure_requester_router(&settings.requester_server, requester_router);

    listen(
        wallet_listener,
        requester_listener,
        Router::new().nest("/disclosure/sessions", wallet_disclosure_router),
        Router::new().nest("/disclosure/sessions", requester_router),
        log_requests,
    )
    .await
}

/// Secure [requester_router] with an API key when required by [settings].
fn secure_requester_router(requester_server: &RequesterAuth, requester_router: Router) -> Router {
    match requester_server {
        RequesterAuth::Authentication(Authentication::ApiKey(api_key))
        | RequesterAuth::ProtectedInternalEndpoint {
            authentication: Authentication::ApiKey(api_key),
            ..
        } => requester_router.layer(ValidateRequestHeaderLayer::bearer(api_key)),
        RequesterAuth::InternalEndpoint(_) => requester_router,
    }
}

/// Sanity check to see if [requester_listener] is set conform [settings].
fn check_requester_listener_with_settings(requester_listener: &Option<TcpListener>, settings: &VerifierSettings) {
    match settings.requester_server {
        RequesterAuth::Authentication(_) => {
            assert!(
                requester_listener.is_none(),
                "no request listener should be provided for authentication only"
            );
        }
        RequesterAuth::ProtectedInternalEndpoint { .. } | RequesterAuth::InternalEndpoint(_) => {
            assert!(
                requester_listener.is_some(),
                "a request listener should be provided for internal endpoint"
            );
        }
    }
}

/// Create Requester listener when required by [settings].
async fn create_requester_listener(requester_server: &RequesterAuth) -> Result<Option<TcpListener>, io::Error> {
    match requester_server {
        RequesterAuth::Authentication(_) => None,
        RequesterAuth::ProtectedInternalEndpoint { server, .. } | RequesterAuth::InternalEndpoint(server) => {
            TcpListener::bind((server.ip, server.port)).await.into()
        }
    }
    .transpose()
}

async fn listen(
    wallet_listener: TcpListener,
    requester_listener: Option<TcpListener>,
    mut wallet_router: Router,
    mut requester_router: Router,
    log_requests: bool,
) -> Result<()> {
    info!("{}", version_string());

    match requester_listener {
        Some(requester_listener) => {
            wallet_router = decorate_router(wallet_router, log_requests);
            requester_router = decorate_router(requester_router, log_requests);

            info!("listening for requester on {}", requester_listener.local_addr()?);
            let requester_server = tokio::spawn(async move {
                axum::serve(requester_listener, requester_router)
                    .await
                    .expect("requester server should be started");
            });

            info!("listening for wallet on {}", wallet_listener.local_addr()?);
            let wallet_server = tokio::spawn(async move {
                axum::serve(wallet_listener, wallet_router)
                    .await
                    .expect("wallet server should be started");
            });

            tokio::try_join!(requester_server, wallet_server)?;
        }
        None => {
            wallet_router = decorate_router(wallet_router.merge(requester_router), log_requests);
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
