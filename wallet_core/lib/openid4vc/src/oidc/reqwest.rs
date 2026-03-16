use derive_more::AsRef;
use reqwest::RequestBuilder;
use reqwest::Response;
use serde::de::DeserializeOwned;
use url::Url;

use crypto::trust_anchor::BorrowingTrustAnchor;
use http_utils::reqwest::client_builder_accept_json;
use http_utils::reqwest::default_reqwest_client_builder;
use http_utils::reqwest::trusted_reqwest_client_builder;

#[derive(Debug, Clone, AsRef)]
pub struct OidcReqwestClient(reqwest::Client);

impl OidcReqwestClient {
    pub fn try_new() -> Result<Self, reqwest::Error> {
        let client = client_builder_accept_json(default_reqwest_client_builder()).build()?;

        Ok(OidcReqwestClient(client))
    }

    pub fn try_new_with_borrowing_trust_anchors(
        trust_anchors: impl IntoIterator<Item = BorrowingTrustAnchor>,
    ) -> Result<Self, reqwest::Error> {
        let trust_anchors = trust_anchors
            .into_iter()
            .map(|a| reqwest::Certificate::from_der(a.as_ref()))
            .collect::<Result<Vec<_>, _>>()?;

        Self::try_new_with_trust_anchors(trust_anchors)
    }

    pub fn try_new_with_trust_anchors(
        trust_anchors: impl IntoIterator<Item = reqwest::Certificate>,
    ) -> Result<Self, reqwest::Error> {
        let client = client_builder_accept_json(trusted_reqwest_client_builder(trust_anchors)).build()?;

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
