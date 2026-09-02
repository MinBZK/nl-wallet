use std::collections::HashSet;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::str::FromStr;

use chrono::Duration;
use jwt::nonce::Nonce;
use serde::Deserialize;
use serde::Serialize;
use serde_with::DeserializeFromStr;
use serde_with::DurationSeconds;
use serde_with::SerializeDisplay;
use serde_with::StringWithSeparator;
use serde_with::formats::SpaceSeparator;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use url::Url;
use utils::spec::SpecForbidden;
use utils::spec::SpecOptional;

use crate::pkce::PkceCodeChallenge;
use crate::pkce::PkcePair;
use crate::scope::Scope;

/// Media type for a JWT-Secured Authorization Request (JAR), as defined by
/// [RFC 9101](https://www.rfc-editor.org/rfc/rfc9101.html#section-4).
pub const APPLICATION_OAUTH_AUTHZ_REQ_JWT: &str = "application/oauth-authz-req+jwt";

/// The shared [OAuth 2.0 RFC 6749](https://www.rfc-editor.org/rfc/rfc6749.html#section-4.1.1) fields that any
/// authorization request must carry. Generic over the `response_type` value, since supported response types vary
/// per profile (e.g. plain OAuth `code`, OpenID4VP's `vp_token`, SIOPv2's `id_token`).
///
/// Flow-specific request types embed this with `#[serde(flatten)]` and add their own fields.
#[serde_as]
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuthorizationRequestBase<T>
where
    T: Display + FromStr + Eq + Hash + Clone + Debug,
    T::Err: Display,
{
    #[serde_as(as = "StringWithSeparator::<SpaceSeparator, T>")]
    pub response_type: HashSet<T>,

    pub client_id: String,
    pub state: Option<String>,

    // Should not be present for PAR and openid4vp.
    #[serde(default, skip_serializing, rename = "request_uri")]
    _request_uri: SpecForbidden,
}

impl<T> AuthorizationRequestBase<T>
where
    T: Display + FromStr + Eq + Hash + Clone + Debug,
    T::Err: Display,
{
    pub fn new(response_type: HashSet<T>, client_id: String, state: Option<String>) -> Self {
        Self {
            response_type,
            client_id,
            state,
            _request_uri: SpecForbidden,
        }
    }
}

/// The values the `response_type` authorization request parameter can take. `code` is defined by
/// [OAuth 2.0](https://www.rfc-editor.org/rfc/rfc6749.html#section-3.1.1), the other two by profiles built on top of
/// it; all three are registered in the IANA OAuth Authorization Endpoint Response Types registry.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Hash,
    SerializeDisplay,
    DeserializeFromStr,
    strum::EnumString,
    strum::Display,
)]
#[strum(serialize_all = "snake_case")]
pub enum ResponseType {
    /// OAuth
    #[default]
    Code,

    /// OpenID4VP
    VpToken,

    /// SIOPv2 (not supported (yet))
    IdToken,
}

/// An authorization request for the OAuth 2.0 authorization code grant
/// ([RFC 6749](https://www.rfc-editor.org/rfc/rfc6749.html#section-4.1.1)) protected with PKCE
/// ([RFC 7636](https://www.rfc-editor.org/rfc/rfc7636.html#section-4.3)).
///
/// This carries no parameters from any profile layered on top of OAuth: OpenID Connect adds `nonce` in
/// [`OidcAuthorizationRequest`], OpenID4VCI adds `issuer_state` and `authorization_details` in its own
/// `VciAuthorizationRequest`. Both embed this type with `#[serde(flatten)]` rather than one another, since neither
/// profile's parameters are valid in the other's requests.
///
/// Note that `redirect_uri` and `code_challenge` are stricter here than plain OAuth 2.0, which allows both to be
/// omitted: this type follows OAuth 2.1 in requiring PKCE, and both profiles using it require a `redirect_uri` (see
/// the field documentation).
#[serde_as]
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuthorizationCodeRequest {
    #[serde(flatten)]
    pub oauth_request: AuthorizationRequestBase<ResponseType>,

    /// Required in this setting: OAuth 2.0 only permits omitting `redirect_uri` when the client has a single
    /// pre-registered redirect URI with the Authorization Server (RFC 6749 §3.1.2.3). OpenID4VCI wallets aren't
    /// registered, and OpenID Connect requires the parameter unconditionally
    /// (<https://openid.net/specs/openid-connect-core-1_0.html#AuthRequest>).
    pub redirect_uri: SpecOptional<Url>,

    #[serde(flatten)]
    pub code_challenge: PkceCodeChallenge,

    #[serde_as(as = "StringWithSeparator::<SpaceSeparator, Scope>")]
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub scope: HashSet<Scope>,
}

impl AuthorizationCodeRequest {
    pub fn new<P: PkcePair>(
        client_id: String,
        redirect_uri: Url,
        state: String,
        scope: HashSet<Scope>,
        pkce_pair: &P,
    ) -> Self {
        Self {
            oauth_request: AuthorizationRequestBase::new(HashSet::from([ResponseType::Code]), client_id, Some(state)),
            redirect_uri: redirect_uri.into(),
            code_challenge: PkceCodeChallenge::S256 {
                code_challenge: String::from(pkce_pair.code_challenge()),
            },
            scope,
        }
    }
}

/// An [OIDC](https://openid.net/specs/openid-connect-core-1_0.html#AuthRequest) authorization request: the OAuth 2.0
/// authorization code request plus the OpenID Connect `nonce` parameter.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OidcAuthorizationRequest {
    #[serde(flatten)]
    pub auth_request: AuthorizationCodeRequest,

    /// REQUIRED for the implicit and hybrid flows, OPTIONAL for the authorization code flow.
    pub nonce: Option<Nonce>,
}

/// The OAuth 2.0 Authorization Response, which is URL-encoded and provided as query parameters added to the
/// `redirect_uri` that was passed in the Authorization Request.
///
/// See: <https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.2>.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationResponse {
    pub code: String,
    pub state: Option<String>,
}

/// Represents the response from the `/par` endpoint containing a `request_uri` that can be used to retrieve the
/// pushed authorization request later at the `/authorize` endpoint, as defined by
/// [RFC 9126](https://www.rfc-editor.org/rfc/rfc9126.html#section-2.2).
#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct PushedAuthorizationResponse {
    pub request_uri: String,

    #[serde_as(as = "DurationSeconds<i64>")]
    pub expires_in: Duration,
}

/// Represents the parameters that are passed in the query string of the `/authorize` endpoint where the
/// `request_uri` refers to a pushed authorization request sent earlier, as defined by
/// [RFC 9126](https://www.rfc-editor.org/rfc/rfc9126.html#section-4).
#[derive(Serialize, Deserialize, Debug)]
pub struct PushedAuthorizationRequest {
    pub client_id: String,
    pub request_uri: String,
}

impl PushedAuthorizationRequest {
    pub fn from_par_response(client_id: String, response: &PushedAuthorizationResponse) -> Self {
        Self {
            client_id,
            request_uri: response.request_uri.clone(),
        }
    }

    pub fn into_authorization_url(self, mut endpoint: url::Url) -> url::Url {
        endpoint
            .query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("request_uri", &self.request_uri);
        endpoint
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use jwt::nonce::Nonce;
    use url::Url;

    use super::AuthorizationCodeRequest;
    use super::AuthorizationRequestBase;
    use super::OidcAuthorizationRequest;
    use super::ResponseType;
    use crate::pkce::PkceCodeChallenge;

    fn example_request() -> AuthorizationCodeRequest {
        AuthorizationCodeRequest {
            oauth_request: AuthorizationRequestBase::new(
                HashSet::from([ResponseType::Code]),
                "client-123".to_string(),
                Some("state-abc".to_string()),
            ),
            redirect_uri: Url::parse("https://example.com/callback").unwrap().into(),
            code_challenge: PkceCodeChallenge::S256 {
                code_challenge: "challenge-xyz".to_string(),
            },
            scope: HashSet::from(["openid".parse().unwrap(), "profile".parse().unwrap()]),
        }
    }

    #[test]
    fn authorization_code_request_urlencoded_roundtrip() {
        let request = example_request();

        let encoded = serde_qs::to_string(&request).unwrap();
        let decoded: AuthorizationCodeRequest = serde_qs::from_str(&encoded).unwrap();

        assert_eq!(decoded.oauth_request.client_id, "client-123");
        assert_eq!(decoded.oauth_request.state.as_deref(), Some("state-abc"));
        assert_eq!(decoded.redirect_uri.as_ref().as_str(), "https://example.com/callback");
        assert_eq!(
            decoded.scope,
            HashSet::from(["openid".parse().unwrap(), "profile".parse().unwrap()])
        );
        assert!(matches!(
            decoded.code_challenge,
            PkceCodeChallenge::S256 { code_challenge } if code_challenge == "challenge-xyz"
        ));
    }

    #[test]
    fn authorization_code_request_rejects_request_uri() {
        let mut encoded = serde_qs::to_string(&example_request()).unwrap();
        encoded.push_str("&request_uri=should-not-be-here");

        let err = serde_qs::from_str::<AuthorizationCodeRequest>(&encoded).unwrap_err();
        assert!(
            err.to_string().contains("MUST NOT be present"),
            "expected SpecForbidden rejection, got: {err}"
        );
    }

    #[test]
    fn oidc_authorization_request_urlencoded_roundtrip() {
        let nonce = Nonce::new_random();
        let request = OidcAuthorizationRequest {
            auth_request: example_request(),
            nonce: Some(nonce.clone()),
        };

        let encoded = serde_qs::to_string(&request).unwrap();
        let decoded: OidcAuthorizationRequest = serde_qs::from_str(&encoded).unwrap();

        assert_eq!(decoded.nonce, Some(nonce));
        assert_eq!(decoded.auth_request.oauth_request.client_id, "client-123");
    }

    #[test]
    fn oidc_authorization_request_has_no_openid4vci_parameters() {
        let request = OidcAuthorizationRequest {
            auth_request: example_request(),
            nonce: Some(Nonce::new_random()),
        };

        let encoded = serde_qs::to_string(&request).unwrap();

        assert!(!encoded.contains("issuer_state"));
        assert!(!encoded.contains("authorization_details"));
    }
}
