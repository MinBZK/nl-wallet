use chrono::DateTime;
use chrono::Utc;

use super::errors::PersistenceError;
use crate::model::admin_portal_session::AdminPortalLoginAttempt;
use crate::model::admin_portal_session::AdminPortalUserSession;

type Result<T> = std::result::Result<T, PersistenceError>;

/// Persists the Admin Portal's OIDC login state (pending logins and server-side sessions), so that
/// they are shared between replicas instead of being kept in memory.
#[trait_variant::make(Send)]
pub trait AdminPortalSessionRepository {
    async fn insert_login_attempt(&self, state: String, login_attempt: AdminPortalLoginAttempt) -> Result<()>;

    /// Removes and returns the login attempt identified by `state`, if present.
    async fn take_login_attempt(&self, state: &str) -> Result<Option<AdminPortalLoginAttempt>>;

    /// Deletes login attempts created before `created_before`.
    async fn cleanup_expired_login_attempts(&self, created_before: DateTime<Utc>) -> Result<()>;

    async fn insert_user_session(&self, session_id: String, session: AdminPortalUserSession) -> Result<()>;

    /// Extends the session's expiry to `new_expires_at`, provided it has not already expired as of
    /// `now`, and returns the updated session. Returns `None` if no valid session was found.
    async fn touch_user_session(
        &self,
        session_id: &str,
        now: DateTime<Utc>,
        new_expires_at: DateTime<Utc>,
    ) -> Result<Option<AdminPortalUserSession>>;

    /// Removes and returns the session identified by `session_id`, if present.
    async fn take_user_session(&self, session_id: &str) -> Result<Option<AdminPortalUserSession>>;

    /// Deletes sessions that expired at or before `now`.
    async fn cleanup_expired_user_sessions(&self, now: DateTime<Utc>) -> Result<()>;
}
