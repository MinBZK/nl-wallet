use std::io;

use anyhow::Result;
use axum::Router;
use hsm::service::Pkcs11Hsm;
use tokio::net::TcpListener;
use tower_http::validate_request::ValidateRequestHeaderLayer;
use tracing::info;

use openid4vc::server_state::SessionStore;
use openid4vc::verifier::DisclosureData;
use openid4vc::verifier::UseCases;
use openid4vc_server::verifier;
use server_utils::server::create_wallet_listener;
use server_utils::server::decorate_router;
use server_utils::settings::Authentication;
use server_utils::settings::RequesterAuth;
use server_utils::settings::Server;
use server_utils::settings::TryFromKeySettings;
use wallet_common::built_info::version_string;
use wallet_common::trust_anchor::BorrowingTrustAnchor;

use crate::settings::VerifierSettings;

pub async fn serve<S>(settings: VerifierSettings, hsm: Option<Pkcs11Hsm>, disclosure_sessions: S) -> Result<()>
where
    S: SessionStore<DisclosureData> + Send + Sync + 'static,
{
    let log_requests = settings.server_settings.log_requests;

    let (wallet_disclosure_router, requester_router) = verifier::create_routers(
        settings.server_settings.public_url,
        settings.universal_link_base_url,
        UseCases::try_from_key_settings(settings.usecases, hsm).await?,
        (&settings.ephemeral_id_secret).into(),
        settings
            .server_settings
            .issuer_trust_anchors
            .iter()
            .map(BorrowingTrustAnchor::to_owned_trust_anchor)
            .collect(),
        settings.allow_origins,
        disclosure_sessions,
    );

    listen(
        settings.server_settings.wallet_server,
        settings.requester_server,
        Router::new().nest("/disclosure", wallet_disclosure_router),
        Router::new().nest("/disclosure", requester_router),
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
    wallet_server: Server,
    requester_server: RequesterAuth,
    mut wallet_router: Router,
    mut requester_router: Router,
    log_requests: bool,
) -> Result<()> {
    let wallet_listener = create_wallet_listener(wallet_server).await?;
    let requester_listener = create_requester_listener(&requester_server).await?;

    requester_router = secure_requester_router(&requester_server, requester_router);

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
