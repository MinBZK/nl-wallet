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
use utils::vec_at_least::VecNonEmpty;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

use crate::postgres::PostgresStatusListService;
use crate::postgres::RevokeAll;
#[cfg(feature = "test_api")]
use crate::postgres::revocation_helper::BatchIsRevoked;
#[cfg(feature = "test_api")]
use crate::postgres::revocation_helper::PostgresRevocationHelper;

#[derive(Debug)]
struct RevocationRouterState<K, R> {
    status_list_services: VecNonEmpty<PostgresStatusListService<K, R>>,
    #[cfg(feature = "test_api")]
    revocation_helper: PostgresRevocationHelper,
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
    State(state): State<Arc<RevocationRouterState<K, R>>>,
    Json(batch_ids): Json<Vec<Uuid>>,
) -> Result<(), RevocationError>
where
    K: EcdsaKeySend + Sync + 'static,
    R: RevokeAll + Clone + Sync + 'static,
{
    let service_count = state.status_list_services.len().get();

    try_join_all(
        state
            .status_list_services
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
    State(state): State<Arc<RevocationRouterState<K, R>>>,
) -> Result<Json<Vec<BatchIsRevoked>>, RevocationError> {
    Ok(Json(state.revocation_helper.list_attestation_batches().await?))
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
    State(state): State<Arc<RevocationRouterState<K, R>>>,
    Path(batch_id): Path<Uuid>,
) -> Result<Json<BatchIsRevoked>, RevocationError> {
    Ok(Json(state.revocation_helper.get_attestation_batch(batch_id).await?))
}

pub fn create_revocation_router<K, R>(
    status_list_services: VecNonEmpty<PostgresStatusListService<K, R>>,
) -> (Router, utoipa::openapi::OpenApi)
where
    K: EcdsaKeySend + Sync + 'static,
    R: RevokeAll + Clone + Sync + 'static,
{
    let router = OpenApiRouter::with_openapi(ApiDoc::openapi()).routes(routes!(revoke_batch));

    #[cfg(feature = "test_api")]
    let router = router.routes(routes!(get_batch)).routes(routes!(list_batch));

    #[cfg(feature = "test_api")]
    let revocation_helper = PostgresRevocationHelper::from_status_list(status_list_services.first());

    let state = RevocationRouterState {
        status_list_services,
        #[cfg(feature = "test_api")]
        revocation_helper,
    };
    let router = router.with_state(Arc::new(state));

    router.split_for_parts()
}
