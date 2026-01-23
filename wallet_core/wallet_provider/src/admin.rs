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
    path = "/revoke/",
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
    Ok(
        wallet_provider_service::revocation::revoke_wallets(&wallet_ids, &router_state.user_state, &TimeGenerator)
            .await?,
    )
}

#[utoipa::path(
    post,
    path = "/nuke/",
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
    path = "/wallet/",
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

pub fn internal_router<GRC, PIC>(state: Arc<RouterState<GRC, PIC>>) -> (Router, utoipa::openapi::OpenApi)
where
    PIC: Send + Sync + 'static,
    GRC: Send + Sync + 'static,
{
    let router = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(revoke_wallets))
        .routes(routes!(nuke));

    #[cfg(feature = "test_admin_ui")]
    let router = router.routes(routes!(list_wallets));

    let router = router.with_state(state);

    router.split_for_parts()
}
