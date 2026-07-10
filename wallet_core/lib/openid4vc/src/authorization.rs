use std::collections::HashSet;

use chrono::Duration;
use jwt::nonce::Nonce;
use serde::Deserialize;
use serde::Serialize;
use serde_with::DeserializeFromStr;
use serde_with::DurationSeconds;
use serde_with::SerializeDisplay;
use serde_with::StringWithSeparator;
use serde_with::TryFromInto;
use serde_with::formats::SpaceSeparator;
use serde_with::json::JsonString;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use url::Url;
use utils::spec::SpecForbidden;
use utils::spec::SpecOptional;

use crate::authorization_details::WalletAuthorizationDetails;
use crate::authorization_details::WalletAuthorizationDetailsEntries;
use crate::pkce::PkcePair;
use crate::scope::Scope;

/// The shared [OAuth2 RFC 6749](https://www.rfc-editor.org/rfc/rfc6749.html#section-4.1.1) fields that any
/// authorization request — whether for OpenID4VCI issuance or OpenID4VP presentation — must carry.
///
/// Flow-specific variants embed this with `#[serde(flatten)]` and add their own fields.
#[serde_as]
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuthorizationRequestBase {
    #[serde_as(as = "StringWithSeparator::<SpaceSeparator, ResponseType>")]
    pub response_type: HashSet<ResponseType>,

    pub client_id: String,
    pub state: Option<String>,

    // Should not be present for PAR and openid4vp.
    #[serde(default, skip_serializing, rename = "request_uri")]
    _request_uri: SpecForbidden,
}

impl AuthorizationRequestBase {
    pub fn for_vp(client_id: String, state: Option<String>) -> Self {
        Self {
            response_type: HashSet::from([ResponseType::VpToken]),
            client_id,
            state,
            _request_uri: SpecForbidden,
        }
    }
}

/// An OpenID4VCI authorization request, posted in URL-encoded form to the `/par` endpoint
/// (RFC 9126) and later referenced from `/authorize` via [`PushedAuthorizationRequest`].
#[serde_as]
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VciAuthorizationRequest {
    #[serde(flatten)]
    pub oauth_request: AuthorizationRequestBase,

    /// Required in this setting: OAuth 2.0 only permits omitting `redirect_uri` when the client has a single
    /// pre-registered redirect URI with the Authorization Server (RFC 6749 §3.1.2.3), and OpenID4VCI wallets
    /// aren't registered.
    pub redirect_uri: SpecOptional<Url>,

    #[serde(flatten)]
    pub code_challenge: PkceCodeChallenge,

    #[serde_as(as = "StringWithSeparator::<SpaceSeparator, Scope>")]
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub scope: HashSet<Scope>,

    /// String value identifying a certain processing context at the Credential Issuer. A value for this parameter is
    /// typically passed in a Credential Offer from the Credential Issuer to the Wallet. This request parameter is used
    /// to pass the `issuer_state` value back to the Credential Issuer.
    ///
    /// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-5.1.3-2.1>
    pub issuer_state: Option<String>,

    #[serde_as(as = "Option<JsonString<TryFromInto<WalletAuthorizationDetailsEntries>>>")]
    pub authorization_details: Option<WalletAuthorizationDetails>,
}

impl VciAuthorizationRequest {
    pub fn for_auth_code<P: PkcePair>(
        client_id: String,
        redirect_uri: Url,
        state: String,
        issuer_state: Option<String>,
        scope: HashSet<Scope>,
        pkce_pair: &P,
    ) -> Self {
        Self {
            oauth_request: AuthorizationRequestBase {
                response_type: HashSet::from([ResponseType::Code]),
                client_id,
                state: Some(state),
                _request_uri: SpecForbidden,
            },
            redirect_uri: redirect_uri.into(),
            code_challenge: PkceCodeChallenge::S256 {
                code_challenge: String::from(pkce_pair.code_challenge()),
            },
            scope,
            issuer_state,
            authorization_details: None,
        }
    }
}

/// An [OIDC](https://openid.net/specs/openid-connect-core-1_0.html#AuthRequest) authorization request. Adds the OIDC `nonce` parameter.
#[serde_as]
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OidcAuthorizationRequest {
    #[serde(flatten)]
    pub vci_request: VciAuthorizationRequest,

    pub nonce: Option<Nonce>,
}

/// Represents the response from the /par endpoint containing a `request_uri` that can be used to retrieve the pushed
/// [`VciAuthorizationRequest`] later at the /authorize endpoint. Note: this is not a response to the
/// [`PushedAuthorizationRequest`] defined below.
#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct PushedAuthorizationResponse {
    pub request_uri: String,

    #[serde_as(as = "DurationSeconds<i64>")]
    pub expires_in: Duration,
}

/// Represents the parameters that are passed in the query string of the /authorize endpoint where the `request_uri`
/// refers to a pushed [`VciAuthorizationRequest`] sent earlier.
#[derive(Serialize, Deserialize, Debug)]
pub struct PushedAuthorizationRequest {
    pub client_id: String,
    pub request_uri: String,
}

/// Defined in https://openid.net/specs/oauth-v2-multiple-response-types-1_0.html#ResponseModes
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseMode {
    Query,
    #[default]
    Fragment,

    // The following two are defined in OpenID4VP
    DirectPost,
    #[serde(rename = "direct_post.jwt")]
    DirectPostJwt,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "code_challenge_method")]
pub enum PkceCodeChallenge {
    S256 {
        code_challenge: String,
    },
    #[serde(rename = "plain")]
    Plain {
        code_challenge: String,
    },
}

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

/// The OAuth 2.0 Authorization Response, which is URL-encoded and provided as query parameters added to the
/// `redirect_uri` that was passed in the [`VciAuthorizationRequest`]. Contains the token that may be exchanged for an
/// access token.
///
/// See: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-5.2> and
/// <https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.2>.
/// [`VciAuthorizationRequest`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationResponse {
    pub code: String,
    pub state: Option<String>,
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use itertools::Itertools;
    use jwt::nonce::Nonce;
    use serde_qs;
    use url::Url;
    use utils::spec::SpecForbidden;

    use super::AuthorizationRequestBase;
    use super::PkceCodeChallenge;
    use super::ResponseType;
    use super::VciAuthorizationRequest;
    use crate::authorization_details::TypedAuthorizationDetailsEntry;

    fn example_vci_request() -> VciAuthorizationRequest {
        let scope = HashSet::from(["openid".parse().unwrap(), "profile".parse().unwrap()]);

        VciAuthorizationRequest {
            oauth_request: AuthorizationRequestBase {
                response_type: HashSet::from([ResponseType::Code]),
                client_id: "client-123".to_string(),
                state: Some("state-abc".to_string()),
                _request_uri: SpecForbidden,
            },
            redirect_uri: Url::parse("https://example.com/callback").unwrap().into(),
            code_challenge: PkceCodeChallenge::S256 {
                code_challenge: "challenge-xyz".to_string(),
            },
            scope,
            issuer_state: Some("state-xyz".to_string()),
            authorization_details: None,
        }
    }

    #[test]
    fn vci_authorization_request_urlencoded_roundtrip() {
        let request = example_vci_request();

        let encoded = serde_qs::to_string(&request).unwrap();
        let decoded: VciAuthorizationRequest = serde_qs::from_str(&encoded).unwrap();

        assert_eq!(decoded.oauth_request.client_id, "client-123");
        assert_eq!(decoded.oauth_request.state.as_deref(), Some("state-abc"));
        assert_eq!(
            decoded.scope,
            HashSet::from(["openid".parse().unwrap(), "profile".parse().unwrap()])
        );
        assert!(matches!(
            decoded.code_challenge,
            PkceCodeChallenge::S256 { code_challenge } if code_challenge == "challenge-xyz"
        ));
        assert_eq!(decoded.issuer_state.as_deref(), Some("state-xyz"));
        assert!(decoded.authorization_details.is_none());
    }

    #[test]
    fn oidc_authorization_request_urlencoded_roundtrip() {
        use crate::authorization::OidcAuthorizationRequest;

        let nonce = Nonce::new_random();
        let request = OidcAuthorizationRequest {
            vci_request: example_vci_request(),
            nonce: Some(nonce.clone()),
        };

        let encoded = serde_qs::to_string(&request).unwrap();
        let decoded: OidcAuthorizationRequest = serde_qs::from_str(&encoded).unwrap();

        assert_eq!(decoded.nonce, Some(nonce));
        assert_eq!(decoded.vci_request.oauth_request.client_id, "client-123");
    }

    #[test]
    fn vci_authorization_request_rejects_request_uri() {
        let request = example_vci_request();
        let mut encoded = serde_qs::to_string(&request).unwrap();
        encoded.push_str("&request_uri=should-not-be-here");

        let err = serde_qs::from_str::<VciAuthorizationRequest>(&encoded).unwrap_err();
        assert!(
            err.to_string().contains("MUST NOT be present"),
            "expected SpecForbidden rejection, got: {err}"
        );
    }

    #[test]
    fn vci_authorization_request_deserialize_scope_example() {
        // Source: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-5.1.2-10>
        let example = "response_type=code&scope=UniversityDegreeCredential&resource=https%3A%2F%2Fcredential-issuer.\
                       example.com&client_id=s6BhdRkqt3&code_challenge=E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM&\
                       code_challenge_method=S256&redirect_uri=https%3A%2F%2Fwallet.example.org%2Fcb";

        let auth_request = serde_qs::from_str::<VciAuthorizationRequest>(example)
            .expect("deserializing VciAuthorizationRequest should succeed");

        assert_eq!(
            auth_request.oauth_request.response_type,
            HashSet::from([ResponseType::Code])
        );
        assert_eq!(auth_request.oauth_request.client_id, "s6BhdRkqt3");
        assert!(auth_request.oauth_request.state.is_none());
        assert_eq!(
            auth_request.redirect_uri.as_ref().as_str(),
            "https://wallet.example.org/cb"
        );
        assert_eq!(
            auth_request.code_challenge,
            PkceCodeChallenge::S256 {
                code_challenge: "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM".to_string()
            }
        );
        assert_eq!(
            auth_request.scope,
            HashSet::from(["UniversityDegreeCredential".parse().unwrap()])
        );
        assert!(auth_request.issuer_state.is_none());
        assert!(auth_request.authorization_details.is_none());
    }

    #[test]
    fn vci_authorization_request_deserialize_authorization_details_example() {
        // Source: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-5.1.1-9>
        let example = "response_type=code&client_id=s6BhdRkqt3&\
                       code_challenge=E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM&code_challenge_method=S256&\
                       authorization_details=%5B%7B%22type%22%3A%20%22openid_credential%22%2C%20%\
                       22credential_configuration_id%22%3A%20%22UniversityDegreeCredential%22%7D%5D&\
                       redirect_uri=https%3A%2F%2Fwallet.example.org%2Fcb";

        let auth_request = serde_qs::from_str::<VciAuthorizationRequest>(example)
            .expect("deserializing VciAuthorizationRequest should succeed");

        assert_eq!(
            auth_request.oauth_request.response_type,
            HashSet::from([ResponseType::Code])
        );
        assert_eq!(auth_request.oauth_request.client_id, "s6BhdRkqt3");
        assert!(auth_request.oauth_request.state.is_none());
        assert_eq!(
            auth_request.redirect_uri.as_ref().as_str(),
            "https://wallet.example.org/cb"
        );
        assert_eq!(
            auth_request.code_challenge,
            PkceCodeChallenge::S256 {
                code_challenge: "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM".to_string()
            }
        );
        assert_eq!(auth_request.scope, HashSet::new());
        assert!(auth_request.issuer_state.is_none());

        let authorization_details = auth_request
            .authorization_details
            .as_ref()
            .expect("authorization_details should be present in Authorization Request");

        let entry = authorization_details
            .as_ref()
            .iter()
            .exactly_one()
            .expect("there should exactly one authorization_details entry");

        assert!(entry.locations.is_none());

        let TypedAuthorizationDetailsEntry::OpenidCredential(vci_entry) = &entry.typed_entry else {
            panic!("authorization details entry should be of type openid_credential");
        };

        assert_eq!(
            vci_entry.credential_configuration_id.as_ref(),
            "UniversityDegreeCredential"
        );
    }
}
