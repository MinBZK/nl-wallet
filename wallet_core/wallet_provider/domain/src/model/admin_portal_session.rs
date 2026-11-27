use std::fmt;
use std::fmt::Debug;
use std::fmt::Formatter;

use chrono::DateTime;
use chrono::Utc;
use crypto::utils::random_string;
use nutype::nutype;
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
    pub csrf_token: CsrfToken,
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
#[nutype(
    derive(Clone, AsRef, TryFrom, Into),
    validate(predicate = |token| token.len() == CSRF_TOKEN_LENGTH && token.bytes().all(|byte| byte.is_ascii_alphanumeric()))
)]
pub struct CsrfToken(String);

impl CsrfToken {
    pub fn generate() -> Self {
        Self::try_new(random_string(CSRF_TOKEN_LENGTH))
            .expect("random_string only produces alphanumeric characters of the requested length")
    }

    pub fn verify(&self, supplied: &str) -> bool {
        self.as_ref().as_bytes().ct_eq(supplied.as_bytes()).into()
    }
}

impl Debug for CsrfToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
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
    }

    #[test]
    fn csrf_token_debug_output_is_redacted() {
        let token = CsrfToken::generate();
        assert_eq!(format!("{token:?}"), "<redacted>");
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
