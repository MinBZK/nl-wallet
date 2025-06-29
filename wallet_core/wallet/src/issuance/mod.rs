mod app2app;

#[cfg(test)]
pub mod mock;

use url::Url;

use error_category::ErrorCategory;
use http_utils::reqwest::IntoPinnedReqwestClient;
use openid4vc::oidc::OidcError;
use openid4vc::token::TokenRequest;
use wallet_configuration::wallet_config::DigidConfiguration;

pub use app2app::App2AppErrorMessage;
pub use app2app::HttpDigidSession;

pub const PID_DOCTYPE: &str = "urn:eudi:pid:nl:1";
pub const BSN_ATTR_NAME: &str = "bsn";

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum DigidSessionError {
    #[error("OIDC error: {0}")]
    Oidc(#[from] OidcError),
    #[error("HTTP error: {0}")]
    #[category(critical)] // DigiD/OIDC urls do not contain sensitive data
    Http(#[from] reqwest::Error),
    #[error("missing location header")]
    #[category(critical)]
    MissingLocation,
    #[error("cannot parse location header to str: {0}")]
    #[category(pd)]
    HeaderNotAStr(#[from] http::header::ToStrError),
    #[error("cannot parse location header to URL: {0}")]
    #[category(pd)]
    NotAUrl(#[from] url::ParseError),
    #[error("missing query in location header")]
    #[category(critical)]
    MissingLocationQuery,
    #[error("expected HTTP 307 Temporary Redirect, got: {0}")]
    #[category(critical)]
    ExpectedRedirect(http::StatusCode),
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

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait DigidSession {
    async fn start<C>(
        digid_config: DigidConfiguration,
        http_config: C,
        redirect_uri: Url,
    ) -> Result<(Self, Url), DigidSessionError>
    where
        C: IntoPinnedReqwestClient + Clone + 'static,
        Self: Sized;

    async fn into_token_request(self, received_redirect_uri: Url) -> Result<TokenRequest, DigidSessionError>;
}
