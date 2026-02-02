use std::sync::Arc;

use axum::Json;
use axum::Router;
#[cfg(feature = "test_api")]
use axum::extract::Path;
use axum::extract::State;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

#[cfg(feature = "test_api")]
use token_status_list::status_list_service::BatchIsRevoked;
use token_status_list::status_list_service::RevocationError;
use token_status_list::status_list_service::StatusListRevocationService;

#[derive(OpenApi)]
#[openapi(info(title = "Revocation API"))]
struct ApiDoc;

#[utoipa::path(
    post,
    path = "/revoke/",
    request_body(
        content = Vec<Uuid>,
        example = json!(["67e55044-10b1-426f-9247-bb680e5fe0c8"]),
    ),
    responses(
        (status = OK, description = "Successfully revoked the provided batch IDs.")
    )
)]
async fn revoke_batch<L>(
    State(status_list_service): State<Arc<L>>,
    Json(batch_ids): Json<Vec<Uuid>>,
) -> Result<(), RevocationError>
where
    L: StatusListRevocationService + Send + Sync + 'static,
{
    status_list_service.revoke_attestation_batches(batch_ids).await
}

#[cfg(feature = "test_api")]
#[utoipa::path(
    get,
    path = "/batch/",
    responses(
        (status = OK, body = Vec<BatchIsRevoked>, description = "Successfully listed the issued batch IDs."),
    )
)]
async fn list_batch<L>(State(status_list_service): State<Arc<L>>) -> Result<Json<Vec<BatchIsRevoked>>, RevocationError>
where
    L: StatusListRevocationService + Send + Sync + 'static,
{
    Ok(Json(status_list_service.list_attestation_batches().await?))
}

#[cfg(feature = "test_api")]
#[utoipa::path(
    get,
    path = "/batch/{batch_id}",
    params(
        ("batch_id" = Uuid, Path),
    ),
    responses(
        (status = OK, body = BatchIsRevoked, description = "Successfully found the selected batch ID."),
        (status = NOT_FOUND, description = "Batch ID not found."),
    )
)]
async fn get_batch<L>(
    State(status_list_service): State<Arc<L>>,
    Path(batch_id): Path<Uuid>,
) -> Result<Json<BatchIsRevoked>, RevocationError>
where
    L: StatusListRevocationService + Send + Sync + 'static,
{
    Ok(Json(status_list_service.get_attestation_batch(batch_id).await?))
}

pub fn create_revocation_router<L>(status_list_service: Arc<L>) -> (Router, utoipa::openapi::OpenApi)
where
    L: StatusListRevocationService + Send + Sync + 'static,
{
    let router = OpenApiRouter::with_openapi(ApiDoc::openapi()).routes(routes!(revoke_batch));

    #[cfg(feature = "test_api")]
    let router = router.routes(routes!(get_batch)).routes(routes!(list_batch));

    let router = router.with_state(status_list_service);

    router.split_for_parts()
}
