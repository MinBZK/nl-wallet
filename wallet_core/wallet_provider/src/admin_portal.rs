//! HTTP layer for the Admin Portal's OIDC login flow. The actual OIDC/session logic lives in
//! [`wallet_provider_service::admin_portal::AdminPortalService`]; this module only wires it up to
//! axum: routing, query/cookie extraction and turning results into redirects or JSON responses.
//!
//! `/auth/login` redirects to Keycloak (with PKCE), `/auth/callback` exchanges the resulting
//! authorization code for tokens and starts a server-side session, `/auth/logout` ends both the
//! local session and the Keycloak SSO session, and `/api/me` reports the logged-in user to the
//! SPA based on the session cookie.

use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Redirect;
use axum::response::Response;
use axum::routing::get;
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::cookie::SameSite;
use http_utils::reqwest::ReqwestTrustAnchor;
use http_utils::reqwest::tls_pinned_client_builder;
use http_utils::urls::BaseUrl;
use serde::Deserialize;
use serde::Serialize;
use tracing::warn;
use url::Url;
use wallet_provider_domain::repository::AdminPortalSessionRepository;
use wallet_provider_service::admin_portal::AdminPortalConfig;
use wallet_provider_service::admin_portal::AdminPortalService;
use wallet_provider_service::admin_portal::LoggedInUser;

use crate::settings::AdminPortalSettings;

const SESSION_ID_COOKIE: &str = "sid";

pub struct AdminPortalState<R> {
    service: AdminPortalService<R>,
    public_url: BaseUrl,
}

impl<R> AdminPortalState<R> {
    pub fn try_new(settings: AdminPortalSettings, repository: R) -> Result<Self, reqwest::Error> {
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
        })
    }

    fn frontend_url(&self, path: &str) -> Url {
        self.public_url.join(path)
    }
}

pub fn router<R>(state: Arc<AdminPortalState<R>>) -> Router
where
    R: AdminPortalSessionRepository + Send + Sync + 'static,
{
    Router::new()
        .route("/auth/login", get(login))
        .route("/auth/callback", get(callback))
        .route("/auth/logout", get(logout))
        .route("/api/me", get(me))
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
fn error_redirect<R>(state: &AdminPortalState<R>, reason: &str) -> Response {
    let mut url = state.frontend_url("/error");
    url.query_pairs_mut().append_pair("reason", reason);

    Redirect::to(url.as_str()).into_response()
}

#[derive(Deserialize)]
struct LoginQuery {
    login_hint: Option<String>,
}

async fn login<R>(State(state): State<Arc<AdminPortalState<R>>>, Query(query): Query<LoginQuery>) -> Response
where
    R: AdminPortalSessionRepository + Send + Sync + 'static,
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

async fn callback<R>(State(state): State<Arc<AdminPortalState<R>>>, Query(query): Query<CallbackQuery>) -> Response
where
    R: AdminPortalSessionRepository + Send + Sync + 'static,
{
    let session_id = state
        .service
        .handle_callback(query.state.as_deref(), query.code.as_deref(), query.error.as_deref())
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

async fn logout<R>(State(state): State<Arc<AdminPortalState<R>>>, cookie_jar: CookieJar) -> Response
where
    R: AdminPortalSessionRepository + Send + Sync + 'static,
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
        }: LoggedInUser,
    ) -> Self {
        Self {
            display_name,
            privileges,
        }
    }
}

async fn me<R>(State(state): State<Arc<AdminPortalState<R>>>, cookie_jar: CookieJar) -> Response
where
    R: AdminPortalSessionRepository + Send + Sync + 'static,
{
    // Retrieve the session_id from the cookie, note that we don't need to touch the cookie's `max_age`, as it is a
    // session cookie, see the documentation at `fn session_cookie`.
    let Some(session_id) = cookie_jar.get(SESSION_ID_COOKIE).map(|cookie| cookie.value()) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    match state.service.authenticated_user(session_id).await {
        Ok(Some(user)) => Json(MeResponse::from(user)).into_response(),
        Ok(None) => StatusCode::UNAUTHORIZED.into_response(),
        Err(error) => {
            warn!("/api/me failed: {error}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::net::TcpListener;
    use std::sync::Mutex;
    use std::time::Duration;

    use axum::body::to_bytes;
    use axum::http::StatusCode;
    use axum::http::header;
    use chrono::DateTime;
    use chrono::TimeDelta;
    use chrono::Utc;
    use http_utils::reqwest::test::get_test_trust_anchor;
    use http_utils::urls::BaseUrl;
    use serde_json::Value;
    use utils::vec_nonempty;
    use wallet_provider_domain::model::admin_portal_session::AdminPortalLoginAttempt;
    use wallet_provider_domain::model::admin_portal_session::AdminPortalUserSession;
    use wallet_provider_domain::repository::AdminPortalSessionRepository;
    use wallet_provider_domain::repository::PersistenceError;

    use super::*;

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

        async fn touch_user_session(
            &self,
            session_id: &str,
            now: DateTime<Utc>,
            new_expires_at: DateTime<Utc>,
        ) -> Result<Option<AdminPortalUserSession>, PersistenceError> {
            self.handle_error()?;
            let mut sessions = self.sessions.lock().unwrap();
            let Some(session) = sessions.get_mut(session_id) else {
                return Ok(None);
            };
            if session.expires_at <= now {
                return Ok(None);
            }
            session.expires_at = new_expires_at;
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

    /// Binds an ephemeral local port and immediately releases it, so that connecting to it is reliably refused.
    fn unreachable_keycloak_url() -> BaseUrl {
        let port = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        format!("https://127.0.0.1:{port}").parse().unwrap()
    }

    fn state(repository: InMemoryRepository, keycloak_url: BaseUrl) -> Arc<AdminPortalState<InMemoryRepository>> {
        Arc::new(
            AdminPortalState::try_new(
                AdminPortalSettings {
                    keycloak_url,
                    keycloak_realm: "test-realm".to_string(),
                    keycloak_client_id: "test-client".to_string(),
                    public_url: "https://portal.example.org/".parse().unwrap(),
                    keycloak_trust_anchors: vec_nonempty![get_test_trust_anchor()],
                    session_ttl: Duration::from_mins(15),
                    login_attempt_ttl: Duration::from_mins(10),
                },
                repository,
            )
            .unwrap(),
        )
    }

    #[tokio::test]
    async fn login_reports_server_error_when_oidc_provider_is_unreachable() {
        let state = state(InMemoryRepository::default(), unreachable_keycloak_url());

        let response = login(State(state), Query(LoginQuery { login_hint: None })).await;

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn callback_without_state_redirects_to_frontend_error_page() {
        let state = state(InMemoryRepository::default(), unreachable_keycloak_url());

        let response = callback(
            State(state),
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
    async fn logout_without_session_cookie_redirects_to_frontend_entry() {
        let state = state(InMemoryRepository::default(), unreachable_keycloak_url());

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

        let state = state(session_repository, unreachable_keycloak_url());
        let cookie_jar = CookieJar::new().add(Cookie::new(SESSION_ID_COOKIE, "my_session"));

        let response = logout(State(state), cookie_jar).await;

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            "https://portal.example.org/error?reason=no+rows+were+updated"
        );
    }

    #[tokio::test]
    async fn me_without_session_cookie_is_unauthorized() {
        let state = state(InMemoryRepository::default(), unreachable_keycloak_url());

        let response = me(State(state), CookieJar::new()).await;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn me_with_unknown_session_cookie_is_unauthorized() {
        let state = state(InMemoryRepository::default(), unreachable_keycloak_url());
        let jar = CookieJar::new().add(Cookie::new(SESSION_ID_COOKIE, "unknown-session"));

        let response = me(State(state), jar).await;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn me_with_valid_session_reports_logged_in_user() {
        let repository = InMemoryRepository::default();
        repository.sessions.lock().unwrap().insert(
            "the-session-id".to_string(),
            AdminPortalUserSession {
                display_name: "Jane Doe".to_string(),
                roles: vec!["privilege_admin".to_string(), "offline_access".to_string()],
                id_token: "the-id-token".to_string(),
                expires_at: Utc::now() + TimeDelta::minutes(5),
            },
        );
        let state = state(repository, unreachable_keycloak_url());
        let jar = CookieJar::new().add(Cookie::new(SESSION_ID_COOKIE, "the-session-id"));

        let response = me(State(state), jar).await;

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["displayName"], "Jane Doe");
        assert_eq!(json["privileges"], serde_json::json!(["admin"]));
    }
}
