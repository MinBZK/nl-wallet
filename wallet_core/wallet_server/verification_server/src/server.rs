use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use tokio::net::TcpListener;

use crypto::trust_anchor::BorrowingTrustAnchor;
use hsm::service::Pkcs11Hsm;
use http_utils::health::create_health_router;
use openid4vc::server_state::SessionStore;
use openid4vc::verifier::DisclosureData;
use openid4vc_server::verifier::VerifierFactory;
use server_utils::server::add_cache_control_no_store_layer;
use server_utils::server::check_internal_listener_with_settings;
use server_utils::server::create_internal_listener;
use server_utils::server::create_wallet_listener;
use server_utils::server::listen;
use server_utils::server::secure_internal_router;
use token_status_list::verification::client::StatusListClient;
use token_status_list::verification::verifier::RevocationVerifier;
use utils::generator::TimeGenerator;

use crate::settings::VerifierSettings;

pub async fn serve<S, C>(
    settings: VerifierSettings,
    hsm: Option<Pkcs11Hsm>,
    disclosure_sessions: Arc<S>,
    status_list_client: C,
) -> Result<()>
where
    S: SessionStore<DisclosureData> + Send + Sync + 'static,
    C: StatusListClient + Sync + 'static,
{
    serve_with_listeners(
        create_wallet_listener(&settings.server_settings.wallet_server).await?,
        create_internal_listener(&settings.server_settings.internal_server).await?,
        settings,
        hsm,
        disclosure_sessions,
        status_list_client,
    )
    .await
}

pub async fn serve_with_listeners<S, C>(
    wallet_listener: TcpListener,
    requester_listener: Option<TcpListener>,
    settings: VerifierSettings,
    hsm: Option<Pkcs11Hsm>,
    disclosure_sessions: Arc<S>,
    status_list_client: C,
) -> Result<()>
where
    S: SessionStore<DisclosureData> + Send + Sync + 'static,
    C: StatusListClient + Sync + 'static,
{
    // Needed when called directly
    check_internal_listener_with_settings(&requester_listener, &settings.server_settings.internal_server);
    let log_requests = settings.server_settings.log_requests;

    let usecases = settings
        .usecases
        .parse(
            hsm,
            (&settings.ephemeral_id_secret).into(),
            Arc::clone(&disclosure_sessions),
        )
        .await?;

    let revocation_verifier = RevocationVerifier::new(
        Arc::new(status_list_client),
        settings.status_list_token_cache_settings.capacity,
        settings.status_list_token_cache_settings.default_ttl,
        settings.status_list_token_cache_settings.error_ttl,
        TimeGenerator,
    );

    let (wallet_disclosure_router, requester_router) = VerifierFactory::new(
        settings.public_url.join_base_url("disclosure/sessions"),
        settings.universal_link_base_url,
        usecases,
        settings
            .server_settings
            .issuer_trust_anchors
            .iter()
            .map(BorrowingTrustAnchor::to_owned_trust_anchor)
            .collect(),
        settings.wallet_client_ids,
        settings.extending_vct_values.unwrap_or_default(),
    )
    .create_routers(settings.allow_origins, disclosure_sessions, revocation_verifier, None);

    let requester_router = secure_internal_router(&settings.server_settings.internal_server, requester_router);
    listen(
        wallet_listener,
        requester_listener,
        Router::new().nest(
            "/disclosure/sessions",
            add_cache_control_no_store_layer(wallet_disclosure_router),
        ),
        Router::new().nest(
            "/disclosure/sessions",
            add_cache_control_no_store_layer(requester_router),
        ),
        create_health_router([]),
        log_requests,
    )
    .await
}
