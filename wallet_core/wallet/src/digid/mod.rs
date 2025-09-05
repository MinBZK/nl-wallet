use std::hash::Hash;

use url::Url;

use error_category::ErrorCategory;
use http_utils::reqwest::IntoPinnedReqwestClient;
use http_utils::tls::pinning::TlsPinningConfig;
use openid4vc::oidc::OidcError;
use openid4vc::token::TokenRequest;
use wallet_configuration::wallet_config::DigidConfiguration;

pub use app2app::App2AppErrorMessage;
pub use http::HttpDigidClient;

mod app2app;
mod http;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum DigidError {
    #[error("HTTP error: {0}")]
    #[category(critical)] // DigiD/OIDC urls do not contain sensitive data
    Http(#[from] reqwest::Error),

    #[error("OIDC error: {0}")]
    Oidc(#[from] OidcError),

    #[error("missing location header")]
    #[category(critical)]
    MissingLocation,

    #[error("cannot parse location header to str: {0}")]
    #[category(pd)]
    HeaderNotAStr(#[from] ::http::header::ToStrError),

    #[error("cannot parse location header to URL: {0}")]
    #[category(pd)]
    NotAUrl(#[from] url::ParseError),

    #[error("expected HTTP 307 Temporary Redirect, got: {0}")]
    #[category(critical)]
    ExpectedRedirect(::http::StatusCode),

    #[error("missing query in location header")]
    #[category(critical)]
    MissingLocationQuery,

    #[error("cannot deserialize from URL query: {0}")]
    #[category(pd)]
    UrlDeserialize(#[from] serde_urlencoded::de::Error),

    #[error("cannot serialize to URL query: {0}")]
    #[category(pd)]
    UrlSerialize(#[from] serde_urlencoded::ser::Error),

    #[error("error in app2app response: {0}")]
    #[category(pd)]
    App2AppError(App2AppErrorMessage),
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

#[cfg(test)]
mod test {
    use base64::prelude::*;
    use serde::Serialize;

    pub(super) fn base64<T: Serialize>(input: T) -> String {
        BASE64_URL_SAFE.encode(serde_json::to_string(&input).unwrap())
    }
}
