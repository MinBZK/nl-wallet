use std::time::Duration;

use async_trait::async_trait;
use futures::future::TryFutureExt;
use once_cell::sync::Lazy;
use serde::Deserialize;
use url::Url;

use super::{PidIssuerClient, PidIssuerError};

const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);

/// The base url of the PID issuer.
// NOTE: MUST end with a slash
// TODO: read from configuration
// The android emulator uses 10.0.2.2 as special IP address to connect to localhost of the host OS.
static PID_ISSUER_BASE_URL: Lazy<Url> =
    Lazy::new(|| Url::parse("http://10.0.2.2:3003/").expect("Could not parse PID issuer base URL"));

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
    async fn extract_bsn(&self, access_token: &str) -> Result<String, PidIssuerError> {
        let url = PID_ISSUER_BASE_URL
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
