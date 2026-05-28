use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use http_utils::health::HealthChecker;
use http_utils::health::create_health_router;
use issuer_common::nonce_store::ProofNonceStore;
use issuer_common::par_store::IssuerParStore;
use itertools::Itertools;
use openid4vc::authorization_code_flow::AuthorizationCodeFlow;
use openid4vc::authorizing_issuer::AuthorizingIssuer;
use openid4vc::issuer::IssuanceData;
use openid4vc_server::issuer::create_authorization_router;
use openid4vc_server::issuer::create_issuance_router;
use server_utils::keys::PrivateKeyVariant;
use server_utils::server::add_cache_control_no_store_layer;
use server_utils::server::create_internal_listener;
use server_utils::server::create_wallet_listener;
use server_utils::server::listen;
use server_utils::server::secure_internal_router;
use server_utils::settings::Settings;
use server_utils::store::SessionStoreVariant;
use status_lists::postgres::NoRevokeAll;
use status_lists::postgres::PostgresStatusListService;
use status_lists::revoke::create_revocation_router;
use status_lists::serve::create_serve_router;
use tokio::net::TcpListener;
use utils::vec_at_least::VecNonEmpty;

pub type PidIssuer<AF> = AuthorizingIssuer<
    PrivateKeyVariant,
    PostgresStatusListService<PrivateKeyVariant, NoRevokeAll>,
    SessionStoreVariant<IssuanceData>,
    ProofNonceStore,
    IssuerParStore,
    AF,
>;

pub async fn serve<AF>(
    authorizing_issuer: Arc<PidIssuer<AF>>,
    auth_flow_router: Router,
    server_settings: Settings,
    serve_status_lists: bool,
    health_checkers: impl IntoIterator<Item = Box<dyn HealthChecker + Send + Sync>>,
) -> Result<()>
where
    AF: AuthorizationCodeFlow + Send + Sync + 'static,
{
    serve_with_listeners(
        create_wallet_listener(&server_settings.wallet_server).await?,
        create_internal_listener(&server_settings.internal_server).await?,
        authorizing_issuer,
        auth_flow_router,
        server_settings,
        serve_status_lists,
        health_checkers,
    )
    .await
}

#[expect(clippy::too_many_arguments, reason = "Setup function")]
pub async fn serve_with_listeners<AF>(
    wallet_listener: TcpListener,
    internal_listener: Option<TcpListener>,
    authorizing_issuer: Arc<PidIssuer<AF>>,
    auth_flow_router: Router,
    server_settings: Settings,
    serve_status_lists: bool,
    health_checkers: impl IntoIterator<Item = Box<dyn HealthChecker + Send + Sync>>,
) -> Result<()>
where
    AF: AuthorizationCodeFlow + Send + Sync + 'static,
{
    let status_list_services =
        VecNonEmpty::try_from(authorizing_issuer.issuer().status_lists().cloned().collect_vec())?;

    let issuance_router = create_issuance_router(Arc::clone(authorizing_issuer.issuer()));
    let authorization_router = create_authorization_router(Arc::clone(&authorizing_issuer));
    let mut router =
        add_cache_control_no_store_layer(issuance_router.merge(authorization_router).merge(auth_flow_router));

    if serve_status_lists {
        let status_list_router = create_serve_router(
            status_list_services
                .iter()
                .map(|service| service.config().to_route_source()),
        )?;

        router = router.merge(status_list_router);
    }

    let (internal_router, internal_openapi) = create_revocation_router(status_list_services);

    #[cfg(feature = "test_internal_ui")]
    let mut internal_router = internal_router.merge(
        utoipa_swagger_ui::SwaggerUi::new("/api-docs")
            .config(utoipa_swagger_ui::Config::default().validator_url("none"))
            .url("/openapi.json", internal_openapi),
    );

    #[cfg(not(feature = "test_internal_ui"))]
    let mut internal_router = internal_router.route("/openapi.json", axum::routing::get(axum::Json(internal_openapi)));

    internal_router = secure_internal_router(&server_settings.internal_server, internal_router);
    internal_router = add_cache_control_no_store_layer(internal_router);

    let health_router = create_health_router(health_checkers);

    listen(
        wallet_listener,
        internal_listener,
        router,
        internal_router,
        health_router,
        server_settings.log_requests,
    )
    .await
}
