use std::hash::Hash;

use url::Url;

use error_category::ErrorCategory;
use http_utils::reqwest::IntoPinnedReqwestClient;
use http_utils::tls::pinning::TlsPinningConfig;
use openid4vc::oidc::OidcClient;
use openid4vc::oidc::OidcError;
use openid4vc::token::TokenRequest;
use wallet_configuration::wallet_config::DigidConfiguration;

pub use http::HttpDigidClient;

mod http;

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

#[cfg_attr(any(test, feature = "test"), mockall::automock(type OC = openid4vc::oidc::MockOidcClient;))]
pub trait DigidClient<C = TlsPinningConfig>
where
    C: IntoPinnedReqwestClient + Clone + Hash,
{
    type OC: OidcClient;

    async fn start_session(
        &self,
        digid_config: DigidConfiguration,
        http_config: C,
        redirect_uri: Url,
    ) -> Result<DigidSessionState<Self::OC>, DigidError>;
}

#[cfg(test)]
pub mod mock {
    use openid4vc::oidc::MockOidcClient;

    use super::*;

    pub const AUTH_URL: &str = "http://example.com/auth";

    pub fn mock_digid_session_state() -> DigidSessionState<MockOidcClient> {
        DigidSessionState {
            oidc_client: MockOidcClient::new(),
            auth_url: Url::parse("http://example.com/auth").unwrap(),
        }
    }
}
