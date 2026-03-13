use tracing::info;
use url::Url;

use error_category::ErrorCategory;
use openid4vc::issuer_identifier::IssuerIdentifier;
use openid4vc::issuer_metadata::IssuerMetadataDiscoveryError;
use openid4vc::oidc::AuthorizationServer;
use openid4vc::oidc::OidcDiscovery;
use openid4vc::oidc::OidcError;
use openid4vc::token::TokenRequest;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum OidcSessionError {
    #[error("HTTP error: {0}")]
    #[category(critical)] // DigiD/OIDC urls do not contain sensitive data
    Http(#[from] reqwest::Error),

    #[error("OIDC error: {0}")]
    Oidc(#[from] OidcError),

    #[error("issuer metadata error: {0}")]
    #[category(expected)]
    IssuerMetadata(#[from] IssuerMetadataDiscoveryError),
}

/// The state of an OIDC authorization code flow session after OIDC discovery.
/// Contains the authorization server (for token exchange) and the authorization URL.
#[derive(Debug)]
pub struct OidcSession<S: AuthorizationServer> {
    pub oidc_client: S,
    pub auth_url: Url,
}

impl<S: AuthorizationServer> OidcSession<S> {
    pub fn into_token_request(self, redirect_uri: &Url) -> Result<TokenRequest, OidcSessionError> {
        let token_request = self.oidc_client.into_token_request(redirect_uri)?;

        Ok(token_request)
    }
}

pub async fn start_oidc_session<OD>(
    oidc_discovery: &OD,
    auth_server: &IssuerIdentifier,
    client_id: String,
    redirect_uri: Url,
) -> Result<OidcSession<OD::Server>, OidcSessionError>
where
    OD: OidcDiscovery,
{
    let (oidc_client, auth_url) = oidc_discovery.discover(auth_server, client_id, redirect_uri).await?;

    info!("OIDC auth URL generated");

    Ok(OidcSession { oidc_client, auth_url })
}

#[cfg(test)]
pub mod mock {
    use openid4vc::oidc::MockAuthorizationServer;

    use super::*;

    pub const AUTH_URL: &str = "http://example.com/auth";

    pub fn mock_oidc_session() -> OidcSession<MockAuthorizationServer> {
        OidcSession {
            oidc_client: MockAuthorizationServer::new(),
            auth_url: Url::parse(AUTH_URL).unwrap(),
        }
    }

    pub fn mock_oidc_session_tuple() -> (MockAuthorizationServer, Url) {
        let OidcSession { oidc_client, auth_url } = mock_oidc_session();
        (oidc_client, auth_url)
    }
}

#[cfg(test)]
mod test {
    use url::Url;

    use openid4vc::oidc::MockAuthorizationServer;
    use openid4vc::oidc::MockOidcDiscovery;
    use openid4vc::token::TokenRequest;
    use openid4vc::token::TokenRequestGrantType;

    use crate::oidc_session::OidcSession;

    use super::start_oidc_session;

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
    async fn test_start_session() {
        let mut mock_oidc = MockOidcDiscovery::new();
        mock_oidc.expect_start_sync().return_once(|_, _, _| {
            Ok((
                MockAuthorizationServer::default(),
                Url::parse("https://example.com/").unwrap(),
            ))
        });

        let session: OidcSession<MockAuthorizationServer> = start_oidc_session(
            &mock_oidc,
            &"https://digid.example.com/".parse().unwrap(),
            String::from("client_id"),
            "https://app.example.com".parse().unwrap(),
        )
        .await
        .expect("starting OIDC session should succeed");

        assert_eq!(session.auth_url, "https://example.com/".parse().unwrap());
    }

    #[tokio::test]
    async fn test_into_token_request() {
        let mut oidc_client = MockAuthorizationServer::default();
        oidc_client
            .expect_token_request()
            .return_once(|_| Ok(default_token_request()));

        let session = super::OidcSession {
            oidc_client,
            auth_url: "https://example.com/".parse().unwrap(),
        };

        let token_request =
            session.into_token_request(&"https://example.com/deeplink/return-from-digid".parse().unwrap());

        assert!(token_request.is_ok());
    }
}
