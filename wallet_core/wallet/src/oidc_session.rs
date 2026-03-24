use tracing::info;
use url::Url;

use error_category::ErrorCategory;
use openid4vc::oauth::AuthorizationServer;
use openid4vc::oauth::AuthorizationServerMetadata;
use openid4vc::oauth::HttpAuthorizationServer;
use openid4vc::oauth::OAuthError;
use openid4vc::token::TokenRequest;
use openid4vc::well_known::WellKnownError;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum OidcSessionError {
    #[error("HTTP error: {0}")]
    #[category(critical)] // DigiD/OIDC urls do not contain sensitive data
    Http(#[from] reqwest::Error),

    #[error("OIDC error: {0}")]
    Oidc(#[from] OAuthError),

    #[error("issuer metadata error: {0}")]
    #[category(expected)]
    IssuerMetadata(#[from] WellKnownError),
}

/// The state of an OIDC authorization code flow session after OIDC discovery.
/// Contains the authorization server (for token exchange) and the authorization URL.
#[derive(Debug)]
pub struct OidcSession<S: AuthorizationServer> {
    pub http_client: S,
    pub auth_url: Url,
}

impl<S: AuthorizationServer> OidcSession<S> {
    pub fn into_token_request(self, redirect_uri: &Url) -> Result<TokenRequest, OidcSessionError> {
        let token_request = self.http_client.into_token_request(redirect_uri)?;

        Ok(token_request)
    }
}

pub fn build_oidc_session(
    config: AuthorizationServerMetadata,
    client_id: String,
    redirect_uri: Url,
) -> Result<OidcSession<HttpAuthorizationServer>, OidcSessionError> {
    let http_client = HttpAuthorizationServer::new(config, None, client_id, redirect_uri);
    let auth_url = http_client.auth_url()?;

    info!("OIDC auth URL generated");

    Ok(OidcSession { http_client, auth_url })
}

#[cfg(test)]
mod test {
    use openid4vc::oauth::MockAuthorizationServer;
    use openid4vc::token::TokenRequest;
    use openid4vc::token::TokenRequestGrantType;

    fn default_token_request() -> TokenRequest {
        TokenRequest {
            grant_type: TokenRequestGrantType::PreAuthorizedCode {
                pre_authorized_code: "".to_owned().into(),
            },
            code_verifier: Default::default(),
            client_id: Default::default(),
            redirect_uri: Default::default(),
        }
    }

    #[tokio::test]
    async fn test_into_token_request() {
        let mut http_client = MockAuthorizationServer::new();
        http_client
            .expect_into_token_request()
            .return_once(|_| Ok(default_token_request()));

        let session = super::OidcSession {
            http_client,
            auth_url: "https://example.com/".parse().unwrap(),
        };

        let token_request =
            session.into_token_request(&"https://example.com/deeplink/return-from-digid".parse().unwrap());

        assert!(token_request.is_ok());
    }
}
