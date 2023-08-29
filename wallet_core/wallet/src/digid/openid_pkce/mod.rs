mod auth_url;
mod authenticate;
mod validate_fix;

use openid::{error::Error, Config};
use url::Url;

/// This wraps `openid::Client` in order to add some enhancements and fixes.
pub struct Client(openid::Client);

// Forward some methods directly to openid::Client
impl Client {
    pub async fn discover_with_client(
        http_client: reqwest::Client,
        id: String,
        secret: impl Into<Option<String>>,
        redirect: impl Into<Option<String>>,
        issuer: Url,
    ) -> Result<Self, Error> {
        let client = openid::Client::discover_with_client(http_client, id, secret, redirect, issuer).await?;
        let client = Client(client);

        Ok(client)
    }

    pub fn config(&self) -> &Config {
        self.0.config()
    }

    pub fn redirect_url(&self) -> &str {
        self.0.redirect_url()
    }
}
