use derive_more::AsRef;
use reqwest::RequestBuilder;
use reqwest::Response;
use serde::de::DeserializeOwned;
use url::Url;

use http_utils::reqwest::client_builder_accept_json;
use http_utils::reqwest::default_reqwest_client_builder;

#[derive(Debug, Clone, AsRef)]
pub struct OidcReqwestClient(reqwest::Client);

impl OidcReqwestClient {
    pub fn try_new() -> Result<Self, reqwest::Error> {
        let client = client_builder_accept_json(default_reqwest_client_builder()).build()?;

        Ok(OidcReqwestClient(client))
    }

    pub async fn get<T: DeserializeOwned>(&self, url: Url) -> Result<T, reqwest::Error> {
        self.0.get(url).send().await?.error_for_status()?.json().await
    }

    pub async fn post<F>(&self, url: Url, adapter: F) -> Result<Response, reqwest::Error>
    where
        F: FnOnce(RequestBuilder) -> RequestBuilder,
    {
        adapter(self.0.post(url)).send().await
    }
}
