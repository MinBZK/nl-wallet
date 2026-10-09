//! HTTP layer for the Admin Portal's OIDC login flow. The actual OIDC/session logic lives in
//! [`wallet_provider_service::admin_portal::AdminPortalService`]; this module only wires it up to
//! axum: routing, query/cookie extraction and turning results into redirects or JSON responses.
//!
//! `/auth/login` redirects to Keycloak (with PKCE), `/auth/callback` exchanges the resulting
//! authorization code for tokens and starts a server-side session, `/auth/logout` ends both the
//! local session and the Keycloak SSO session, and `/api/me` reports the logged-in user to the
//! SPA based on the session cookie.

use std::sync::Arc;

use audit_log::model::DbAuditLog;
use audit_log::repository::AuditLogRepository;
use axum::Json;
use axum::Router;
use axum::extract::FromRequestParts;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
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
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value as JsonValue;
use tracing::warn;
use url::Url;
use utils::generator::TimeGenerator;
use uuid::Uuid;
use wallet_provider_domain::model::admin_task::AdminTask;
use wallet_provider_domain::model::admin_task::TaskStatus;
use wallet_provider_domain::model::admin_task::TaskType;
use wallet_provider_domain::repository::AdminPortalSessionRepository;
use wallet_provider_domain::repository::AdminTaskRepository;
use wallet_provider_service::admin_portal;
use wallet_provider_service::admin_portal::AdminPortalConfig;
use wallet_provider_service::admin_portal::AdminPortalService;
use wallet_provider_service::admin_portal::LoggedInUser;
use wallet_provider_service::admin_portal::task::CreateTaskError;
use wallet_provider_service::admin_portal::task::WithdrawTaskError;

use crate::router_state::UuidV7Generator;
use crate::settings::AdminPortalSettings;

const SESSION_ID_COOKIE: &str = "sid";

struct CreateTaskApiError(CreateTaskError);

impl IntoResponse for CreateTaskApiError {
    fn into_response(self) -> Response {
        let CreateTaskApiError(error) = self;
        warn!("create_task failed: {error:?}");
        match error {
            CreateTaskError::UnknownTaskType(_) => StatusCode::BAD_REQUEST.into_response(),
            CreateTaskError::MissingPrivilege(_) => StatusCode::FORBIDDEN.into_response(),
            CreateTaskError::Storage(_) | CreateTaskError::AuditLog(_) => {
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

struct WithdrawTaskApiError(WithdrawTaskError);

impl IntoResponse for WithdrawTaskApiError {
    fn into_response(self) -> Response {
        let Self(error) = self;
        warn!("withdraw_task failed: {error:?}");
        match error {
            WithdrawTaskError::NotFound(_) => StatusCode::NOT_FOUND.into_response(),
            WithdrawTaskError::InvalidState => StatusCode::CONFLICT.into_response(),
            WithdrawTaskError::Storage(_) | WithdrawTaskError::AuditLog(_) => {
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

pub struct AdminPortalState<R, AR> {
    service: AdminPortalService<R>,
    public_url: BaseUrl,
    audit_log: DbAuditLog<AR, UuidV7Generator>,
}

impl<R, AR> AdminPortalState<R, AR> {
    pub fn try_new(
        settings: AdminPortalSettings,
        repository: R,
        audit_log: DbAuditLog<AR, UuidV7Generator>,
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
        );

        Ok(Self {
            service,
            public_url: settings.public_url,
            audit_log,
        })
    }

    fn frontend_url(&self, path: &str) -> Url {
        self.public_url.join(path)
    }
}

pub fn router<R, AR>(state: Arc<AdminPortalState<R, AR>>) -> Router
where
    R: AdminPortalSessionRepository + AdminTaskRepository + Send + Sync + 'static,
    AR: AuditLogRepository + Send + Sync + 'static,
{
    Router::new()
        .route("/auth/login", get(login))
        .route("/auth/callback", get(callback))
        .route("/auth/logout", get(logout))
        .route("/api/me", get(me))
        .nest(
            "/tasks",
            Router::new()
                .route("/", post(create_task))
                .route("/{task_id}/withdraw", post(withdraw_task)),
        )
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
fn error_redirect<R, AR>(state: &AdminPortalState<R, AR>, reason: &str) -> Response {
    let mut url = state.frontend_url("/error");
    url.query_pairs_mut().append_pair("reason", reason);

    Redirect::to(url.as_str()).into_response()
}

#[derive(Deserialize)]
struct LoginQuery {
    login_hint: Option<String>,
}

async fn login<R, AR>(State(state): State<Arc<AdminPortalState<R, AR>>>, Query(query): Query<LoginQuery>) -> Response
where
    R: AdminPortalSessionRepository + Send + Sync + 'static,
    AR: Send + Sync + 'static,
{
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

async fn callback<R, AR>(
    State(state): State<Arc<AdminPortalState<R, AR>>>,
    cookie_jar: CookieJar,
    Query(query): Query<CallbackQuery>,
) -> Response
where
    R: AdminPortalSessionRepository + Send + Sync + 'static,
    AR: Send + Sync + 'static,
{
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

async fn logout<R, AR>(State(state): State<Arc<AdminPortalState<R, AR>>>, cookie_jar: CookieJar) -> Response
where
    R: AdminPortalSessionRepository + Send + Sync + 'static,
    AR: Send + Sync + 'static,
{
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

impl<R, AR> FromRequestParts<Arc<AdminPortalState<R, AR>>> for AuthenticatedAdminUser
where
    R: AdminPortalSessionRepository + Send + Sync + 'static,
    AR: AuditLogRepository + Send + Sync + 'static,
{
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AdminPortalState<R, AR>>,
    ) -> Result<Self, Self::Rejection> {
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
#[serde(rename_all = "camelCase")]
struct CreateTaskRequest {
    task_type: String,
    #[serde(default = "serde_json::Map::new")]
    task_parameters: serde_json::Map<String, serde_json::Value>,
    task_reason: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TaskResponse {
    task_id: Uuid,
    created_at: DateTime<Utc>,
    created_by: String,
    task_type: TaskType,
    task_parameters: JsonValue,
    task_reason: String,
    status: TaskStatus,
    reviewed_at: Option<DateTime<Utc>>,
    reviewed_by: Option<String>,
    review_comment: Option<String>,
    executed_at: Option<DateTime<Utc>>,
    execution_result: Option<String>,
    execution_result_details: Option<JsonValue>,
}

impl From<AdminTask> for TaskResponse {
    fn from(task: AdminTask) -> Self {
        Self {
            task_id: task.id,
            created_at: task.created_at,
            created_by: task.created_by_display_name,
            task_type: task.task_type,
            task_parameters: task.task_parameters,
            task_reason: task.task_reason,
            status: task.status,
            reviewed_at: task.reviewed_at,
            reviewed_by: task.reviewed_by_display_name,
            review_comment: task.review_comment,
            executed_at: task.executed_at,
            execution_result: task.execution_result,
            execution_result_details: task.execution_result_details,
        }
    }
}

async fn create_task<R, AR>(
    State(state): State<Arc<AdminPortalState<R, AR>>>,
    AuthenticatedAdminUser(user): AuthenticatedAdminUser,
    Json(payload): Json<CreateTaskRequest>,
) -> Response
where
    R: AdminPortalSessionRepository + AdminTaskRepository + Send + Sync + 'static,
    AR: AuditLogRepository + Send + Sync + 'static,
{
    let result = admin_portal::task::create_task(
        &payload.task_type,
        &JsonValue::Object(payload.task_parameters),
        &payload.task_reason,
        &user.user_id,
        &user.display_name,
        &user.privileges,
        state.service.repository(),
        &TimeGenerator,
        &UuidV7Generator,
        &state.audit_log,
    )
    .await
    .map_err(CreateTaskApiError);

    match result {
        Ok(task) => (StatusCode::CREATED, Json(TaskResponse::from(task))).into_response(),
        Err(error) => error.into_response(),
    }
}

async fn withdraw_task<R, AR>(
    State(state): State<Arc<AdminPortalState<R, AR>>>,
    AuthenticatedAdminUser(user): AuthenticatedAdminUser,
    Path(task_id): Path<Uuid>,
) -> Response
where
    R: AdminPortalSessionRepository + AdminTaskRepository + Send + Sync + 'static,
    AR: AuditLogRepository + Send + Sync + 'static,
{
    match admin_portal::task::withdraw_task(
        task_id,
        &user.user_id,
        state.service.repository(),
        &TimeGenerator,
        &state.audit_log,
    )
    .await
    {
        Ok(task) => Json(TaskResponse::from(task)).into_response(),
        Err(error) => WithdrawTaskApiError(error).into_response(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::time::Duration;

    use audit_log::model::DbAuditLog;
    use audit_log::repository::mock::NoopAuditLogRepository;
    use axum::body::Body;
    use axum::body::to_bytes;
    use axum::http::Request;
    use axum::http::StatusCode;
    use axum::http::header;
    use chrono::DateTime;
    use chrono::TimeDelta;
    use chrono::Utc;
    use crypto::utils::random_string;
    use http_utils::reqwest::test::get_test_trust_anchor;
    use serde_json::Value;
    use tower::ServiceExt;
    use utils::generator::TimeGenerator;
    use utils::vec_nonempty;
    use wallet_provider_domain::model::admin_portal_session::AdminPortalLoginAttempt;
    use wallet_provider_domain::model::admin_portal_session::AdminPortalUserSession;
    use wallet_provider_domain::model::admin_task::AdminTaskCreate;
    use wallet_provider_domain::model::admin_task::AdminTaskReview;
    use wallet_provider_domain::model::admin_task::TaskListView;
    use wallet_provider_domain::repository::AdminPortalSessionRepository;
    use wallet_provider_domain::repository::PersistenceError;

    use super::*;
    use crate::router_state::UuidV7Generator;

    #[test]
    fn create_task_error_status_codes() {
        assert_eq!(
            CreateTaskApiError(CreateTaskError::UnknownTaskType("x".to_string()))
                .into_response()
                .status(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            CreateTaskApiError(CreateTaskError::MissingPrivilege(TaskType::RevokeWallet))
                .into_response()
                .status(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            CreateTaskApiError(CreateTaskError::Storage(PersistenceError::NoRowsUpdated))
                .into_response()
                .status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
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

    /// A minimal in-memory [`AdminPortalSessionRepository`], so the HTTP layer can be tested without a database.
    #[derive(Default)]
    struct InMemoryRepository {
        sessions: Mutex<HashMap<String, AdminPortalUserSession>>,
        tasks: Mutex<HashMap<Uuid, AdminTask>>,
        errors: bool,
    }

    impl InMemoryRepository {
        fn set_errors(&mut self, errors: bool) {
            self.errors = errors;
        }

        fn handle_error(&self) -> Result<(), PersistenceError> {
            if self.errors {
                Err(PersistenceError::NoRowsUpdated)
            } else {
                Ok(())
            }
        }
    }

    impl AdminPortalSessionRepository for InMemoryRepository {
        async fn insert_login_attempt(
            &self,
            _state: String,
            _login_attempt: AdminPortalLoginAttempt,
        ) -> Result<(), PersistenceError> {
            self.handle_error()?;
            Ok(())
        }

        async fn take_login_attempt(&self, _state: &str) -> Result<Option<AdminPortalLoginAttempt>, PersistenceError> {
            self.handle_error()?;
            Ok(None)
        }

        async fn cleanup_expired_login_attempts(&self, _created_before: DateTime<Utc>) -> Result<(), PersistenceError> {
            self.handle_error()?;
            Ok(())
        }

        async fn insert_user_session(
            &self,
            session_id: String,
            session: AdminPortalUserSession,
        ) -> Result<(), PersistenceError> {
            self.handle_error()?;
            self.sessions.lock().unwrap().insert(session_id, session);
            Ok(())
        }

        async fn fetch_user_session(
            &self,
            session_id: &str,
            now: DateTime<Utc>,
            new_expires_at: Option<DateTime<Utc>>,
        ) -> Result<Option<AdminPortalUserSession>, PersistenceError> {
            self.handle_error()?;
            let mut sessions = self.sessions.lock().unwrap();
            let Some(session) = sessions.get_mut(session_id) else {
                return Ok(None);
            };
            if session.expires_at <= now {
                return Ok(None);
            }
            if let Some(new_expires_at) = new_expires_at {
                session.expires_at = new_expires_at;
            }
            Ok(Some(session.clone()))
        }

        async fn take_user_session(
            &self,
            session_id: &str,
        ) -> Result<Option<AdminPortalUserSession>, PersistenceError> {
            self.handle_error()?;
            Ok(self.sessions.lock().unwrap().remove(session_id))
        }

        async fn cleanup_expired_user_sessions(&self, _now: DateTime<Utc>) -> Result<(), PersistenceError> {
            self.handle_error()?;
            Ok(())
        }
    }

    impl AdminTaskRepository for InMemoryRepository {
        async fn insert_task(&self, task: AdminTaskCreate) -> Result<AdminTask, PersistenceError> {
            self.handle_error()?;
            let task: AdminTask = task.into();
            self.tasks.lock().unwrap().insert(task.id, task.clone());
            Ok(task)
        }

        async fn find_task_by_id(&self, task_id: Uuid) -> Result<Option<AdminTask>, PersistenceError> {
            self.handle_error()?;
            Ok(self.tasks.lock().unwrap().get(&task_id).cloned())
        }

        async fn withdraw_task(
            &self,
            task_id: Uuid,
            user_id: &str,
            now: DateTime<Utc>,
        ) -> Result<Option<AdminTask>, PersistenceError> {
            self.handle_error()?;
            let mut tasks = self.tasks.lock().unwrap();
            let Some(task) = tasks.get_mut(&task_id) else {
                return Ok(None);
            };
            if task.created_by_user_id != user_id || task.status != TaskStatus::Created {
                return Ok(None);
            }
            task.status = TaskStatus::Withdrawn;
            task.reviewed_at = Some(now);
            Ok(Some(task.clone()))
        }

        async fn review_task(&self, _review: AdminTaskReview) -> Result<Option<AdminTask>, PersistenceError> {
            Err(PersistenceError::NoRowsUpdated)
        }

        async fn list_tasks(&self, _view: TaskListView) -> Result<Vec<AdminTask>, PersistenceError> {
            Err(PersistenceError::NoRowsUpdated)
        }
    }

    fn setup_state() -> Arc<AdminPortalState<InMemoryRepository, NoopAuditLogRepository>> {
        setup_state_with_repository(InMemoryRepository::default())
    }

    fn setup_state_with_repository(
        repository: InMemoryRepository,
    ) -> Arc<AdminPortalState<InMemoryRepository, NoopAuditLogRepository>> {
        Arc::new(
            AdminPortalState::try_new(
                AdminPortalSettings {
                    // .invalid is reserved by the IETF and will never resolve
                    keycloak_url: "https://keycloak.invalid".parse().unwrap(),
                    keycloak_realm: "test-realm".to_string(),
                    keycloak_client_id: "test-client".to_string(),
                    public_url: "https://portal.example.org/".parse().unwrap(),
                    keycloak_trust_anchors: vec_nonempty![get_test_trust_anchor()],
                    session_ttl: Duration::from_mins(15),
                    login_attempt_ttl: Duration::from_mins(10),
                },
                repository,
                DbAuditLog {
                    repository: NoopAuditLogRepository,
                    time_generator: TimeGenerator,
                    uuid_generator: UuidV7Generator,
                },
            )
            .unwrap(),
        )
    }

    fn setup_state_with_valid_user() -> (
        Arc<AdminPortalState<InMemoryRepository, NoopAuditLogRepository>>,
        CookieJar,
    ) {
        let session_id = random_string(12);
        let repository = InMemoryRepository::default();
        repository.sessions.lock().unwrap().insert(
            session_id.clone(),
            AdminPortalUserSession {
                user_id: "user_id".to_string(),
                display_name: "Jane Doe".to_string(),
                roles: vec!["privilege_admin".to_string(), "offline_access".to_string()],
                id_token: "the-id-token".to_string(),
                expires_at: Utc::now() + TimeDelta::minutes(5),
            },
        );
        let state = setup_state_with_repository(repository);
        let jar = CookieJar::new().add(Cookie::new(SESSION_ID_COOKIE, session_id));

        (state, jar)
    }

    #[tokio::test]
    async fn login_reports_server_error_when_oidc_provider_is_unreachable() {
        let state = setup_state();

        let response = login(State(state), Query(LoginQuery { login_hint: None })).await;

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn callback_without_state_redirects_to_frontend_error_page() {
        let state = setup_state();

        let response = callback(
            State(state),
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

    #[tokio::test]
    async fn callback_without_state_but_with_session_redirects_to_frontend_entry() {
        let (state, jar) = setup_state_with_valid_user();

        let response = callback(
            State(state),
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

    #[tokio::test]
    async fn logout_without_session_cookie_redirects_to_frontend_entry() {
        let state = setup_state();

        let response = logout(State(state), CookieJar::new()).await;

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            "https://portal.example.org/"
        );
    }

    #[tokio::test]
    async fn logout_redirects_to_error_page_for_database_errors() {
        let mut session_repository = InMemoryRepository::default();
        session_repository.set_errors(true);

        let state = setup_state_with_repository(session_repository);
        let cookie_jar = CookieJar::new().add(Cookie::new(SESSION_ID_COOKIE, "my_session"));

        let response = logout(State(state), cookie_jar).await;

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            "https://portal.example.org/error?reason=no+rows+were+updated"
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

    #[tokio::test]
    async fn authenticated_admin_user_without_session_cookie_is_rejected_with_401() {
        let state = setup_state();
        let mut parts = request_parts(&CookieJar::new());

        let response = AuthenticatedAdminUser::from_request_parts(&mut parts, &state)
            .await
            .expect_err("missing session cookie should be rejected");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn authenticated_admin_user_with_unknown_session_cookie_is_rejected_with_401() {
        let state = setup_state();
        let jar = CookieJar::new().add(Cookie::new(SESSION_ID_COOKIE, "unknown-session"));
        let mut parts = request_parts(&jar);

        let response = AuthenticatedAdminUser::from_request_parts(&mut parts, &state)
            .await
            .expect_err("unknown session should be rejected");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn authenticated_admin_user_reports_server_error_for_database_errors() {
        let mut repository = InMemoryRepository::default();
        repository.set_errors(true);
        let state = setup_state_with_repository(repository);
        let jar = CookieJar::new().add(Cookie::new(SESSION_ID_COOKIE, "my-session"));
        let mut parts = request_parts(&jar);

        let response = AuthenticatedAdminUser::from_request_parts(&mut parts, &state)
            .await
            .expect_err("database errors should reject extraction");

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn authenticated_admin_user_with_valid_session_cookie_is_extracted() {
        let (state, jar) = setup_state_with_valid_user();
        let mut parts = request_parts(&jar);

        let AuthenticatedAdminUser(user) = AuthenticatedAdminUser::from_request_parts(&mut parts, &state)
            .await
            .expect("valid session should be extracted");

        assert_eq!(user.user_id, "user_id");
        assert_eq!(user.display_name, "Jane Doe");
        assert_eq!(user.privileges, vec!["admin"]);
    }

    #[tokio::test]
    async fn me_with_valid_session_reports_logged_in_user() {
        let (state, jar) = setup_state_with_valid_user();
        let mut parts = request_parts(&jar);
        let user = AuthenticatedAdminUser::from_request_parts(&mut parts, &state)
            .await
            .expect("valid session should be extracted");

        let response = me(user).await;

        assert_eq!(response.status(), StatusCode::OK);
        assert!(!response.headers().contains_key(header::SET_COOKIE));
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            json,
            serde_json::json!({"displayName": "Jane Doe", "privileges": ["admin"]})
        );
    }

    #[test]
    fn task_response_maps_display_names_not_user_ids() {
        let task = AdminTask {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            created_by_user_id: "user-1".to_string(),
            created_by_display_name: "User One".to_string(),
            task_type: TaskType::RevokeWallet,
            task_parameters: serde_json::json!({}),
            task_reason: "reason".to_string(),
            status: TaskStatus::Created,
            reviewed_at: None,
            reviewed_by_user_id: None,
            reviewed_by_display_name: None,
            review_comment: None,
            executed_at: None,
            execution_result: None,
            execution_result_details: None,
        };

        let response = TaskResponse::from(task);

        assert_eq!(response.created_by, "User One");
    }

    #[test]
    fn withdraw_task_error_status_codes() {
        for (error, status) in [
            (WithdrawTaskError::NotFound(Uuid::new_v4()), StatusCode::NOT_FOUND),
            (WithdrawTaskError::InvalidState, StatusCode::CONFLICT),
            (
                WithdrawTaskError::Storage(PersistenceError::NoRowsUpdated),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            (
                WithdrawTaskError::AuditLog(Box::new(PersistenceError::NoRowsUpdated)),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        ] {
            assert_eq!(WithdrawTaskApiError(error).into_response().status(), status);
        }
    }

    #[rstest::rstest]
    #[case::missing_session(None)]
    #[case::unknown_session(Some("unknown-session"))]
    #[tokio::test]
    async fn withdrawal_requires_authentication(#[case] session: Option<&str>) {
        let mut request = Request::builder()
            .method("POST")
            .uri(format!("/tasks/{}/withdraw", Uuid::new_v4()));
        if let Some(session) = session {
            request = request.header(header::COOKIE, format!("sid={session}"));
        }
        let response = router(setup_state())
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[rstest::rstest]
    #[case::not_found(false, "user_id", TaskStatus::Created, StatusCode::NOT_FOUND)]
    #[case::wrong_owner(true, "other-user", TaskStatus::Created, StatusCode::CONFLICT)]
    #[case::already_approved(true, "user_id", TaskStatus::Approved, StatusCode::CONFLICT)]
    #[case::success(true, "user_id", TaskStatus::Created, StatusCode::OK)]
    #[tokio::test]
    async fn withdrawal_endpoint(
        #[case] exists: bool,
        #[case] owner: &str,
        #[case] status: TaskStatus,
        #[case] expected_status: StatusCode,
    ) {
        let (state, jar) = setup_state_with_valid_user();
        let id = Uuid::new_v4();
        if exists {
            let repository = state.service.repository();
            repository
                .insert_task(AdminTaskCreate {
                    id,
                    created_at: Utc::now(),
                    created_by_user_id: owner.to_string(),
                    created_by_display_name: "Jane Doe".to_string(),
                    task_type: TaskType::RevokeWallet,
                    task_parameters: serde_json::json!({}),
                    task_reason: "reason".to_string(),
                })
                .await
                .unwrap();
            if status != TaskStatus::Created {
                // Seed the existing status for cases where the task has already been reviewed.
                repository.tasks.lock().unwrap().get_mut(&id).unwrap().status = status;
            }
        }
        let request = Request::builder()
            .method("POST")
            .uri(format!("/tasks/{id}/withdraw"))
            .header(
                header::COOKIE,
                format!("sid={}", jar.get(SESSION_ID_COOKIE).unwrap().value()),
            )
            .body(Body::empty())
            .unwrap();
        let app = router(state);
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), expected_status);
        if expected_status == StatusCode::OK {
            let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let task: Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(task["taskId"], id.to_string());
            assert_eq!(task["status"], "withdrawn");
            assert_eq!(task["createdBy"], "Jane Doe");
            for field in ["executedAt", "executionResult", "executionResultDetails"] {
                assert!(task[field].is_null());
            }
            let request = Request::builder()
                .method("POST")
                .uri(format!("/tasks/{id}/withdraw"))
                .header(
                    header::COOKIE,
                    format!("sid={}", jar.get(SESSION_ID_COOKIE).unwrap().value()),
                )
                .body(Body::empty())
                .unwrap();
            assert_eq!(app.oneshot(request).await.unwrap().status(), StatusCode::CONFLICT);
        }
    }
}
