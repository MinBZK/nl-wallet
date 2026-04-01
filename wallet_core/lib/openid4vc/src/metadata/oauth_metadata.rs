//! OAuth 2.0 Authorization Server Metadata, loosely based on https://crates.io/crates/openid.

use indexmap::IndexSet;
use serde::Deserialize;
use serde::Serialize;
use serde_with::skip_serializing_none;
use url::Url;

use crate::issuer_identifier::IssuerIdentifier;
use crate::metadata::well_known::WellKnownMetadata;

/// OAuth 2.0 Authorization Server Metadata as defined by [RFC 8414](https://www.rfc-editor.org/rfc/rfc8414),
/// to be published at `.well-known/oauth-authorization-server`, and a superset of the OpenID Connect
/// Discovery 1.0 metadata published at `.well-known/openid-configuration`.
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AuthorizationServerMetadata {
    pub issuer: IssuerIdentifier,
    #[serde(default)]
    pub authorization_endpoint: Option<Url>,
    pub token_endpoint: Url,
    #[serde(default)]
    pub userinfo_endpoint: Option<Url>,
    #[serde(default)]
    pub jwks_uri: Option<Url>,
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

impl AuthorizationServerMetadata {
    /// Returns a new instance with the specified URLs, and all other parameters set to none/empty/false.
    pub fn new(issuer: IssuerIdentifier, token_endpoint: Url) -> Self {
        Self {
            issuer,
            authorization_endpoint: None,
            token_endpoint,
            userinfo_endpoint: None,
            jwks_uri: None,
            registration_endpoint: None,
            scopes_supported: None,
            response_types_supported: IndexSet::new(),
            response_modes_supported: None,
            grant_types_supported: None,
            acr_values_supported: None,
            subject_types_supported: IndexSet::new(),
            id_token_signing_alg_values_supported: IndexSet::new(),
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

impl WellKnownMetadata for AuthorizationServerMetadata {
    fn issuer_identifier(&self) -> &IssuerIdentifier {
        &self.issuer
    }
}

const fn bool_value<const B: bool>() -> bool {
    B
}

#[cfg(feature = "mock")]
pub mod mock {
    use url::Url;

    use crate::metadata::oauth_metadata::AuthorizationServerMetadata;

    impl AuthorizationServerMetadata {
        pub fn new_with_auth_url(auth_url: &str) -> Self {
            AuthorizationServerMetadata {
                authorization_endpoint: Some(Url::parse(auth_url).unwrap()),
                jwks_uri: Some(Url::parse(auth_url).unwrap()),
                ..AuthorizationServerMetadata::new("http://example.com".parse().unwrap(), Url::parse(auth_url).unwrap())
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use http_utils::reqwest::default_reqwest_client_builder;
    use http_utils::urls::BaseUrl;
    use serde_json::json;
    use wiremock::Mock;
    use wiremock::MockServer;
    use wiremock::ResponseTemplate;
    use wiremock::matchers::method;
    use wiremock::matchers::path;

    use super::AuthorizationServerMetadata;
    use http_utils::reqwest::HttpJsonClient;

    pub async fn start_discovery_server() -> (MockServer, BaseUrl) {
        let server = MockServer::start().await;
        let server_url: BaseUrl = server.uri().parse().unwrap();

        // Mock OpenID configuration endpoint
        Mock::given(method("GET"))
            .and(path("/.well-known/openid-configuration"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "issuer": "https://example.com/",
                "authorization_endpoint": server_url.join("/oauth2/authorize"),
                "token_endpoint": server_url.join("/oauth2/token"),
                "jwks_uri": server_url.join("/.well-known/jwks.json"),
                "response_types_supported": ["code", "id_token", "token id_token"],
                "scopes_supported": ["openid"],
            })))
            .expect(1)
            .mount(&server)
            .await;

        (server, server_url)
    }

    #[tokio::test]
    async fn test_discovery() {
        let (_server, server_url) = start_discovery_server().await;
        let client = HttpJsonClient::try_new(default_reqwest_client_builder()).unwrap();
        let discovery_url = server_url.join(".well-known/openid-configuration");

        let discovered: AuthorizationServerMetadata = client.get(discovery_url).await.unwrap();

        assert_eq!(discovered.issuer.as_ref(), "https://example.com/");
    }
}
