use std::hash::Hash;

use tracing::info;
use url::Url;

use error_category::ErrorCategory;
use http_utils::reqwest::IntoReqwestClient;
use openid4vc::oidc::OidcClient;
use openid4vc::oidc::OidcError;
use openid4vc::oidc::OidcReqwestClient;
use openid4vc::token::TokenRequest;
use wallet_configuration::wallet_config::DigidConfiguration;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum DigidError {
    #[error("HTTP error: {0}")]
    #[category(critical)] // DigiD/OIDC urls do not contain sensitive data
    Http(#[from] reqwest::Error),

    #[error("OIDC error: {0}")]
    Oidc(#[from] OidcError),
}

/// The state of a DigiD session after OIDC discovery.
/// Contains the OIDC client (for token exchange) and the authorization URL.
#[derive(Debug)]
pub struct DigidSessionState<O: OidcClient> {
    pub oidc_client: O,
    pub auth_url: Url,
}

impl<O: OidcClient> DigidSessionState<O> {
    pub fn into_token_request(self, redirect_uri: &Url) -> Result<TokenRequest, DigidError> {
        let token_request = self.oidc_client.into_token_request(redirect_uri)?;
        Ok(token_request)
    }
}

pub async fn start_digid_session<O, C>(
    digid_config: DigidConfiguration,
    http_config: C,
    redirect_uri: Url,
) -> Result<DigidSessionState<O>, DigidError>
where
    C: IntoReqwestClient + Clone + Hash,
    O: OidcClient,
{
    let http_client = OidcReqwestClient::try_new(http_config)?;
    let (oidc_client, auth_url) = O::start(&http_client, digid_config.client_id, redirect_uri).await?;

    info!("DigiD auth URL generated");

    Ok(DigidSessionState { oidc_client, auth_url })
}

#[cfg(test)]
pub mod mock {
    use openid4vc::oidc::MockOidcClient;

    use super::*;

    pub const AUTH_URL: &str = "http://example.com/auth";

    pub fn mock_digid_session_state() -> DigidSessionState<MockOidcClient> {
        DigidSessionState {
            oidc_client: MockOidcClient::new(),
            auth_url: Url::parse(AUTH_URL).unwrap(),
        }
    }

    pub fn mock_digid_session_state_tuple() -> (MockOidcClient, Url) {
        let DigidSessionState { oidc_client, auth_url } = mock_digid_session_state();
        (oidc_client, auth_url)
    }
}

#[cfg(test)]
mod test {
    use http_utils::reqwest::test::get_tls_pinning_config_for_url;
    use serial_test::serial;
    use url::Url;

    use openid4vc::oidc::MockOidcClient;
    use openid4vc::token::TokenRequest;
    use openid4vc::token::TokenRequestGrantType;
    use wallet_configuration::wallet_config::DigidConfiguration;

    use crate::digid::DigidSessionState;

    use super::start_digid_session;

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

        let session: DigidSessionState<MockOidcClient> = start_digid_session(
            DigidConfiguration::default(),
            get_tls_pinning_config_for_url("https://digid.example.com"),
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

        let session = super::DigidSessionState {
            oidc_client,
            auth_url: "https://example.com/".parse().unwrap(),
        };

        let token_request =
            session.into_token_request(&"https://example.com/deeplink/return-from-digid".parse().unwrap());

        assert!(token_request.is_ok());
    }
}
