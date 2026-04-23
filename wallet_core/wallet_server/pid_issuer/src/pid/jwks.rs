use derive_more::Constructor;
use http_utils::reqwest::HttpJsonClient;
use jsonwebtoken::jwk::JwkSet;
use url::Url;

#[derive(Debug, thiserror::Error)]
pub enum JwksError {
    #[error("error fetching jwks: {0}")]
    Http(#[from] reqwest::Error),
}

#[derive(Debug, Clone, Constructor)]
pub struct HttpJwksClient {
    client: HttpJsonClient,
}

impl HttpJwksClient {
    pub async fn jwks(&self, uri: Url) -> Result<JwkSet, JwksError> {
        let jwks = self.client.get(uri).await?;
        Ok(jwks)
    }
}
