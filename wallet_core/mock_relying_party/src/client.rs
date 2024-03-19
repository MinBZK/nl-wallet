use reqwest::Client;
use url::Url;

use nl_wallet_mdoc::{
    server_state::SessionToken,
    verifier::{DisclosedAttributes, ItemsRequests, SessionType, StatusResponse},
};
use wallet_common::config::wallet_config::BaseUrl;
use wallet_server::verifier::{ReturnUrlTemplate, StartDisclosureRequest, StartDisclosureResponse};

pub struct WalletServerClient {
    client: Client,
    base_url: BaseUrl,
}

impl WalletServerClient {
    pub fn new(base_url: BaseUrl) -> Self {
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
    ) -> Result<(Url, Url, Url), anyhow::Error> {
        let response = self
            .client
            .post(self.base_url.join("/disclosure/sessions"))
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
        Ok((
            response.session_url,
            response.engagement_url,
            response.disclosed_attributes_url,
        ))
    }

    pub async fn status(&self, session_id: SessionToken) -> Result<StatusResponse, anyhow::Error> {
        Ok(self
            .client
            .get(self.base_url.join(&format!("/disclosure/{session_id}/status")))
            .send()
            .await?
            .error_for_status()?
            .json::<StatusResponse>()
            .await?)
    }

    pub async fn disclosed_attributes(
        &self,
        session_id: SessionToken,
        transcript_hash: Option<String>,
    ) -> Result<DisclosedAttributes, anyhow::Error> {
        let mut disclosed_attributes_url = self
            .base_url
            .clone()
            .join(&format!("/disclosure/sessions/{session_id}/disclosed_attributes"));
        if let Some(hash) = transcript_hash {
            disclosed_attributes_url.set_query(Some(&format!("transcript_hash={}", hash)));
        }

        Ok(self
            .client
            .get(disclosed_attributes_url)
            .send()
            .await?
            .error_for_status()?
            .json::<DisclosedAttributes>()
            .await?)
    }
}
