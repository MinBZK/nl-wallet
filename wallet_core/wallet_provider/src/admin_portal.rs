use std::sync::Arc;

use audit_log::model::DbAuditLog;
use audit_log::repository::PostgresAuditLogRepository;
use axum::Json;
use axum::Router;
use axum::extract::FromRequestParts;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum::response::IntoResponse;
use axum::response::Redirect;
use axum::response::Response;
use axum::routing::get;
use axum::routing::post;
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::cookie::SameSite;
use chrono::DateTime;
use chrono::Utc;
use http_utils::reqwest::ReqwestTrustAnchor;
use http_utils::reqwest::tls_pinned_client_builder;
use http_utils::urls::BaseUrl;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use tracing::warn;
use url::Url;
use utils::generator::TimeGenerator;
use uuid::Uuid;
use wallet_provider_domain::model::admin_task::AdminTask;
use wallet_provider_domain::model::admin_task::ReviewVerdict;
use wallet_provider_domain::model::admin_task::TaskPayload;
use wallet_provider_domain::model::admin_task::TaskStatus;
use wallet_provider_domain::model::wallet_user::WalletId;
use wallet_provider_persistence::repositories::Repositories;
use wallet_provider_service::admin_portal::AdminPortalConfig;
use wallet_provider_service::admin_portal::AdminPortalService;
use wallet_provider_service::admin_portal::LoggedInUser;
use wallet_provider_service::admin_portal::UuidV7Generator;
use wallet_provider_service::admin_portal::task::CreateTaskError;
use wallet_provider_service::admin_portal::task::ExecuteTaskError;
use wallet_provider_service::admin_portal::task::ListTasksError;
use wallet_provider_service::admin_portal::task::RetrieveTaskError;
use wallet_provider_service::admin_portal::task::ReviewTaskError;
use wallet_provider_service::admin_portal::task::TaskListViewKind;
use wallet_provider_service::admin_portal::task::WithdrawTaskError;
use wallet_provider_service::admin_portal::wallet_lookup::GetWalletRevocationStateError;
use wallet_provider_service::admin_portal::wallet_lookup::WalletRevocationState;

use crate::router_state::ProductionUserState;
use crate::settings::AdminPortalSettings;

const SESSION_ID_COOKIE: &str = "sid";

/// Failure of a `/tasks` endpoint, mapped to the matching HTTP status code.
#[derive(Debug)]
enum TaskApiError {
    InvalidRequest(JsonRejection),
    Create(CreateTaskError),
    Withdraw(WithdrawTaskError),
    Review(ReviewTaskError),
    List(ListTasksError),
    Execution(ExecuteTaskError),
    Retrieve(RetrieveTaskError),
}

impl IntoResponse for TaskApiError {
    fn into_response(self) -> Response {
        warn!("task endpoint failed: {self:?}");
        let status = match self {
            // Report why the request could not be parsed, e.g. an unknown task type or invalid task parameters.
            Self::InvalidRequest(rejection) => return (StatusCode::BAD_REQUEST, rejection.body_text()).into_response(),
            // The request references wallet IDs that do not exist; this is a client error, not a missing resource.
            Self::Create(CreateTaskError::WalletIdsNotFound(_)) => StatusCode::BAD_REQUEST,
            Self::Retrieve(RetrieveTaskError::UnAuthorized)
            | Self::Create(CreateTaskError::MissingPrivilege(_))
            | Self::Review(ReviewTaskError::MissingPrivilege(_)) => StatusCode::FORBIDDEN,
            Self::Retrieve(RetrieveTaskError::NotFound(_))
            | Self::Withdraw(WithdrawTaskError::NotFound(_))
            | Self::Review(ReviewTaskError::NotFound(_))
            | Self::Execution(ExecuteTaskError::NotFound(_)) => StatusCode::NOT_FOUND,
            Self::Withdraw(WithdrawTaskError::InvalidState)
            | Self::Review(ReviewTaskError::InvalidState)
            | Self::Execution(ExecuteTaskError::InvalidState) => StatusCode::CONFLICT,
            Self::Retrieve(RetrieveTaskError::Storage(_) | RetrieveTaskError::AuditLog(_))
            | Self::Create(CreateTaskError::Storage(_) | CreateTaskError::AuditLog(_))
            | Self::Withdraw(WithdrawTaskError::Storage(_) | WithdrawTaskError::AuditLog(_))
            | Self::Review(ReviewTaskError::Storage(_) | ReviewTaskError::AuditLog(_))
            | Self::List(ListTasksError::Storage(_) | ListTasksError::AuditLog(_))
            | Self::Execution(ExecuteTaskError::Storage(_)) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        status.into_response()
    }
}

/// Failure of a `/wallets/check` endpoint, mapped to the matching HTTP status code.
#[derive(Debug)]
enum WalletApiError {
    Check(GetWalletRevocationStateError),
    InvalidRequest(JsonRejection),
}

impl IntoResponse for WalletApiError {
    fn into_response(self) -> Response {
        warn!("wallet endpoint failed: {self:?}");
        let status = match self {
            Self::InvalidRequest(rejection) => return (StatusCode::BAD_REQUEST, rejection.body_text()).into_response(),
            Self::Check(GetWalletRevocationStateError::MissingPrivilege(_)) => StatusCode::FORBIDDEN,
            Self::Check(GetWalletRevocationStateError::NotFound(_)) => StatusCode::NOT_FOUND,
            Self::Check(GetWalletRevocationStateError::Storage(_)) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        status.into_response()
    }
}

pub struct AdminPortalState {
    service: AdminPortalService<Repositories, TimeGenerator, UuidV7Generator>,
    user_state: Arc<ProductionUserState>,
    public_url: BaseUrl,
    audit_log: DbAuditLog<PostgresAuditLogRepository, UuidV7Generator>,
}

impl AdminPortalState {
    pub fn try_new(
        settings: AdminPortalSettings,
        repository: Repositories,
        audit_log: DbAuditLog<PostgresAuditLogRepository, UuidV7Generator>,
        user_state: Arc<ProductionUserState>,
    ) -> Result<Self, reqwest::Error> {
        let certs = settings
            .keycloak_trust_anchors
            .into_iter()
            .map(ReqwestTrustAnchor::into_certificate);
        let client = tls_pinned_client_builder(certs).build()?;

        let service = AdminPortalService::new(
            AdminPortalConfig {
                keycloak_url: settings.keycloak_url,
                keycloak_realm: settings.keycloak_realm,
                keycloak_client_id: settings.keycloak_client_id,
                public_url: settings.public_url.clone(),
                session_ttl: settings.session_ttl,
                login_attempt_ttl: settings.login_attempt_ttl,
            },
            client,
            repository,
            TimeGenerator,
            UuidV7Generator,
        );

        Ok(Self {
            service,
            user_state,
            public_url: settings.public_url,
            audit_log,
        })
    }

    fn frontend_url(&self, path: &str) -> Url {
        self.public_url.join(path)
    }
}

// TODO (PVW-6427): Add CSRF protection
pub fn router(state: Arc<AdminPortalState>) -> Router {
    Router::new()
        .route("/auth/login", get(login))
        .route("/auth/callback", get(callback))
        .route("/auth/logout", get(logout))
        .route("/api/me", get(me))
        .nest(
            "/tasks",
            Router::new()
                .route("/", post(create_task).get(list_tasks))
                .route("/{task_id}", get(retrieve_task))
                .route("/{task_id}/withdraw", post(withdraw_task))
                .route("/{task_id}/review", post(review_task)),
        )
        .route("/wallets/check", post(check_wallet))
        .with_state(state)
}

/// Returns a session cookie with [session_id].
/// This is also a session cookie in the sense that it has no `max_age`, so it will only expire when the browser is
/// closed. This does not decrease the security of the system, as the session is managed conservatively server side.
fn session_cookie(session_id: String) -> Cookie<'static> {
    Cookie::build((SESSION_ID_COOKIE, session_id))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .path("/")
        .into()
}

/// Redirects to the frontend's error page, so it can show the user a proper error message
/// instead of the SPA rendering a blank page or the browser rendering a raw JSON error.
fn error_redirect(state: &AdminPortalState, reason: &str) -> Response {
    let mut url = state.frontend_url("/error");
    url.query_pairs_mut().append_pair("reason", reason);

    Redirect::to(url.as_str()).into_response()
}

#[derive(Deserialize)]
struct LoginQuery {
    login_hint: Option<String>,
}

async fn login(State(state): State<Arc<AdminPortalState>>, Query(query): Query<LoginQuery>) -> Response {
    match state.service.authorization_url(query.login_hint.as_deref()).await {
        Ok(auth_url) => Redirect::to(auth_url.as_str()).into_response(),
        Err(error) => {
            warn!("/auth/login failed: {error}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[derive(Deserialize)]
struct CallbackQuery {
    state: Option<String>,
    code: Option<String>,
    error: Option<String>,
}

async fn callback(
    State(state): State<Arc<AdminPortalState>>,
    cookie_jar: CookieJar,
    Query(query): Query<CallbackQuery>,
) -> Response {
    let session_id = cookie_jar
        .get(SESSION_ID_COOKIE)
        .map(|cookie| cookie.value().to_string());

    let session_id = state
        .service
        .handle_callback(
            session_id.as_deref(),
            query.state.as_deref(),
            query.code.as_deref(),
            query.error.as_deref(),
        )
        .await
        .inspect_err(|error| warn!("/auth/callback failed: {error}"));

    let session_id = match session_id {
        Ok(session_id) => session_id,
        Err(error) => return error_redirect(&state, error.reason()),
    };

    let jar = CookieJar::new().add(session_cookie(session_id));
    let entry_url = state.frontend_url("/");

    (jar, Redirect::to(entry_url.as_str())).into_response()
}

async fn logout(State(state): State<Arc<AdminPortalState>>, cookie_jar: CookieJar) -> Response {
    let session_id = cookie_jar
        .get(SESSION_ID_COOKIE)
        .map(|cookie| cookie.value().to_string());
    let cookie_jar = cookie_jar.remove(Cookie::build(SESSION_ID_COOKIE).path("/"));

    if let Some(session_id) = session_id
        && let Err(error) = state.service.end_session(&session_id).await
    {
        warn!("/auth/logout failed: {error}");
        // redirect to error page, with the session cookie removed
        return (cookie_jar, error_redirect(&state, &error.to_string())).into_response();
    }

    let redirect_url = state.frontend_url("/");
    (cookie_jar, Redirect::to(redirect_url.as_str())).into_response()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MeResponse {
    display_name: String,
    privileges: Vec<String>,
}

impl From<LoggedInUser> for MeResponse {
    fn from(
        LoggedInUser {
            display_name,
            privileges,
            ..
        }: LoggedInUser,
    ) -> Self {
        Self {
            display_name,
            privileges,
        }
    }
}

async fn me(AuthenticatedAdminUser(user): AuthenticatedAdminUser) -> Response {
    Json(MeResponse::from(user)).into_response()
}

/// Extracts the logged-in Admin Portal user from the session cookie, rejecting with `401  Unauthorized` when there is
/// no valid session. Shared by every endpoint under `/admin-portal` that requires a logged-in user.
#[derive(Debug)]
pub struct AuthenticatedAdminUser(pub LoggedInUser);

impl FromRequestParts<Arc<AdminPortalState>> for AuthenticatedAdminUser {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &Arc<AdminPortalState>) -> Result<Self, Self::Rejection> {
        let cookie_jar = CookieJar::from_headers(&parts.headers);

        // Retrieve the session_id from the cookie, note that we don't need to touch the cookie's `max_age`, as it is a
        // session cookie, see the documentation at `fn session_cookie`.
        let Some(session_id) = cookie_jar.get(SESSION_ID_COOKIE).map(|cookie| cookie.value()) else {
            return Err(StatusCode::UNAUTHORIZED.into_response());
        };

        match state.service.authenticated_user(session_id).await {
            Ok(Some(user)) => Ok(AuthenticatedAdminUser(user)),
            Ok(None) => Err(StatusCode::UNAUTHORIZED.into_response()),
            Err(error) => {
                warn!("session lookup failed: {error}");
                Err(StatusCode::INTERNAL_SERVER_ERROR.into_response())
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "web_test", derive(Serialize))]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskRequest {
    #[serde(flatten)]
    pub payload: TaskPayload,
    pub task_reason: String,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "web_test", derive(Serialize))]
#[serde(rename_all = "camelCase")]
pub struct WithdrawTaskRequest {
    pub withdrawal_comment: String,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "web_test", derive(Serialize))]
#[serde(rename_all = "camelCase")]
pub struct ReviewTaskRequest {
    pub verdict: ReviewVerdict,
    pub review_comment: String,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "web_test", derive(Deserialize))]
#[serde(rename_all = "camelCase")]
pub struct TaskResponse {
    pub task_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    #[serde(flatten)]
    pub payload: TaskPayload,
    pub task_reason: String,
    pub status: TaskStatus,
    pub withdrawal_comment: Option<String>,
    pub is_own_task: bool,
    pub withdrawn_at: Option<DateTime<Utc>>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub reviewed_by: Option<String>,
    pub review_comment: Option<String>,
    pub executed_at: Option<DateTime<Utc>>,
    pub execution_successful: Option<bool>,
    pub execution_details: Option<String>,
}

impl From<(&LoggedInUser, AdminTask)> for TaskResponse {
    fn from((user, task): (&LoggedInUser, AdminTask)) -> Self {
        Self {
            task_id: task.id,
            created_at: task.created_at,
            created_by: task.created_by_display_name,
            payload: task.payload,
            task_reason: task.task_reason,
            status: task.status,
            withdrawal_comment: task.withdrawal_comment,
            is_own_task: user.user_id == task.created_by_user_id,
            withdrawn_at: task.withdrawn_at,
            reviewed_at: task.reviewed_at,
            reviewed_by: task.reviewed_by_display_name,
            review_comment: task.review_comment,
            executed_at: task.executed_at,
            execution_successful: task.execution_successful,
            execution_details: task.execution_details,
        }
    }
}

async fn create_task(
    State(state): State<Arc<AdminPortalState>>,
    AuthenticatedAdminUser(user): AuthenticatedAdminUser,
    request: Result<Json<CreateTaskRequest>, JsonRejection>,
) -> Result<impl IntoResponse, TaskApiError> {
    let Json(request) = request.map_err(TaskApiError::InvalidRequest)?;
    let task = state
        .service
        .create_task(
            &request.payload,
            &request.task_reason,
            &user,
            &state.user_state,
            &state.audit_log,
        )
        .await
        .map_err(TaskApiError::Create)?;

    Ok((StatusCode::CREATED, Json(TaskResponse::from((&user, task)))))
}

async fn withdraw_task(
    State(state): State<Arc<AdminPortalState>>,
    AuthenticatedAdminUser(user): AuthenticatedAdminUser,
    Path(task_id): Path<Uuid>,
    Json(payload): Json<WithdrawTaskRequest>,
) -> Result<Json<TaskResponse>, TaskApiError> {
    let task = state
        .service
        .withdraw_task(task_id, &payload.withdrawal_comment, &user, &state.audit_log)
        .await
        .map_err(TaskApiError::Withdraw)?;

    Ok(Json(TaskResponse::from((&user, task))))
}

async fn review_task(
    State(state): State<Arc<AdminPortalState>>,
    AuthenticatedAdminUser(user): AuthenticatedAdminUser,
    Path(task_id): Path<Uuid>,
    Json(payload): Json<ReviewTaskRequest>,
) -> Result<Json<TaskResponse>, TaskApiError> {
    let task = state
        .service
        .review_task(
            task_id,
            payload.verdict,
            &payload.review_comment,
            &user,
            &state.audit_log,
        )
        .await
        .map_err(TaskApiError::Review)?;

    let task = state
        .service
        .execute_task(task.id, &user, &state.user_state, &state.audit_log)
        .await
        .map_err(TaskApiError::Execution)?;

    Ok(Json(TaskResponse::from((&user, task))))
}

async fn retrieve_task(
    State(state): State<Arc<AdminPortalState>>,
    AuthenticatedAdminUser(user): AuthenticatedAdminUser,
    Path(task_id): Path<Uuid>,
) -> Result<Json<TaskResponse>, TaskApiError> {
    let task = state
        .service
        .retrieve_task(task_id, &user, &state.audit_log)
        .await
        .map_err(TaskApiError::Retrieve)?;

    Ok(Json(TaskResponse::from((&user, task))))
}

#[derive(Debug, Deserialize)]
struct ListTasksQuery {
    view: TaskListViewKind,
}

async fn list_tasks(
    State(state): State<Arc<AdminPortalState>>,
    AuthenticatedAdminUser(user): AuthenticatedAdminUser,
    Query(query): Query<ListTasksQuery>,
) -> Result<Json<Vec<TaskResponse>>, TaskApiError> {
    let tasks = state
        .service
        .list_tasks(query.view, &user, &state.audit_log)
        .await
        .map_err(TaskApiError::List)?;

    Ok(Json(
        tasks
            .into_iter()
            .map(|task| TaskResponse::from((&user, task)))
            .collect_vec(),
    ))
}

#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "web_test", derive(Serialize))]
#[serde(rename_all = "camelCase")]
pub struct CheckWalletRequest {
    pub wallet_id: WalletId,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "web_test", derive(Deserialize))]
#[serde(rename_all = "camelCase")]
pub struct CheckWalletResponse {
    pub status: WalletRevocationState,
}

async fn check_wallet(
    State(state): State<Arc<AdminPortalState>>,
    AuthenticatedAdminUser(user): AuthenticatedAdminUser,
    request: Result<Json<CheckWalletRequest>, JsonRejection>,
) -> Result<Json<CheckWalletResponse>, WalletApiError> {
    let Json(request) = request.map_err(WalletApiError::InvalidRequest)?;

    let status = state
        .service
        .get_wallet_revocation_state(&request.wallet_id, &user, &state.user_state)
        .await
        .map_err(WalletApiError::Check)?;

    Ok(Json(CheckWalletResponse { status }))
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use axum::extract::rejection::MissingJsonContentType;
    use rstest::rstest;
    use utils::vec_nonempty;
    use wallet_provider_domain::model::admin_task::RevokeWalletParameters;
    use wallet_provider_domain::model::admin_task::TaskType;
    use wallet_provider_domain::repository::PersistenceError;

    use super::*;

    fn storage_error() -> PersistenceError {
        PersistenceError::NoRowsUpdated
    }

    fn audit_log_error() -> Box<dyn std::error::Error + Send + Sync> {
        Box::new(PersistenceError::NoRowsUpdated)
    }

    fn revoke_wallet_payload() -> TaskPayload {
        TaskPayload::RevokeWallet(RevokeWalletParameters {
            wallet_ids: vec_nonempty!["wallet-1".to_string().into()].into(),
        })
    }

    #[test]
    fn session_cookie_is_secure_http_only_strict_and_session_only() {
        let cookie = session_cookie("the-session-id".to_string());

        assert_eq!(cookie.name(), SESSION_ID_COOKIE);
        assert_eq!(cookie.value(), "the-session-id");
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.secure(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Strict));
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.max_age(), None);
    }

    #[test]
    fn task_response_maps_display_names_not_user_ids() {
        let user = LoggedInUser {
            user_id: "user-1".to_string(),
            display_name: "User One".to_string(),
            privileges: vec![],
        };
        let task = AdminTask {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            created_by_user_id: "user-1".to_string(),
            created_by_display_name: "User One".to_string(),
            payload: revoke_wallet_payload(),
            task_reason: "reason".to_string(),
            status: TaskStatus::Created,
            withdrawal_comment: None,
            withdrawn_at: None,
            reviewed_at: None,
            reviewed_by_user_id: None,
            reviewed_by_display_name: None,
            review_comment: None,
            executed_at: None,
            execution_successful: None,
            execution_details: None,
        };

        let response = TaskResponse::from((&user, task));

        assert_eq!(response.created_by, "User One");
    }

    #[rstest]
    #[case::invalid_request(
        TaskApiError::InvalidRequest(JsonRejection::MissingJsonContentType(MissingJsonContentType::default())),
        StatusCode::BAD_REQUEST
    )]
    #[case::create_missing_privilege(
        TaskApiError::Create(CreateTaskError::MissingPrivilege(TaskType::RevokeWallet)),
        StatusCode::FORBIDDEN
    )]
    #[case::create_wallet_ids_not_found(
        TaskApiError::Create(CreateTaskError::WalletIdsNotFound(HashSet::from([WalletId::from(
            "wallet-1".to_string()
        )]))),
        StatusCode::BAD_REQUEST
    )]
    #[case::create_storage(
        TaskApiError::Create(CreateTaskError::Storage(storage_error())),
        StatusCode::INTERNAL_SERVER_ERROR
    )]
    #[case::create_audit_log(
        TaskApiError::Create(CreateTaskError::AuditLog(audit_log_error())),
        StatusCode::INTERNAL_SERVER_ERROR
    )]
    #[case::withdraw_not_found(
        TaskApiError::Withdraw(WithdrawTaskError::NotFound(Uuid::new_v4())),
        StatusCode::NOT_FOUND
    )]
    #[case::withdraw_invalid_state(TaskApiError::Withdraw(WithdrawTaskError::InvalidState), StatusCode::CONFLICT)]
    #[case::withdraw_storage(
        TaskApiError::Withdraw(WithdrawTaskError::Storage(storage_error())),
        StatusCode::INTERNAL_SERVER_ERROR
    )]
    #[case::withdraw_audit_log(
        TaskApiError::Withdraw(WithdrawTaskError::AuditLog(audit_log_error())),
        StatusCode::INTERNAL_SERVER_ERROR
    )]
    #[case::review_not_found(
        TaskApiError::Review(ReviewTaskError::NotFound(Uuid::new_v4())),
        StatusCode::NOT_FOUND
    )]
    #[case::review_missing_privilege(
        TaskApiError::Review(ReviewTaskError::MissingPrivilege(TaskType::RevokeWallet)),
        StatusCode::FORBIDDEN
    )]
    #[case::review_invalid_state(TaskApiError::Review(ReviewTaskError::InvalidState), StatusCode::CONFLICT)]
    #[case::review_storage(
        TaskApiError::Review(ReviewTaskError::Storage(storage_error())),
        StatusCode::INTERNAL_SERVER_ERROR
    )]
    #[case::review_audit_log(
        TaskApiError::Review(ReviewTaskError::AuditLog(audit_log_error())),
        StatusCode::INTERNAL_SERVER_ERROR
    )]
    #[case::list_storage(
        TaskApiError::List(ListTasksError::Storage(storage_error())),
        StatusCode::INTERNAL_SERVER_ERROR
    )]
    #[case::list_audit_log(
        TaskApiError::List(ListTasksError::AuditLog(audit_log_error())),
        StatusCode::INTERNAL_SERVER_ERROR
    )]
    #[case::execution_not_found(
        TaskApiError::Execution(ExecuteTaskError::NotFound(Uuid::new_v4())),
        StatusCode::NOT_FOUND
    )]
    #[case::execution_invalid_state(TaskApiError::Execution(ExecuteTaskError::InvalidState), StatusCode::CONFLICT)]
    #[case::execution_storage(
        TaskApiError::Execution(ExecuteTaskError::Storage(storage_error())),
        StatusCode::INTERNAL_SERVER_ERROR
    )]
    fn task_api_error_status_codes(#[case] error: TaskApiError, #[case] status: StatusCode) {
        assert_eq!(error.into_response().status(), status);
    }

    #[rstest]
    #[case::missing_privilege(
        WalletApiError::Check(GetWalletRevocationStateError::MissingPrivilege(TaskType::RevokeWallet)),
        StatusCode::FORBIDDEN
    )]
    #[case::not_found(
        WalletApiError::Check(GetWalletRevocationStateError::NotFound(WalletId::from("wallet-1".to_string()))),
        StatusCode::NOT_FOUND
    )]
    #[case::storage(
        WalletApiError::Check(GetWalletRevocationStateError::Storage(storage_error())),
        StatusCode::INTERNAL_SERVER_ERROR
    )]
    fn wallet_api_error_status_codes(#[case] error: WalletApiError, #[case] status: StatusCode) {
        assert_eq!(error.into_response().status(), status);
    }
}

#[cfg(all(test, feature = "db_test", feature = "hsm_test"))]
mod http_tests {
    use axum::body::Body;
    use axum::body::to_bytes;
    use axum::http::Request;
    use axum::http::header;
    use chrono::TimeDelta;
    use crypto::utils::random_string;
    use serde_json::Value;
    use serde_json::json;
    use tower::ServiceExt;
    use wallet_provider_domain::model::admin_portal_session::AdminPortalUserSession;
    use wallet_provider_domain::model::admin_task::TaskType;
    use wallet_provider_domain::repository::AdminPortalRepository;

    use super::*;
    use crate::test_support::TestState;
    use crate::test_support::setup_state;

    async fn setup_state_with_database_errors() -> TestState {
        let state = setup_state().await;
        state.repositories.as_ref().to_connection().close().await.unwrap();
        state
    }

    async fn setup_state_with_valid_user() -> (TestState, CookieJar) {
        let state = setup_state().await;
        let session_id = random_string(12);
        state
            .repositories
            .insert_user_session(
                session_id.clone(),
                AdminPortalUserSession {
                    user_id: "user_id".to_string(),
                    display_name: "Jane Doe".to_string(),
                    roles: vec!["privilege_admin".to_string(), "offline_access".to_string()],
                    id_token: "the-id-token".to_string(),
                    expires_at: Utc::now() + TimeDelta::minutes(5),
                },
            )
            .await
            .unwrap();
        let jar = CookieJar::new().add(Cookie::new(SESSION_ID_COOKIE, session_id));
        (state, jar)
    }

    #[serial_test::serial(hsm)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn login_reports_server_error_when_oidc_provider_is_unreachable() {
        let state = setup_state().await;

        let response = login(State(Arc::clone(&state)), Query(LoginQuery { login_hint: None })).await;

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[serial_test::serial(hsm)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn callback_without_state_redirects_to_frontend_error_page() {
        let state = setup_state().await;

        let response = callback(
            State(Arc::clone(&state)),
            CookieJar::new(),
            Query(CallbackQuery {
                state: None,
                code: None,
                error: None,
            }),
        )
        .await;

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            "https://portal.example.org/error?reason=invalid_state"
        );
    }

    #[serial_test::serial(hsm)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn callback_without_state_but_with_session_redirects_to_frontend_entry() {
        let (state, jar) = setup_state_with_valid_user().await;

        let response = callback(
            State(Arc::clone(&state)),
            jar,
            Query(CallbackQuery {
                state: Some("unknown-state".to_string()),
                code: None,
                error: None,
            }),
        )
        .await;

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            "https://portal.example.org/"
        );
    }

    #[serial_test::serial(hsm)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn logout_without_session_cookie_redirects_to_frontend_entry() {
        let state = setup_state().await;

        let response = logout(State(Arc::clone(&state)), CookieJar::new()).await;

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            "https://portal.example.org/"
        );
    }

    #[serial_test::serial(hsm)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn logout_redirects_to_error_page_for_database_errors() {
        let state = setup_state_with_database_errors().await;
        let cookie_jar = CookieJar::new().add(Cookie::new(SESSION_ID_COOKIE, "my_session"));

        let response = logout(State(Arc::clone(&state)), cookie_jar).await;

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let location = Url::parse(response.headers().get(header::LOCATION).unwrap().to_str().unwrap()).unwrap();
        assert_eq!(location.path(), "/error");
        assert!(
            location
                .query_pairs()
                .any(|(key, value)| key == "reason" && !value.is_empty())
        );
    }

    fn request_parts(cookie_jar: &CookieJar) -> Parts {
        let mut request = Request::new(());
        for cookie in cookie_jar.iter() {
            request.headers_mut().append(
                header::COOKIE,
                format!("{}={}", cookie.name(), cookie.value()).parse().unwrap(),
            );
        }
        request.into_parts().0
    }

    async fn json_body<T: serde::de::DeserializeOwned>(response: Response) -> T {
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    #[serial_test::serial(hsm)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn authenticated_admin_user_without_session_cookie_is_rejected_with_401() {
        let state = setup_state().await;
        let mut parts = request_parts(&CookieJar::new());

        let response = AuthenticatedAdminUser::from_request_parts(&mut parts, &state)
            .await
            .expect_err("missing session cookie should be rejected");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[serial_test::serial(hsm)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn authenticated_admin_user_with_unknown_session_cookie_is_rejected_with_401() {
        let state = setup_state().await;
        let jar = CookieJar::new().add(Cookie::new(SESSION_ID_COOKIE, "unknown-session"));
        let mut parts = request_parts(&jar);

        let response = AuthenticatedAdminUser::from_request_parts(&mut parts, &state)
            .await
            .expect_err("unknown session should be rejected");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[serial_test::serial(hsm)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn authenticated_admin_user_reports_server_error_for_database_errors() {
        let state = setup_state_with_database_errors().await;
        let jar = CookieJar::new().add(Cookie::new(SESSION_ID_COOKIE, "my-session"));
        let mut parts = request_parts(&jar);

        let response = AuthenticatedAdminUser::from_request_parts(&mut parts, &state)
            .await
            .expect_err("database errors should reject extraction");

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[serial_test::serial(hsm)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn me_with_valid_session_reports_logged_in_user() {
        let (state, jar) = setup_state_with_valid_user().await;
        let mut parts = request_parts(&jar);
        let AuthenticatedAdminUser(user) = AuthenticatedAdminUser::from_request_parts(&mut parts, &state)
            .await
            .expect("valid session should be extracted");

        assert_eq!(user.user_id, "user_id");
        assert_eq!(user.display_name, "Jane Doe");
        assert_eq!(user.privileges, vec!["admin"]);

        let response = me(AuthenticatedAdminUser(user)).await;

        assert_eq!(response.status(), StatusCode::OK);
        assert!(!response.headers().contains_key(header::SET_COOKIE));
        let json: Value = json_body(response).await;
        assert_eq!(
            json,
            serde_json::json!({"displayName": "Jane Doe", "privileges": ["admin"]})
        );
    }

    #[rstest::rstest]
    #[case::create("POST", "/tasks")]
    #[case::list("GET", "/tasks?view=open")]
    #[case::withdraw("POST", "/tasks/00000000-0000-0000-0000-000000000000/withdraw")]
    #[case::review("POST", "/tasks/00000000-0000-0000-0000-000000000000/review")]
    #[case::check_wallet("POST", "/wallets/check")]
    #[serial_test::serial(hsm)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn admin_portal_routes_require_authentication(#[case] method: &str, #[case] uri: &str) {
        let state = setup_state().await;
        let request = Request::builder().method(method).uri(uri).body(Body::empty()).unwrap();

        let response = router(Arc::clone(&state)).oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    async fn create_task_from_body(body: Value) -> Response {
        let state = setup_state().await;
        create_task(
            State(Arc::clone(&state)),
            AuthenticatedAdminUser(LoggedInUser {
                user_id: "alice".to_owned(),
                display_name: "Alice".to_owned(),
                privileges: vec![TaskType::RevokeWallet.to_string()],
            }),
            Json::from_bytes(&serde_json::to_vec(&body).unwrap()),
        )
        .await
        .into_response()
    }

    #[rstest::rstest]
    #[case::missing_task_reason(json!({"taskType": "revoke_wallet", "taskParameters": {"walletIds": ["wallet-1"]}}))]
    #[case::not_an_object(json!("revoke_wallet"))]
    #[serial_test::serial(hsm)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn create_task_rejects_invalid_request_with_bad_request(#[case] body: Value) {
        let response = create_task_from_body(body).await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[serial_test::serial(hsm)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn create_task_rejects_unknown_wallet_ids_with_bad_request() {
        let response = create_task_from_body(json!({
            "taskType": "revoke_wallet",
            "taskParameters": {"walletIds": ["wallet-1"]},
            "taskReason": "because",
        }))
        .await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    async fn check_wallet_with(wallet_id: &str, privileges: Vec<String>) -> Response {
        let state = setup_state().await;
        check_wallet(
            State(Arc::clone(&state)),
            AuthenticatedAdminUser(LoggedInUser {
                user_id: "alice".to_owned(),
                display_name: "Alice".to_owned(),
                privileges,
            }),
            Ok(Json(CheckWalletRequest {
                wallet_id: WalletId::from(wallet_id.to_string()),
            })),
        )
        .await
        .into_response()
    }

    #[serial_test::serial(hsm)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn check_wallet_rejects_missing_privilege_with_forbidden() {
        let response = check_wallet_with("wallet-1", vec![]).await;

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[serial_test::serial(hsm)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn check_wallet_reports_unknown_wallet_with_not_found() {
        let response = check_wallet_with("wallet-1", vec![TaskType::RevokeWallet.to_string()]).await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
