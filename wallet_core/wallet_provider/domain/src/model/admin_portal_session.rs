use chrono::DateTime;
use chrono::Utc;
use crypto::utils::random_string;
use subtle::ConstantTimeEq;

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
    pub user_id: String,
    pub display_name: String,
    pub roles: Vec<String>,
    pub id_token: String,
    pub expires_at: DateTime<Utc>,
}

/// Length of a CSRF token; 43 alphanumeric characters carry more than 256 bits of entropy.
const CSRF_TOKEN_LENGTH: usize = 43;

/// An independent random secret for one login session, submitted explicitly on task mutations.
/// Keep it out of debug logs; only the authenticated token endpoint exposes its value.
#[derive(Clone, derive_more::Debug, derive_more::Into)]
#[debug("<redacted>")]
pub struct CsrfToken(String);

impl CsrfToken {
    pub fn generate() -> Self {
        Self(random_string(CSRF_TOKEN_LENGTH))
    }

    pub fn verify(&self, supplied: &str) -> bool {
        self.0.as_bytes().ct_eq(supplied.as_bytes()).into()
    }
}

impl AsRef<str> for CsrfToken {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, thiserror::Error)]
#[error("invalid CSRF token")]
pub struct InvalidCsrfToken;

impl TryFrom<String> for CsrfToken {
    type Error = InvalidCsrfToken;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() != CSRF_TOKEN_LENGTH || !value.bytes().all(|byte| byte.is_ascii_alphanumeric()) {
            return Err(InvalidCsrfToken);
        }
        Ok(Self(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csrf_token_only_accepts_its_own_value() {
        let token = CsrfToken::generate();
        assert!(token.verify(token.as_ref()));
        assert!(!token.verify(CsrfToken::generate().as_ref()));
        assert!(!token.verify(""));
        assert!(!token.verify(&format!("{}extra", token.as_ref())));
        assert!(!format!("{token:?}").contains(token.as_ref()));
    }

    #[test]
    fn csrf_token_parses_only_valid_stored_tokens() {
        let valid = "a".repeat(CSRF_TOKEN_LENGTH);
        assert!(CsrfToken::try_from(valid.clone()).unwrap().verify(&valid));
        for invalid in [
            String::new(),
            "a".repeat(CSRF_TOKEN_LENGTH - 1),
            "a".repeat(CSRF_TOKEN_LENGTH + 1),
            "!".repeat(CSRF_TOKEN_LENGTH),
            "é".repeat(CSRF_TOKEN_LENGTH),
        ] {
            assert!(CsrfToken::try_from(invalid).is_err());
        }
    }
}
