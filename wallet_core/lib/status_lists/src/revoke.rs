use std::sync::Arc;

use axum::Json;
use axum::Router;
#[cfg(feature = "test_api")]
use axum::extract::Path;
use axum::extract::State;
use token_status_list::status_list_service::RevocationError;
use token_status_list::status_list_service::StatusListRevocationService;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

#[cfg(feature = "test_api")]
use crate::postgres::BatchIsRevoked;
use crate::postgres::PostgresRevocationHelper;

#[derive(Debug)]
struct RevocationRouterState<L> {
    status_list_service: Arc<L>,
    revocation_helper: PostgresRevocationHelper,
}

impl<L> Clone for RevocationRouterState<L> {
    fn clone(&self) -> Self {
        Self {
            status_list_service: Arc::clone(&self.status_list_service),
            revocation_helper: self.revocation_helper.clone(),
        }
    }
}

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
    State(RevocationRouterState {
        status_list_service, ..
    }): State<RevocationRouterState<L>>,
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
async fn list_batch<L>(
    State(RevocationRouterState { revocation_helper, .. }): State<RevocationRouterState<L>>,
) -> Result<Json<Vec<BatchIsRevoked>>, RevocationError>
where
    L: StatusListRevocationService + Send + Sync + 'static,
{
    Ok(Json(revocation_helper.list_attestation_batches().await?))
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
    State(RevocationRouterState { revocation_helper, .. }): State<RevocationRouterState<L>>,
    Path(batch_id): Path<Uuid>,
) -> Result<Json<BatchIsRevoked>, RevocationError>
where
    L: StatusListRevocationService + Send + Sync + 'static,
{
    Ok(Json(revocation_helper.get_attestation_batch(batch_id).await?))
}

pub fn create_revocation_router<L>(
    status_list_service: Arc<L>,
    revocation_helper: PostgresRevocationHelper,
) -> (Router, utoipa::openapi::OpenApi)
where
    L: StatusListRevocationService + Send + Sync + 'static,
{
    let router = OpenApiRouter::with_openapi(ApiDoc::openapi()).routes(routes!(revoke_batch));

    #[cfg(feature = "test_api")]
    let router = router.routes(routes!(get_batch)).routes(routes!(list_batch));

    let state = RevocationRouterState {
        status_list_service,
        revocation_helper,
    };
    let router = router.with_state(state);

    router.split_for_parts()
}
