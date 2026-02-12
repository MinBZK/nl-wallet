use std::collections::HashSet;
use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::Path;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::response::NoContent;
use axum::response::Response;
use chrono::DateTime;
use chrono::Utc;
use http::StatusCode;
use serde::Deserialize;
use serde::Serialize;
use tracing::warn;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use readable_identifier::ReadableIdentifierParseError;
use utils::generator::TimeGenerator;
#[cfg(feature = "test_internal_ui")]
use wallet_provider_domain::model::wallet_user::WalletUserIsRevoked;

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

#[derive(Debug, Clone, Deserialize, Serialize, utoipa::ToSchema)]
struct NotFoundResponse {
    missing_wallet_ids: HashSet<String>,
}

impl IntoResponse for RevocationError {
    fn into_response(self) -> Response {
        warn!("error result: {:?}", self);
        match self {
            Self::RevocationService(wallet_provider_service::revocation::RevocationError::RevocationCodeNotFound(
                _,
            )) => StatusCode::NOT_FOUND.into_response(),
            Self::RevocationService(wallet_provider_service::revocation::RevocationError::WalletIdsNotFound(
                missing_wallet_ids,
            )) => (StatusCode::NOT_FOUND, Json(NotFoundResponse { missing_wallet_ids })).into_response(),
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
        (status = NOT_FOUND, body = NotFoundResponse, description = "One or more wallet IDs were not found."),
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

#[derive(Debug, Clone, Deserialize, Serialize, utoipa::ToSchema)]
pub struct RevokeByRevocationCodeResponse {
    revoked_at: DateTime<Utc>,
}

#[utoipa::path(
    post,
    path = "/revoke-wallet-by-revocation-code/",
    request_body(
        content = String,
        content_type = "application/json",
        example = json!("C20C-KF0R-D32B-A5E3-2X"),
    ),
    responses(
        (status = OK, body = RevokeByRevocationCodeResponse, description = "Successfully revoked the wallet."),
        (status = NOT_FOUND, description = "No wallet found for the provided revocation code.")
    )
)]
async fn revoke_wallet_by_revocation_code<GRC, PIC>(
    State(router_state): State<Arc<RouterState<GRC, PIC>>>,
    Json(revocation_code): Json<String>,
) -> Result<Json<RevokeByRevocationCodeResponse>, RevocationError>
where
    GRC: Send + Sync + 'static,
    PIC: Send + Sync + 'static,
{
    let revocation_code = revocation_code.parse()?;

    let revoked_at = wallet_provider_service::revocation::revoke_wallet_by_revocation_code(
        revocation_code,
        &router_state.account_server.keys.revocation_code_key_identifier,
        &router_state.user_state,
        &TimeGenerator,
        &router_state.audit_log,
    )
    .await?;

    Ok(Json(RevokeByRevocationCodeResponse { revoked_at }))
}

#[derive(Debug, Clone, Deserialize, Serialize, utoipa::ToSchema)]
pub struct RevokeByRecoveryCodeResponse {
    found_wallet_count: usize,
}

#[utoipa::path(
    post,
    path = "/revoke-wallet-by-recovery-code/",
    request_body(
        content = String,
        content_type = "application/json",
        example = json!("54aa94af2afc4da286967253a33a61410f0d069c0d77ff748fd83e9fc82c7526"),
    ),
    responses(
        (status = OK, body = RevokeByRecoveryCodeResponse, description = "Successfully revoked the wallets."),
    )
)]
async fn revoke_wallets_by_recovery_code<GRC, PIC>(
    State(router_state): State<Arc<RouterState<GRC, PIC>>>,
    Json(recovery_code): Json<String>,
) -> Result<Json<RevokeByRecoveryCodeResponse>, RevocationError>
where
    GRC: Send + Sync + 'static,
    PIC: Send + Sync + 'static,
{
    let found_wallet_count = wallet_provider_service::revocation::revoke_wallets_by_recovery_code(
        &recovery_code,
        &router_state.user_state,
        &TimeGenerator,
        &router_state.audit_log,
    )
    .await?;

    Ok(Json(RevokeByRecoveryCodeResponse { found_wallet_count }))
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
            body = Vec<WalletUserIsRevoked>,
            description = "Successfully listed the registered wallets.",
        ),
    )
)]
async fn list_wallets<GRC, PIC>(
    State(router_state): State<Arc<RouterState<GRC, PIC>>>,
) -> Result<Json<Vec<WalletUserIsRevoked>>, RevocationError>
where
    GRC: Send + Sync + 'static,
    PIC: Send + Sync + 'static,
{
    Ok(Json(
        wallet_provider_service::revocation::list_wallets(&router_state.user_state)
            .await?
            .into_iter()
            .map(|w| WalletUserIsRevoked {
                wallet_id: w.wallet_id,
                recovery_code: w.recovery_code,
                state: w.state,
                revocation_registration: w.revocation_registration,
                can_register_new_wallet: w.can_register_new_wallet,
            })
            .collect(),
    ))
}

#[utoipa::path(
    get,
    path = "/deny-list/",
    responses(
        (
            status = OK,
            body = Vec<String>,
            description = "Successfully listed the denied recovery codes.",
            example = json!([
                "54aa94af2afc4da286967253a33a61410f0d069c0d77ff748fd83e9fc82c7526",
                "cff292503cba8c4fbf2e5820dcdc468ae00f40c87b1af35513375800128fc00d"
            ])
        ),
    )
)]
async fn list_denied_recovery_codes<GRC, PIC>(
    State(router_state): State<Arc<RouterState<GRC, PIC>>>,
) -> Result<Json<Vec<String>>, RevocationError>
where
    GRC: Send + Sync + 'static,
    PIC: Send + Sync + 'static,
{
    Ok(Json(
        wallet_provider_service::revocation::list_denied_recovery_codes(&router_state.user_state).await?,
    ))
}

#[utoipa::path(
    delete,
    path = "/deny-list/{recovery-code}",
    params(
         (
             "recovery-code" = String,
             Path,
             description = "The recovery code to remove from the deny list.",
             example = "54aa94af2afc4da286967253a33a61410f0d069c0d77ff748fd83e9fc82c7526"
         ),
    ),
    responses(
        (
            status = NO_CONTENT,
            description = "Successfully removed recovery code from the deny list.",
        ),
        (
            status = NOT_FOUND,
            description = "The recovery code was not on the deny list.",
        ),
    )
)]
async fn remove_denied_recovery_code<GRC, PIC>(
    State(router_state): State<Arc<RouterState<GRC, PIC>>>,
    Path(recovery_code): Path<String>,
) -> Result<NoContent, RevocationError>
where
    GRC: Send + Sync + 'static,
    PIC: Send + Sync + 'static,
{
    wallet_provider_service::revocation::remove_denied_recovery_code(&router_state.user_state, &recovery_code).await?;

    Ok(NoContent)
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
        .routes(routes!(nuke))
        .routes(routes!(list_denied_recovery_codes))
        .routes(routes!(remove_denied_recovery_code));

    #[cfg(feature = "test_internal_ui")]
    let router = router.routes(routes!(list_wallets));

    let router = router.with_state(state);

    router.split_for_parts()
}
