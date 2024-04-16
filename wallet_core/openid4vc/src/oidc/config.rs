//! OpenID discovery, loosely based on https://crates.io/crates/openid.

use biscuit::{jwk::JWKSet, Empty};
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;
use wallet_common::config::wallet_config::BaseUrl;

use super::OidcError;

/// OpenID metadata as defind by https://openid.net/specs/openid-connect-discovery-1_0.html,
/// to be published at `.well-known/openid-configuration`.
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub issuer: BaseUrl, // not a cannot_be_a_base URL
    pub authorization_endpoint: Url,
    pub token_endpoint: Url,
    #[serde(default)]
    pub userinfo_endpoint: Option<Url>,
    pub jwks_uri: Url,
    #[serde(default)]
    pub registration_endpoint: Option<Url>,
    #[serde(default)]
    pub scopes_supported: Option<IndexSet<String>>,
    // There are only three valid response types, plus combinations of them, and none
    // If we want to make these user friendly we want a struct to represent all 7 types
    pub response_types_supported: IndexSet<String>,
    // There are only two possible values here, query and fragment. Default is both.
    #[serde(default)]
    pub response_modes_supported: Option<IndexSet<String>>,
    // Must support at least authorization_code and implicit.
    #[serde(default)]
    pub grant_types_supported: Option<IndexSet<String>>,
    #[serde(default)]
    pub acr_values_supported: Option<IndexSet<String>>,
    // pairwise and public are valid by spec, but servers can add more
    #[serde(default = "IndexSet::new")]
    pub subject_types_supported: IndexSet<String>,
    // Must include at least RS256, none is only allowed with response types without id tokens
    #[serde(default = "IndexSet::new")]
    pub id_token_signing_alg_values_supported: IndexSet<String>,
    #[serde(default)]
    pub id_token_encryption_alg_values_supported: Option<IndexSet<String>>,
    #[serde(default)]
    pub id_token_encryption_enc_values_supported: Option<IndexSet<String>>,
    #[serde(default)]
    pub userinfo_signing_alg_values_supported: Option<IndexSet<String>>,
    #[serde(default)]
    pub userinfo_encryption_alg_values_supported: Option<IndexSet<String>>,
    #[serde(default)]
    pub userinfo_encryption_enc_values_supported: Option<IndexSet<String>>,
    #[serde(default)]
    pub request_object_signing_alg_values_supported: Option<IndexSet<String>>,
    #[serde(default)]
    pub request_object_encryption_alg_values_supported: Option<IndexSet<String>>,
    #[serde(default)]
    pub request_object_encryption_enc_values_supported: Option<IndexSet<String>>,
    // Spec options are client_secret_post, client_secret_basic, client_secret_jwt, private_key_jwt
    // If omitted, client_secret_basic is used
    #[serde(default)]
    pub token_endpoint_auth_methods_supported: Option<IndexSet<String>>,
    // Only wanted with jwt auth methods, should have RS256, none not allowed
    #[serde(default)]
    pub token_endpoint_auth_signing_alg_values_supported: Option<IndexSet<String>>,
    #[serde(default)]
    pub display_values_supported: Option<IndexSet<String>>,
    // Valid options are normal, aggregated, and distributed. If omitted, only use normal
    #[serde(default)]
    pub claim_types_supported: Option<IndexSet<String>>,
    #[serde(default)]
    pub claims_supported: Option<IndexSet<String>>,
    #[serde(default)]
    pub service_documentation: Option<Url>,
    #[serde(default)]
    pub claims_locales_supported: Option<IndexSet<String>>,
    #[serde(default)]
    pub ui_locales_supported: Option<IndexSet<String>>,
    #[serde(default)]
    pub claims_parameter_supported: bool,
    #[serde(default)]
    pub request_parameter_supported: bool,
    #[serde(default = "bool_value::<true>")]
    pub request_uri_parameter_supported: bool,
    #[serde(default)]
    pub require_request_uri_registration: bool,

    #[serde(default)]
    pub op_policy_uri: Option<Url>,
    #[serde(default)]
    pub op_tos_uri: Option<Url>,
    // This is a NONSTANDARD extension Google uses that is a part of the Oauth discovery draft
    #[serde(default)]
    pub code_challenge_methods_supported: Option<IndexSet<String>>,
}

impl Config {
    pub async fn discover(client: &reqwest::Client, issuer: &BaseUrl) -> Result<Self, OidcError> {
        // If the Issuer value contains a path component, any terminating / MUST be removed before
        // appending /.well-known/openid-configuration.
        let oidc_conf_url = issuer.join(".well-known/openid-configuration");

        let resp = client.get(oidc_conf_url).send().await?.error_for_status()?;
        resp.json().await.map_err(OidcError::from)
    }

    /// Get the JWK set from the given Url. Errors are either a reqwest error or an Insecure error if
    /// the url isn't https.
    pub(crate) async fn jwks(&self, client: &reqwest::Client) -> Result<JWKSet<Empty>, OidcError> {
        let resp = client.get(self.jwks_uri.as_ref()).send().await?.error_for_status()?;
        resp.json().await.map_err(OidcError::from)
    }

    /// Construct a new `Config` based on the OP's URL and some standardized or reasonable defaults.
    #[cfg(any(test, feature = "mock"))]
    pub fn new_mock(issuer: &BaseUrl) -> Self {
        Self {
            issuer: issuer.clone(),
            authorization_endpoint: issuer.join("/authorize"),
            token_endpoint: issuer.join("/token"),
            userinfo_endpoint: Some(issuer.join("/userinfo")),
            jwks_uri: issuer.join("/jwks.json"),
            registration_endpoint: None,
            scopes_supported: Some(IndexSet::from_iter(["openid".to_string()])),
            response_types_supported: IndexSet::from_iter(
                ["code", "code id_token", "id_token", "id_token token"].map(str::to_string),
            ),
            response_modes_supported: None,
            grant_types_supported: None,
            acr_values_supported: None,
            subject_types_supported: IndexSet::new(),
            id_token_signing_alg_values_supported: IndexSet::from_iter(["RS256".to_string()]),
            id_token_encryption_alg_values_supported: None,
            id_token_encryption_enc_values_supported: None,
            userinfo_signing_alg_values_supported: None,
            userinfo_encryption_alg_values_supported: None,
            userinfo_encryption_enc_values_supported: None,
            request_object_signing_alg_values_supported: None,
            request_object_encryption_alg_values_supported: None,
            request_object_encryption_enc_values_supported: None,
            token_endpoint_auth_methods_supported: None,
            token_endpoint_auth_signing_alg_values_supported: None,
            display_values_supported: None,
            claim_types_supported: None,
            claims_supported: None,
            service_documentation: None,
            claims_locales_supported: None,
            ui_locales_supported: None,
            claims_parameter_supported: false,
            request_parameter_supported: false,
            request_uri_parameter_supported: false,
            require_request_uri_registration: false,
            op_policy_uri: None,
            op_tos_uri: None,
            code_challenge_methods_supported: None,
        }
    }
}

const fn bool_value<const B: bool>() -> bool {
    B
}

#[cfg(test)]
pub mod tests {
    use serde_json::json;
    use wallet_common::config::wallet_config::BaseUrl;
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use super::Config;

    pub async fn start_discovery_server() -> (MockServer, BaseUrl) {
        let server = MockServer::start().await;
        let server_url: BaseUrl = server.uri().parse().unwrap();

        // Mock OpenID configuration endpoint
        Mock::given(method("GET"))
            .and(path("/.well-known/openid-configuration"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "issuer": server_url,
                "authorization_endpoint": server_url.join("/oauth2/authorize"),
                "token_endpoint": server_url.join("/oauth2/token"),
                "jwks_uri": server_url.join("/.well-known/jwks.json"),
                "response_types_supported": ["code", "id_token", "token id_token"],
                "scopes_supported": ["openid"],
            })))
            .expect(1)
            .mount(&server)
            .await;

        // Mock JWKS endpoint
        Mock::given(method("GET"))
            .and(path("/.well-known/jwks.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "keys": []
            })))
            .expect(1)
            .mount(&server)
            .await;

        (server, server_url)
    }

    #[tokio::test]
    async fn test_discovery() {
        let (_server, server_url) = start_discovery_server().await;
        let http_client = reqwest::Client::new();

        let discovered = Config::discover(&http_client, &server_url).await.unwrap();

        assert_eq!(&discovered.issuer, &server_url);
        assert_eq!(
            &discovered.authorization_endpoint,
            &server_url.join("/oauth2/authorize")
        );

        let jwks = discovered.jwks(&http_client).await.unwrap();
        assert!(jwks.keys.is_empty());
    }
}
