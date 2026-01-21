use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::response::Response;
use derive_more::Display;
use http::StatusCode;
use tracing::warn;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
#[cfg(feature = "test_admin_ui")]
use utoipa_swagger_ui::SwaggerUi;

use utils::generator::TimeGenerator;

use crate::router_state::RouterState;

#[derive(OpenApi)]
#[openapi(info(title = "Admin API"))]
struct ApiDoc;

#[derive(Debug, Display, thiserror::Error)]
pub struct RevocationError(#[from] wallet_provider_service::revocation::RevocationError);

impl IntoResponse for RevocationError {
    fn into_response(self) -> Response {
        warn!("error result: {:?}", self);
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

#[utoipa::path(
    post,
    path = "/admin/revoke/",
    request_body(
        content = Vec<String>,
        example = json!([
            "dozCMuQOCEJPtuSNXtB2VkCdaEFNMhEZ",
        ]),
    ),
    responses(
        (status = OK, description = "Successfully revoked the provided wallet IDs."),
    )
)]
async fn revoke_wallets<GRC, PIC>(
    State(router_state): State<Arc<RouterState<GRC, PIC>>>,
    Json(wallet_ids): Json<Vec<String>>,
) -> Result<(), RevocationError>
where
    GRC: Send + Sync + 'static,
    PIC: Send + Sync + 'static,
{
    // TODO since this method takes an array and simply revokes all WUAs associated with all provided wallet IDs, a 404
    // never occurs (PVW-5297)
    Ok(
        wallet_provider_service::revocation::revoke_wallets(&wallet_ids, &router_state.user_state, &TimeGenerator)
            .await?,
    )
}

#[utoipa::path(
    post,
    path = "/admin/nuke/",
    responses(
        (status = OK, description = "Successfully revoked all wallets."),
    )
)]
async fn nuke<GRC, PIC>(State(router_state): State<Arc<RouterState<GRC, PIC>>>) -> Result<(), RevocationError>
where
    GRC: Send + Sync + 'static,
    PIC: Send + Sync + 'static,
{
    Ok(wallet_provider_service::revocation::revoke_all_wallets(&router_state.user_state, &TimeGenerator).await?)
}

#[cfg(feature = "test_admin_ui")]
#[utoipa::path(
    get,
    path = "/admin/wallet/",
    responses(
        (
            status = OK,
            body = Vec<String>,
            description = "Successfully listed the registered wallet IDs.",
            example = json!([ "dozCMuQOCEJPtuSNXtB2VkCdaEFNMhEZ" ])
        ),
    )
)]
async fn list_wallets<GRC, PIC>(
    State(router_state): State<Arc<RouterState<GRC, PIC>>>,
) -> Result<Json<Vec<String>>, RevocationError>
where
    GRC: Send + Sync + 'static,
    PIC: Send + Sync + 'static,
{
    Ok(Json(
        wallet_provider_service::revocation::list_wallets(&router_state.user_state).await?,
    ))
}

pub fn internal_router<GRC, PIC>() -> Router<Arc<RouterState<GRC, PIC>>>
where
    PIC: Send + Sync + 'static,
    GRC: Send + Sync + 'static,
{
    let router = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(revoke_wallets))
        .routes(routes!(nuke));

    #[cfg(feature = "test_admin_ui")]
    // TODO .routes(routes!(get_wallet)) (PVW-5297)
    let router = router.routes(routes!(list_wallets));

    let (router, openapi) = router.split_for_parts();

    #[cfg(feature = "test_admin_ui")]
    let router = router.merge(SwaggerUi::new("/admin/api-docs").url("/openapi.json", openapi));

    #[cfg(not(feature = "test_admin_ui"))]
    let router = router.route("/openapi.json", axum::routing::get(Json(openapi)));

    router
}
