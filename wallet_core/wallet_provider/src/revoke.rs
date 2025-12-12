use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::response::Response;
use derive_more::Display;
use http::StatusCode;
use serde::Deserialize;
use serde::Serialize;
use serde_with::DeserializeFromStr;
use serde_with::SerializeDisplay;
use tracing::warn;
use utoipa::OpenApi;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
#[cfg(feature = "openapi")]
use utoipa_swagger_ui::SwaggerUi;

use crate::router_state::RouterState;

#[derive(OpenApi)]
#[openapi()]
struct ApiDoc;

#[derive(Debug, Clone, Copy, SerializeDisplay, DeserializeFromStr, strum::EnumString, strum::Display, ToSchema)]
#[strum(serialize_all = "snake_case")]
enum RevocationReason {
    // upon the explicit request of the User
    UserRequest,
    // can have several reasons, e.g.,
    // * the security of the mobile device and OS on which the corresponding Wallet Instance is installed
    // * ...
    AdminRequest,
    // the security of the Wallet Solution is breached or compromised
    WalletSolutionCompromised,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
struct RevocationRequest {
    reason: RevocationReason,
    wallet_id: String,
}

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
        content = Vec<RevocationRequest>,
        example = json!([{
            "reason": "user_request",
            "wallet_id": "67e55044-10b1-426f-9247-bb680e5fe0c8",
        }]),
    ),
    responses(
        (status = OK, description = "Successfully revoked the provided wallet IDs."),
    )
)]
async fn revoke_wallets<GRC, PIC>(
    State(router_state): State<Arc<RouterState<GRC, PIC>>>,
    Json(revocation_requests): Json<Vec<RevocationRequest>>,
) -> Result<(), RevocationError>
where
    GRC: Send + Sync + 'static,
    PIC: Send + Sync + 'static,
{
    // TODO verify revocation reason != WalletSolutionCompromised
    // since this method takes an array and simply revokes all WUAs associated with all provided wallet IDs, a 404
    Ok(wallet_provider_service::revocation::revoke_wallets(
        revocation_requests.into_iter().map(|req| req.wallet_id).collect(), // TODO don't collect?
        &router_state.user_state,
    )
    .await?)
}

#[utoipa::path(
    post,
    path = "/admin/nuke/",
    responses(
        (status = OK, description = "Successfully nuked the system."),
    )
)]
async fn nuke<GRC, PIC>(State(router_state): State<Arc<RouterState<GRC, PIC>>>) -> Result<(), RevocationError>
where
    GRC: Send + Sync + 'static,
    PIC: Send + Sync + 'static,
{
    // TODO revocation reason == WalletSolutionCompromised
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
            // TODO .routes(routes!(get_wallet))
            .routes(routes!(list_wallets))
            .split_for_parts();

        router.merge(SwaggerUi::new("/admin/api-docs").url("/openapi.json", openapi))
    };

    #[cfg(not(feature = "openapi"))]
    let router = router.into();

    router
}
