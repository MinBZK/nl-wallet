use std::hash::Hash;

use tracing::info;
use url::Url;

use error_category::ErrorCategory;
use http_utils::reqwest::IntoReqwestClient;
use openid4vc::oidc::OidcClient;
use openid4vc::oidc::OidcError;
use openid4vc::oidc::OidcReqwestClient;
use openid4vc::token::TokenRequest;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum OidcSessionError {
    #[error("HTTP error: {0}")]
    #[category(critical)] // DigiD/OIDC urls do not contain sensitive data
    Http(#[from] reqwest::Error),

    #[error("OIDC error: {0}")]
    Oidc(#[from] OidcError),
}

/// The state of an OIDC authorization code flow session after OIDC discovery.
/// Contains the OIDC client (for token exchange) and the authorization URL.
#[derive(Debug)]
pub struct OidcSession<O: OidcClient> {
    pub oidc_client: O,
    pub auth_url: Url,
}

impl<O: OidcClient> OidcSession<O> {
    pub fn into_token_request(self, redirect_uri: &Url) -> Result<TokenRequest, OidcSessionError> {
        let token_request = self.oidc_client.into_token_request(redirect_uri)?;
        Ok(token_request)
    }
}

pub async fn start_oidc_session<O, C>(
    client_id: String,
    http_config: C,
    redirect_uri: Url,
) -> Result<OidcSession<O>, OidcSessionError>
where
    C: IntoReqwestClient + Clone + Hash,
    O: OidcClient,
{
    let http_client = OidcReqwestClient::try_new(http_config)?;
    let (oidc_client, auth_url) = O::start(&http_client, client_id, redirect_uri).await?;

    info!("DigiD auth URL generated");

    Ok(OidcSession { oidc_client, auth_url })
}

#[cfg(test)]
pub mod mock {
    use openid4vc::oidc::MockOidcClient;

    use super::*;

    pub const AUTH_URL: &str = "http://example.com/auth";

    pub fn mock_oidc_session() -> OidcSession<MockOidcClient> {
        OidcSession {
            oidc_client: MockOidcClient::new(),
            auth_url: Url::parse(AUTH_URL).unwrap(),
        }
    }

    pub fn mock_oidc_session_tuple() -> (MockOidcClient, Url) {
        let OidcSession { oidc_client, auth_url } = mock_oidc_session();
        (oidc_client, auth_url)
    }
}

#[cfg(test)]
mod test {
    use http_utils::client::TlsPinningConfig;
    use serial_test::serial;
    use url::Url;

    use openid4vc::oidc::MockOidcClient;
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
    #[serial(MockOidcClient)]
    async fn test_start_session() {
        // TODO: remove `start_context` and `#[serial(MockOidcClient)]` when implementing ACF (PVW-5575)
        let oidc_client = MockOidcClient::start_context();
        oidc_client
            .expect()
            .return_once(|_, _, _| Ok((MockOidcClient::default(), Url::parse("https://example.com/").unwrap())));

        let session: OidcSession<MockOidcClient> = start_oidc_session(
            String::from("client_id"),
            TlsPinningConfig::try_new("https://digid.example.com".parse().unwrap(), vec![]).unwrap(),
            "https://app.example.com".parse().unwrap(),
        )
        .await
        .expect("starting DigiD session should succeed");

        assert_eq!(session.auth_url, "https://example.com/".parse().unwrap());
    }

    #[tokio::test]
    async fn test_into_token_request() {
        let mut oidc_client = MockOidcClient::default();
        oidc_client
            .expect_into_token_request()
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
