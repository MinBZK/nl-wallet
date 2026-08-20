use crypto::utils::random_string;
use crypto::utils::sha256;
use derive_more::From;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug, Clone, From)]
pub struct AuthorizationCode(String);

impl AsRef<str> for AuthorizationCode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, From)]
pub struct AccessToken(String);

impl AsRef<str> for AccessToken {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AccessToken {
    /// Construct a new random access token, with the specified authorization code appended to it.
    pub fn new(code: &AuthorizationCode) -> Self {
        Self(random_string(32) + code.as_ref())
    }

    /// Returns the authorization code appended to this access token.
    pub fn code(&self) -> Option<AuthorizationCode> {
        self.as_ref().get(32..).map(|code| AuthorizationCode(code.to_string()))
    }

    pub fn sha256(&self) -> Vec<u8> {
        sha256(self.as_ref().as_bytes())
    }
}

/// The token type of an access token, as defined by
/// [RFC 6749 §7.1](https://www.rfc-editor.org/rfc/rfc6749.html#section-7.1) (`Bearer`) and
/// [RFC 9449](https://www.rfc-editor.org/rfc/rfc9449.html) (`DPoP`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenType {
    #[default]
    Bearer,
    DPoP,
}
