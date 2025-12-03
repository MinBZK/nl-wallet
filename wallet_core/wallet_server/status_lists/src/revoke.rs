use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::routing::post;
use token_status_list::status_list_service::RevocationError;
use uuid::Uuid;

use token_status_list::status_list_service::StatusListRevocationService;
use utils::vec_at_least::VecNonEmpty;

async fn revoke_batch<L>(
    status_list_service: State<Arc<L>>,
    Json(batch_ids): Json<VecNonEmpty<Uuid>>,
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
    Router::new()
        .route("/revoke/", post(revoke_batch))
        .with_state(status_list_service)
}
