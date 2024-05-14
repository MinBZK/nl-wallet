use reqwest::Client;
use url::Url;

use nl_wallet_mdoc::{
    server_state::SessionToken,
    verifier::{DisclosedAttributes, ItemsRequests, ReturnUrlTemplate},
};
use wallet_common::config::wallet_config::BaseUrl;
use wallet_server::verifier::{DisclosedAttributesParams, StartDisclosureRequest, StartDisclosureResponse};

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
        return_url_template: Option<ReturnUrlTemplate>,
    ) -> Result<(Url, Url), anyhow::Error> {
        let response = self
            .client
            .post(self.base_url.join("/disclosure/sessions"))
            .json(&StartDisclosureRequest {
                usecase,
                items_requests,
                return_url_template,
            })
            .send()
            .await?
            .error_for_status()?
            .json::<StartDisclosureResponse>()
            .await?;
        Ok((response.status_url, response.disclosed_attributes_url))
    }

    pub async fn disclosed_attributes(
        &self,
        session_token: SessionToken,
        nonce: Option<String>,
    ) -> Result<DisclosedAttributes, anyhow::Error> {
        let mut disclosed_attributes_url = self
            .base_url
            .clone()
            .join(&format!("/disclosure/sessions/{session_token}/disclosed_attributes"));

        let query = nonce
            .map(|nonce| {
                serde_urlencoded::to_string(DisclosedAttributesParams {
                    return_url_nonce: nonce.into(),
                })
            })
            .transpose()?;
        disclosed_attributes_url.set_query(query.as_deref());

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
