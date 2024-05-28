use futures::TryFutureExt;
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
            .join(&format!("/disclosure/sessions/{session_token}/disclosed_attributes"));
        let mut error_url = disclosed_attributes_url.clone();

        let queries = nonce
            .map(|nonce| {
                let query = serde_urlencoded::to_string(DisclosedAttributesParams { nonce: nonce.into() })?;

                // Create a separate error query, where the nonce is masked with "X" characters.
                let error_query = serde_urlencoded::to_string(DisclosedAttributesParams {
                    nonce: "X".repeat(16).into(),
                })?;

                Ok::<_, serde_urlencoded::ser::Error>((query, error_query))
            })
            .transpose()?;

        if let Some((query, error_query)) = queries {
            disclosed_attributes_url.set_query(query.as_str().into());
            error_url.set_query(error_query.as_str().into());
        }

        let disclosed_attributes = self
            .client
            .get(disclosed_attributes_url)
            .send()
            .and_then(|response| async { response.error_for_status() })
            .and_then(|response| async { response.json::<DisclosedAttributes>().await })
            .map_err(|error| error.with_url(error_url)) // Show the prepared error query instead for all reqwest errors.
            .await?;

        Ok(disclosed_attributes)
    }
}
