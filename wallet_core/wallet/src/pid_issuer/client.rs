use std::time::Duration;

use async_trait::async_trait;
use futures::future::TryFutureExt;
use serde::Deserialize;
use url::Url;

use super::{PidIssuerClient, PidIssuerError};

const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);

pub struct RemotePidIssuerClient {
    http_client: reqwest::Client,
}

#[derive(Deserialize)]
struct BsnResponse {
    bsn: String,
}

impl RemotePidIssuerClient {
    fn new() -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(CLIENT_TIMEOUT)
            .build()
            .expect("Could not build reqwest HTTP client");

        RemotePidIssuerClient { http_client }
    }
}

impl Default for RemotePidIssuerClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PidIssuerClient for RemotePidIssuerClient {
    async fn extract_bsn(&self, base_url: &Url, access_token: &str) -> Result<String, PidIssuerError> {
        let url = base_url
            .join("extract_bsn")
            .expect("Could not create \"extract_bsn\" URL from PID issuer base URL");

        let bsn_response = self
            .http_client
            .post(url)
            .bearer_auth(access_token)
            .send()
            .map_err(PidIssuerError::from)
            .and_then(|response| async {
                // Try to get the body from any 4xx or 5xx error responses,
                // in order to create an Error::PidIssuerResponse.
                // TODO: Implement proper JSON-based error reporting
                //       for the mock PID issuer.
                match response.error_for_status_ref() {
                    Ok(_) => Ok(response),
                    Err(error) => {
                        let error = match response.text().await.ok() {
                            Some(body) => PidIssuerError::PidIssuerResponse(error, body),
                            None => PidIssuerError::PidIssuer(error),
                        };

                        Err(error)
                    }
                }
            })
            .await?
            .json::<BsnResponse>()
            .await?;

        Ok(bsn_response.bsn)
    }
}
