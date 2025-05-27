use futures::TryFutureExt;
use reqwest::Client;
use reqwest::Response;

use http_utils::error::HttpJsonErrorBody;
use http_utils::urls::BaseUrl;
use mdoc::verifier::DisclosedAttributes;
use mdoc::verifier::ItemsRequests;
use openid4vc::return_url::ReturnUrlTemplate;
use openid4vc::server_state::SessionToken;
use openid4vc_server::verifier::DisclosedAttributesParams;
use openid4vc_server::verifier::StartDisclosureRequest;
use openid4vc_server::verifier::StartDisclosureResponse;

pub struct WalletServerClient {
    client: Client,
    base_url: BaseUrl,
}

impl WalletServerClient {
    async fn error_for_response(response: Response) -> Result<Response, anyhow::Error> {
        let status = response.status();

        if status.is_client_error() || status.is_server_error() {
            // Try to decode the body as `HttpJsonErrorBody` in order to generate an error message.
            let message = if let Some(body) = response
                .bytes()
                .await
                .ok()
                .and_then(|bytes| serde_json::from_slice::<HttpJsonErrorBody<String>>(&bytes).ok())
            {
                let detail = body
                    .detail
                    .as_deref()
                    .map(|detail| format!("({}) {}", body.r#type, detail))
                    .unwrap_or(body.r#type);
                format!(
                    "verification_server responded with error {}: {}",
                    status.as_u16(),
                    detail
                )
            } else {
                format!("verification_server responded with error {}", status.as_u16())
            };

            return Err(anyhow::Error::msg(message));
        }

        Ok(response)
    }

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
    ) -> Result<SessionToken, anyhow::Error> {
        let response = self
            .client
            .post(self.base_url.join("/disclosure/sessions"))
            .json(&StartDisclosureRequest {
                usecase,
                items_requests: Some(items_requests),
                return_url_template,
            })
            .send()
            .map_err(anyhow::Error::from)
            .and_then(|response| async { Self::error_for_response(response).await })
            .await?
            .json::<StartDisclosureResponse>()
            .await?;
        Ok(response.session_token)
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
            .map_err(anyhow::Error::from)
            .and_then(|response| async { Self::error_for_response(response).await })
            .and_then(|response| async {
                response
                    .json::<DisclosedAttributes>()
                    .map_err(anyhow::Error::from)
                    .await
            })
            .map_err(|error| match error.downcast::<reqwest::Error>() {
                // Show the prepared error query instead for all reqwest errors.
                Ok(req_error) => req_error.with_url(error_url).into(),
                Err(error) => error,
            })
            .await?;

        Ok(disclosed_attributes)
    }
}
