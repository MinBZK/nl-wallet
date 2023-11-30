use http::{header, HeaderMap, HeaderValue};
use url::Url;

use wallet_common::config::wallet_config::WalletConfiguration;

use crate::{config::ConfigurationError, utils::reqwest::default_reqwest_client_builder};

pub struct HttpConfigurationClient {
    http_client: reqwest::Client,
    base_url: Url,
}

impl HttpConfigurationClient {
    pub fn new(base_url: Url) -> Self {
        Self {
            http_client: default_reqwest_client_builder()
                .default_headers(HeaderMap::from_iter([(
                    header::ACCEPT,
                    HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
                )]))
                .build()
                .expect("Could not build reqwest HTTP client"),
            base_url,
        }
    }

    pub async fn get_wallet_config(&self) -> Result<WalletConfiguration, ConfigurationError> {
        let url = self.base_url.join("wallet-config")?;
        let request = self.http_client.get(url).build()?;
        let response = self.http_client.execute(request).await?;

        // Try to get the body from any 4xx or 5xx error responses,
        // in order to create an Error::PidIssuerResponse.
        // TODO: Implement proper JSON-based error reporting?
        let response = match response.error_for_status_ref() {
            Ok(_) => Ok(response),
            Err(error) => {
                let error = match response.text().await.ok() {
                    Some(body) => ConfigurationError::Response(error, body),
                    None => ConfigurationError::Networking(error),
                };

                Err(error)
            }
        }?;

        let body = response.json().await?;
        Ok(body)
    }
}
