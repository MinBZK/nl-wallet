use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::State;
use token_status_list::status_list_service::RevocationError;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
#[cfg(feature = "openapi")]
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

use token_status_list::status_list_service::StatusListRevocationService;

#[derive(OpenApi)]
#[openapi()]
struct ApiDoc;

#[utoipa::path(
    post,
    path = "/revoke/",
    request_body(
        content = Vec<String>,
        example = json!(["67e55044-10b1-426f-9247-bb680e5fe0c8"]),
    ),
    responses(
        (status = OK, description = "Successfully revoked the provided batch IDs."), 
        (status = NOT_FOUND, description = "One or more of the provided batch IDs were not found.")
    )
)]
async fn revoke_batch<L>(
    status_list_service: State<Arc<L>>,
    Json(batch_ids): Json<Vec<Uuid>>,
) -> Result<(), RevocationError>
where
    L: StatusListRevocationService + Send + Sync + 'static,
{
    status_list_service.revoke_attestation_batches(batch_ids).await
}

pub fn create_revocation_router<L>(status_list_service: Arc<L>) -> Router
where
    L: StatusListRevocationService + Send + Sync + 'static,
{
    let router = OpenApiRouter::with_openapi(ApiDoc::openapi()).routes(routes!(revoke_batch));

    #[cfg(feature = "openapi")]
    let router = {
        let (router, openapi) = router.split_for_parts();
        router.merge(SwaggerUi::new("/api-docs").url("/openapi.json", openapi))
    };

    #[cfg(not(feature = "openapi"))]
    let router = router.into();

    router.with_state(status_list_service)
}
