use std::collections::HashSet;

use oauth::authorization::AuthorizationCodeRequest;
pub use oauth::authorization::ResponseType;
use oauth::pkce::PkcePair;
use oauth::scope::Scope;
use serde::Deserialize;
use serde::Serialize;
use serde_with::TryFromInto;
use serde_with::json::JsonString;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use url::Url;

use crate::authorization_details::WalletAuthorizationDetails;
use crate::authorization_details::WalletAuthorizationDetailsEntries;

/// The shared OAuth 2.0 fields that any authorization request — whether for OpenID4VCI issuance or OpenID4VP
/// presentation — must carry.
///
/// Flow-specific variants embed this with `#[serde(flatten)]` and add their own fields.
pub type AuthorizationRequestBase = oauth::authorization::AuthorizationRequestBase<ResponseType>;

/// An OpenID4VCI authorization request, posted in URL-encoded form to the `/par` endpoint
/// (RFC 9126) and later referenced from `/authorize` via [`PushedAuthorizationRequest`].
///
/// This is the OAuth 2.0 authorization code request plus the two parameters OpenID4VCI adds to it. It deliberately
/// does not carry the OpenID Connect `nonce`, which OpenID4VCI does not define for this request; see
/// [`oauth::authorization::OidcAuthorizationRequest`] for that.
#[serde_as]
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VciAuthorizationRequest {
    #[serde(flatten)]
    pub auth_request: AuthorizationCodeRequest,

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
            auth_request: AuthorizationCodeRequest::new(client_id, redirect_uri, state, scope, pkce_pair),
            issuer_state,
            authorization_details: None,
        }
    }
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use itertools::Itertools;
    use oauth::authorization::AuthorizationCodeRequest;
    use oauth::pkce::PkceCodeChallenge;
    use serde_qs;
    use url::Url;

    use super::AuthorizationRequestBase;
    use super::ResponseType;
    use super::VciAuthorizationRequest;

    fn example_vci_request() -> VciAuthorizationRequest {
        let scope = HashSet::from(["openid".parse().unwrap(), "profile".parse().unwrap()]);

        VciAuthorizationRequest {
            auth_request: AuthorizationCodeRequest {
                oauth_request: AuthorizationRequestBase::new(
                    HashSet::from([ResponseType::Code]),
                    "client-123".to_string(),
                    Some("state-abc".to_string()),
                ),
                redirect_uri: Url::parse("https://example.com/callback").unwrap().into(),
                code_challenge: PkceCodeChallenge::S256 {
                    code_challenge: "challenge-xyz".to_string(),
                },
                scope,
            },
            issuer_state: Some("state-xyz".to_string()),
            authorization_details: None,
        }
    }

    #[test]
    fn vci_authorization_request_urlencoded_roundtrip() {
        let request = example_vci_request();

        let encoded = serde_qs::to_string(&request).unwrap();
        let decoded: VciAuthorizationRequest = serde_qs::from_str(&encoded).unwrap();

        assert_eq!(decoded.auth_request.oauth_request.client_id, "client-123");
        assert_eq!(decoded.auth_request.oauth_request.state.as_deref(), Some("state-abc"));
        assert_eq!(
            decoded.auth_request.scope,
            HashSet::from(["openid".parse().unwrap(), "profile".parse().unwrap()])
        );
        assert!(matches!(
            decoded.auth_request.code_challenge,
            PkceCodeChallenge::S256 { code_challenge } if code_challenge == "challenge-xyz"
        ));
        assert_eq!(decoded.issuer_state.as_deref(), Some("state-xyz"));
        assert!(decoded.authorization_details.is_none());
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
            auth_request.auth_request.oauth_request.response_type,
            HashSet::from([ResponseType::Code])
        );
        assert_eq!(auth_request.auth_request.oauth_request.client_id, "s6BhdRkqt3");
        assert!(auth_request.auth_request.oauth_request.state.is_none());
        assert_eq!(
            auth_request.auth_request.redirect_uri.as_ref().as_str(),
            "https://wallet.example.org/cb"
        );
        assert_eq!(
            auth_request.auth_request.code_challenge,
            PkceCodeChallenge::S256 {
                code_challenge: "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM".to_string()
            }
        );
        assert_eq!(
            auth_request.auth_request.scope,
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
            auth_request.auth_request.oauth_request.response_type,
            HashSet::from([ResponseType::Code])
        );
        assert_eq!(auth_request.auth_request.oauth_request.client_id, "s6BhdRkqt3");
        assert!(auth_request.auth_request.oauth_request.state.is_none());
        assert_eq!(
            auth_request.auth_request.redirect_uri.as_ref().as_str(),
            "https://wallet.example.org/cb"
        );
        assert_eq!(
            auth_request.auth_request.code_challenge,
            PkceCodeChallenge::S256 {
                code_challenge: "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM".to_string()
            }
        );
        assert_eq!(auth_request.auth_request.scope, HashSet::new());
        assert!(auth_request.issuer_state.is_none());

        let authorization_details = auth_request
            .authorization_details
            .as_ref()
            .expect("authorization_details should be present in Authorization Request");

        let entry_container = authorization_details
            .as_ref()
            .iter()
            .exactly_one()
            .expect("there should exactly one authorization_details entry");

        assert_eq!(
            entry_container.entry.credential_configuration_id.as_ref(),
            "UniversityDegreeCredential"
        );
    }
}
