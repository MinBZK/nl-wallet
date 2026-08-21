use std::collections::HashSet;
use std::time::Duration;

use crypto::utils::random_string;
use crypto::utils::sha256;
use derive_more::From;
use serde::Deserialize;
use serde::Serialize;
use serde_with::DurationSeconds;
use serde_with::StringWithSeparator;
use serde_with::formats::SpaceSeparator;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use url::Url;

use crate::scope::Scope;

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

/// An OAuth 2.0 Token Request as defined by [RFC 6749 §4.1.3](https://www.rfc-editor.org/rfc/rfc6749.html#section-4.1.3),
/// generic over the `grant_type` value, since the set of supported grant types (and their accompanying fields) vary
/// per profile.
///
/// Sent URL-encoded in the request body to `POST /token`.
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRequest<G> {
    #[serde(flatten)]
    pub grant_type: G,

    pub client_id: Option<String>,

    /// MUST be the redirect URI value as passed in the Authorization Request.
    pub redirect_uri: Option<Url>,

    /// Section 3.3 of RFC 6749 states that the client may include a scope value when sending the Token Request to the
    /// token endpoint. Note that, unlike the Authorization Request, we make a semantic distinction between this value
    /// not being included and the scope value set being empty.
    #[serde_as(as = "Option<StringWithSeparator::<SpaceSeparator, Scope>>")]
    pub scope: Option<HashSet<Scope>>,

    /// The PKCE code verifier as defined in RFC 7636.
    pub code_verifier: Option<String>,
}

/// An OAuth 2.0 Token Response as defined by
/// [RFC 6749 §5.1](https://www.rfc-editor.org/rfc/rfc6749.html#section-5.1).
#[serde_as]
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: AccessToken,
    pub token_type: TokenType,
    pub refresh_token: Option<String>,

    /// Section 3.3 of RFC 6749 states that if the issued access token scope is different than the one requested by the
    /// client, the server MUST include a scope response parameter to inform the client of the actual scope granted.
    /// Conversely, this means that if the scope is absent, access token scope is what the client requested.
    #[serde_as(as = "Option<StringWithSeparator::<SpaceSeparator, Scope>>")]
    pub scope: Option<HashSet<Scope>>,

    #[serde_as(as = "Option<DurationSeconds<u64>>")]
    pub expires_in: Option<Duration>,
}

impl TokenResponse {
    pub fn new(access_token: AccessToken, token_type: TokenType) -> Self {
        Self {
            access_token,
            token_type,
            expires_in: None,
            refresh_token: None,
            scope: None,
        }
    }
}
