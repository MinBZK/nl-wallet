use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use p256::ecdsa::VerifyingKey;
use tokio::net::TcpListener;

use hsm::service::Pkcs11Hsm;
use issuer_settings::settings::IssuerSettings;
use openid4vc::issuer::AttributeService;
use openid4vc::issuer::Issuer;
use openid4vc::issuer::WuaConfig;
use openid4vc::server_state::SessionStore;
use openid4vc_server::issuer::create_issuance_router;
use server_utils::server::add_cache_control_no_store_layer;
use server_utils::server::create_internal_listener;
use server_utils::server::create_wallet_listener;
use server_utils::server::listen;
use server_utils::server::secure_internal_router;
use status_lists::revoke::create_revocation_router;
use token_status_list::status_list_service::StatusListRevocationService;
use token_status_list::status_list_service::StatusListServices;

#[expect(clippy::too_many_arguments, reason = "Setup function")]
pub async fn serve<A, L, IS>(
    attr_service: A,
    settings: IssuerSettings,
    hsm: Option<Pkcs11Hsm>,
    issuance_sessions: Arc<IS>,
    wua_issuer_pubkey: VerifyingKey,
    status_list_services: L,
    status_list_router: Option<Router>,
    health_router: Router,
) -> Result<()>
where
    A: AttributeService + Send + Sync + 'static,
    L: StatusListServices + StatusListRevocationService + Send + Sync + 'static,
    IS: SessionStore<openid4vc::issuer::IssuanceData> + Send + Sync + 'static,
{
    serve_with_listeners(
        create_wallet_listener(&settings.server_settings.wallet_server).await?,
        create_internal_listener(&settings.server_settings.internal_server).await?,
        attr_service,
        settings,
        hsm,
        issuance_sessions,
        wua_issuer_pubkey,
        status_list_services,
        status_list_router,
        health_router,
    )
    .await
}

#[expect(clippy::too_many_arguments, reason = "Setup function")]
pub async fn serve_with_listeners<A, L, IS>(
    wallet_listener: TcpListener,
    internal_listener: Option<TcpListener>,
    attr_service: A,
    settings: IssuerSettings,
    hsm: Option<Pkcs11Hsm>,
    issuance_sessions: Arc<IS>,
    wua_issuer_pubkey: VerifyingKey,
    status_list_services: L,
    status_list_router: Option<Router>,
    health_router: Router,
) -> Result<()>
where
    A: AttributeService + Send + Sync + 'static,
    L: StatusListServices + StatusListRevocationService + Send + Sync + 'static,
    IS: SessionStore<openid4vc::issuer::IssuanceData> + Send + Sync + 'static,
{
    let log_requests = settings.server_settings.log_requests;

    let attestation_config = settings.attestation_settings.parse(&hsm, &settings.metadata).await?;

    let status_list_services = Arc::new(status_list_services);
    let wallet_issuance_router = create_issuance_router(Arc::new(Issuer::new(
        issuance_sessions,
        attr_service,
        attestation_config,
        settings.public_url,
        settings.wallet_client_ids,
        Some(WuaConfig {
            wua_issuer_pubkey: (&wua_issuer_pubkey).into(),
        }),
        Arc::clone(&status_list_services),
    )));

    let mut router = add_cache_control_no_store_layer(wallet_issuance_router);
    if let Some(status_list_router) = status_list_router {
        router = router.merge(status_list_router);
    }

    let (internal_router, internal_openapi) = create_revocation_router(status_list_services);

    #[cfg(feature = "test_internal_ui")]
    let mut internal_router =
        internal_router.merge(utoipa_swagger_ui::SwaggerUi::new("/api-docs").url("/openapi.json", internal_openapi));

    #[cfg(not(feature = "test_internal_ui"))]
    let mut internal_router = internal_router.route("/openapi.json", axum::routing::get(axum::Json(internal_openapi)));

    internal_router = secure_internal_router(&settings.server_settings.internal_server, internal_router);
    internal_router = add_cache_control_no_store_layer(internal_router);
    listen(
        wallet_listener,
        internal_listener,
        router,
        internal_router,
        health_router,
        log_requests,
    )
    .await
}
