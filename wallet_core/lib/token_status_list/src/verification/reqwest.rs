use derive_more::AsRef;
use http::header;
use reqwest::ClientBuilder;
use url::Url;

use crate::status_list_token::StatusListToken;
use crate::verification::client::StatusListClient;
use crate::verification::client::StatusListClientError;

const STATUS_LIST_JWT_ACCEPT: &str = "application/statuslist+jwt";

#[derive(Debug, Clone, AsRef)]
pub struct HttpStatusListClient(reqwest::Client);

impl HttpStatusListClient {
    pub fn new(client_builder: ClientBuilder) -> Result<Self, reqwest::Error> {
        let client = client_builder.build()?;

        Ok(Self(client))
    }
}

impl StatusListClient for HttpStatusListClient {
    async fn fetch(&self, url: Url) -> Result<StatusListToken, StatusListClientError> {
        let response = self
            .as_ref()
            .get(url)
            .header(header::ACCEPT, STATUS_LIST_JWT_ACCEPT)
            .send()
            .await?;

        let status_list_token = response.error_for_status()?.text().await?.parse()?;

        Ok(status_list_token)
    }
}
