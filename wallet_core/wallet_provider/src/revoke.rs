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
#[cfg(feature = "openapi")]
use utoipa_swagger_ui::SwaggerUi;

use crate::router_state::RouterState;

#[derive(OpenApi)]
#[openapi()]
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
            "67e55044-10b1-426f-9247-bb680e5fe0c8",
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
    // TODO store [`RevocationReason::AdminRequest`] (PVW-5302)
    // since this method takes an array and simply revokes all WUAs associated with all provided wallet IDs, a 404
    Ok(wallet_provider_service::revocation::revoke_wallets(wallet_ids, &router_state.user_state).await?)
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
    // TODO store [`RevocationReason::WalletSolutionCompromise`] (PVW-5302)
    Ok(wallet_provider_service::revocation::revoke_all_wallets(&router_state.user_state).await?)
}

#[utoipa::path(
    get,
    path = "/admin/wallet/",
    responses(
        (status = OK, body = Vec<String>, description = "Successfully listed the issued batch IDs."),
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

    #[cfg(feature = "openapi")]
    let router = {
        let (router, openapi) = router
            // only expose these routes when openapi feature is enabled
            // TODO .routes(routes!(get_wallet)) (PVW-5297)
            .routes(routes!(list_wallets))
            .split_for_parts();

        router.merge(SwaggerUi::new("/admin/api-docs").url("/openapi.json", openapi))
    };

    #[cfg(not(feature = "openapi"))]
    let router = router.into();

    router
}
