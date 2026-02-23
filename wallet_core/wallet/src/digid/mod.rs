use std::hash::Hash;

use url::Url;

use error_category::ErrorCategory;
use http_utils::reqwest::IntoPinnedReqwestClient;
use http_utils::tls::pinning::TlsPinningConfig;
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

#[cfg_attr(any(test, feature = "test"), mockall::automock(type Session = MockDigidSession<C>;))]
pub trait DigidClient<C = TlsPinningConfig>
where
    C: IntoPinnedReqwestClient + Clone + Hash,
{
    type Session: DigidSession<C>;

    async fn start_session(
        &self,
        digid_config: DigidConfiguration,
        http_config: C,
        redirect_uri: Url,
    ) -> Result<Self::Session, DigidError>;
}

#[cfg_attr(any(test, feature = "test"), mockall::automock)]
pub trait DigidSession<C = TlsPinningConfig>
where
    C: IntoPinnedReqwestClient + Clone + Hash,
{
    fn auth_url(&self) -> &Url;

    async fn into_token_request(self, http_config: &C, redirect_uri: Url) -> Result<TokenRequest, DigidError>;
}
