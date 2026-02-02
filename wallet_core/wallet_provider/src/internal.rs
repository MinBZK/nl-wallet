use std::collections::HashSet;
use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::response::Response;
use http::StatusCode;
use tracing::warn;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use readable_identifier::ReadableIdentifierParseError;
use utils::generator::TimeGenerator;
use wallet_provider_service::revocation::RevocationResult;

use crate::router_state::RouterState;

#[derive(OpenApi)]
#[openapi(info(title = "Internal API"))]
struct ApiDoc;

#[derive(Debug, thiserror::Error)]
pub enum RevocationError {
    #[error("revocation service error: {0}")]
    RevocationService(#[from] wallet_provider_service::revocation::RevocationError),

    #[error("error parsing revocation code: {0}")]
    RevocationCodeParsing(#[from] ReadableIdentifierParseError),
}

impl IntoResponse for RevocationError {
    fn into_response(self) -> Response {
        warn!("error result: {:?}", self);
        match self {
            Self::RevocationService(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            Self::RevocationCodeParsing(_) => StatusCode::BAD_REQUEST.into_response(),
        }
    }
}

#[utoipa::path(
    post,
    path = "/revoke-wallets-by-id/",
    request_body(
        content = Vec<String>,
        content_type = "application/json",
        example = json!([
            "dozCMuQOCEJPtuSNXtB2VkCdaEFNMhEZ",
        ]),
    ),
    responses(
        (status = OK, description = "Successfully revoked the provided wallet IDs."),
    )
)]
async fn revoke_wallets_by_id<GRC, PIC>(
    State(router_state): State<Arc<RouterState<GRC, PIC>>>,
    Json(wallet_ids): Json<HashSet<String>>,
) -> Result<(), RevocationError>
where
    GRC: Send + Sync + 'static,
    PIC: Send + Sync + 'static,
{
    Ok(wallet_provider_service::revocation::revoke_wallets_by_wallet_id(
        &wallet_ids,
        &router_state.user_state,
        &TimeGenerator,
        &router_state.audit_log,
    )
    .await?)
}

#[utoipa::path(
    post,
    path = "/revoke-wallet-by-revocation-code/",
    request_body(
        content = String,
        content_type = "application/json",
        example = json!("67e55044-10b1-426f-9247-bb680e5fe0c8"),
    ),
    responses(
        (status = OK, description = "Successfully revoked the wallet."),
    )
)]
async fn revoke_wallet_by_revocation_code<GRC, PIC>(
    State(router_state): State<Arc<RouterState<GRC, PIC>>>,
    Json(revocation_code): Json<String>,
) -> Result<Json<RevocationResult>, RevocationError>
where
    GRC: Send + Sync + 'static,
    PIC: Send + Sync + 'static,
{
    let revocation_code = revocation_code.parse()?;

    let revocation_result = wallet_provider_service::revocation::revoke_wallet_by_revocation_code(
        revocation_code,
        &router_state.account_server.keys.revocation_code_key_identifier,
        &router_state.user_state,
        &TimeGenerator,
        &router_state.audit_log,
    )
    .await?;

    Ok(revocation_result.into())
}

#[utoipa::path(
    post,
    path = "/revoke-wallet-by-recovery-code/",
    request_body(
        content = String,
        example = json!("54aa94af2afc4da286967253a33a61410f0d069c0d77ff748fd83e9fc82c7526"),
    ),
    responses(
        (status = OK, description = "Successfully revoked the wallets."),
    )
)]
async fn revoke_wallets_by_recovery_code<GRC, PIC>(
    State(router_state): State<Arc<RouterState<GRC, PIC>>>,
    Json(recovery_code): Json<String>,
) -> Result<(), RevocationError>
where
    GRC: Send + Sync + 'static,
    PIC: Send + Sync + 'static,
{
    wallet_provider_service::revocation::revoke_wallets_by_recovery_code(
        &recovery_code,
        &router_state.user_state,
        &TimeGenerator,
    )
    .await?;

    Ok(())
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
    Ok(wallet_provider_service::revocation::revoke_all_wallets(
        &router_state.user_state,
        &TimeGenerator,
        &router_state.audit_log,
    )
    .await?)
}

#[cfg(feature = "test_internal_ui")]
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
        .routes(routes!(revoke_wallets_by_id))
        .routes(routes!(revoke_wallet_by_revocation_code))
        .routes(routes!(revoke_wallets_by_recovery_code))
        .routes(routes!(nuke));

    #[cfg(feature = "test_internal_ui")]
    let router = router.routes(routes!(list_wallets));

    let router = router.with_state(state);

    router.split_for_parts()
}
