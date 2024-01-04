use reqwest::Client;
use url::Url;

use nl_wallet_mdoc::{
    server_state::SessionToken,
    verifier::{ItemsRequests, SessionType, StatusResponse},
};
use wallet_server::verifier::{ReturnUrlTemplate, StartDisclosureRequest, StartDisclosureResponse};

pub struct WalletServerClient {
    client: Client,
    base_url: Url,
}

impl WalletServerClient {
    pub fn new(base_url: Url) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
        }
    }

    pub async fn start(
        &self,
        usecase: String,
        items_requests: ItemsRequests,
        session_type: SessionType,
        return_url_template: Option<ReturnUrlTemplate>,
    ) -> Result<(Url, Url), anyhow::Error> {
        // TODO check if base_url ends with '/' (possibly already on init)
        let response = self
            .client
            .post(self.base_url.join("/sessions")?)
            .json(&StartDisclosureRequest {
                usecase,
                items_requests,
                session_type,
                return_url_template,
            })
            .send()
            .await?
            .error_for_status()?
            .json::<StartDisclosureResponse>()
            .await?;
        Ok((response.session_url, response.engagement_url))
    }

    pub async fn status(&self, session_id: SessionToken) -> Result<StatusResponse, anyhow::Error> {
        Ok(self
            .client
            .get(self.base_url.join(&format!("/sessions/{session_id}/status"))?)
            .send()
            .await?
            .error_for_status()?
            .json::<StatusResponse>()
            .await?)
    }
}
