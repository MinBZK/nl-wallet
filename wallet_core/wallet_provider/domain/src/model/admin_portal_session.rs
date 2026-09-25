use chrono::DateTime;
use chrono::Utc;

/// State kept between issuing a Keycloak authorization redirect and handling its callback.
#[derive(Debug, Clone)]
pub struct AdminPortalLoginAttempt {
    pub nonce: String,
    pub code_verifier: String,
    pub created_at: DateTime<Utc>,
}

/// A logged-in Admin Portal user's server-side session.
#[derive(Debug, Clone)]
pub struct AdminPortalUserSession {
    pub display_name: String,
    pub roles: Vec<String>,
    pub id_token: String,
    pub expires_at: DateTime<Utc>,
}
