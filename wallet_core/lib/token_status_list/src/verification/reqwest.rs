use derive_more::AsRef;
use http::header;

use http_utils::reqwest::default_reqwest_client_builder;
use http_utils::urls::HttpsUri;

use crate::status_list_token::StatusListToken;
use crate::verification::client::StatusListClient;
use crate::verification::client::StatusListClientError;

const STATUS_LIST_JWT_ACCEPT: &str = "application/statuslist+jwt";

#[derive(Debug, Clone, AsRef)]
pub struct StatusListReqwestClient(reqwest::Client);

impl StatusListReqwestClient {
    pub fn new() -> Result<Self, reqwest::Error> {
        Ok(Self(default_reqwest_client_builder().build()?))
    }
}

impl StatusListClient for StatusListReqwestClient {
    async fn fetch(&self, uri: &HttpsUri) -> Result<StatusListToken, StatusListClientError> {
        let response = self
            .as_ref()
            .get(uri.to_string())
            .header(header::ACCEPT, STATUS_LIST_JWT_ACCEPT)
            .send()
            .await?;

        let status_list_token = response.text().await?.parse()?;

        Ok(status_list_token)
    }
}
