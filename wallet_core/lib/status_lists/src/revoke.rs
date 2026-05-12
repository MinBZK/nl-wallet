use std::sync::Arc;

use axum::Json;
use axum::Router;
#[cfg(feature = "test_api")]
use axum::extract::Path;
use axum::extract::State;
use crypto::EcdsaKeySend;
use futures::future::try_join_all;
use itertools::Itertools;
use token_status_list::status_list_service::RevocationError;
use token_status_list::status_list_service::StatusListService;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

#[cfg(feature = "test_api")]
use crate::postgres::BatchIsRevoked;
use crate::postgres::PostgresRevocationHelper;
use crate::postgres::PostgresStatusListService;
use crate::postgres::RevokeAll;

#[derive(Debug)]
struct RevocationRouterState<K, R> {
    status_list_services: Vec<Arc<PostgresStatusListService<K, R>>>,
    revocation_helper: PostgresRevocationHelper,
}

impl<K, R> Clone for RevocationRouterState<K, R> {
    fn clone(&self) -> Self {
        Self {
            status_list_services: self.status_list_services.iter().map(Arc::clone).collect(),
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
async fn revoke_batch<K, R>(
    State(RevocationRouterState {
        status_list_services, ..
    }): State<RevocationRouterState<K, R>>,
    Json(batch_ids): Json<Vec<Uuid>>,
) -> Result<(), RevocationError>
where
    K: EcdsaKeySend + Sync + 'static,
    R: RevokeAll + Clone + Sync + 'static,
{
    let service_count = status_list_services.len();

    try_join_all(
        status_list_services
            .iter()
            .zip_eq(itertools::repeat_n(batch_ids, service_count))
            .map(|(service, batch_ids)| service.revoke_attestation_batches(batch_ids)),
    )
    .await
    .map(|_| ())
}

#[cfg(feature = "test_api")]
#[utoipa::path(
    get,
    path = "/batch/",
    responses(
        (status = OK, body = Vec<BatchIsRevoked>, description = "Successfully listed the issued batch IDs."),
    )
)]
async fn list_batch<K, R>(
    State(RevocationRouterState { revocation_helper, .. }): State<RevocationRouterState<K, R>>,
) -> Result<Json<Vec<BatchIsRevoked>>, RevocationError> {
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
async fn get_batch<K, R>(
    State(RevocationRouterState { revocation_helper, .. }): State<RevocationRouterState<K, R>>,
    Path(batch_id): Path<Uuid>,
) -> Result<Json<BatchIsRevoked>, RevocationError> {
    Ok(Json(revocation_helper.get_attestation_batch(batch_id).await?))
}

pub fn create_revocation_router<K, R>(
    status_list_services: Vec<Arc<PostgresStatusListService<K, R>>>,
    revocation_helper: PostgresRevocationHelper,
) -> (Router, utoipa::openapi::OpenApi)
where
    K: EcdsaKeySend + Sync + 'static,
    R: RevokeAll + Clone + Sync + 'static,
{
    let router = OpenApiRouter::with_openapi(ApiDoc::openapi()).routes(routes!(revoke_batch));

    #[cfg(feature = "test_api")]
    let router = router.routes(routes!(get_batch)).routes(routes!(list_batch));

    let state = RevocationRouterState {
        status_list_services,
        revocation_helper,
    };
    let router = router.with_state(state);

    router.split_for_parts()
}
