use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

use http::{header, HeaderMap, HeaderValue, StatusCode};
use tokio::fs;
use url::Url;

use wallet_common::config::wallet_config::WalletConfiguration;

use crate::{config::ConfigurationError, utils::reqwest::default_reqwest_client_builder};

use super::FileStorageError;

pub struct HttpConfigurationClient {
    http_client: reqwest::Client,
    base_url: Url,
    storage_path: PathBuf,
    latest_etag: Mutex<Option<HeaderValue>>,
}

const ETAG_FILENAME: &str = "latest-configuration-etag.txt";

impl HttpConfigurationClient {
    pub async fn new(base_url: Url, storage_path: PathBuf) -> Result<Self, ConfigurationError> {
        let initial_etag = Self::read_latest_etag(storage_path.as_path()).await?;

        let client = Self {
            http_client: default_reqwest_client_builder()
                .default_headers(HeaderMap::from_iter([(
                    header::ACCEPT,
                    HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
                )]))
                .build()
                .expect("Could not build reqwest HTTP client"),
            base_url,
            storage_path,
            latest_etag: Mutex::new(initial_etag),
        };

        Ok(client)
    }

    async fn read_latest_etag(storage_path: &Path) -> Result<Option<HeaderValue>, FileStorageError> {
        let path = Self::path_for_etag_file(storage_path);

        if !path.try_exists()? {
            return Ok(None);
        }

        let content = fs::read(path).await?;
        Ok(Some(HeaderValue::from_bytes(&content).unwrap()))
    }

    async fn store_latest_etag(storage_path: &Path, etag: &HeaderValue) -> Result<(), FileStorageError> {
        let path = Self::path_for_etag_file(storage_path);

        fs::write(path, etag.as_bytes()).await?;

        Ok(())
    }

    fn path_for_etag_file(storage_path: &Path) -> PathBuf {
        storage_path.join(ETAG_FILENAME)
    }

    pub async fn get_wallet_config(&self) -> Result<Option<WalletConfiguration>, ConfigurationError> {
        let url = self.base_url.join("wallet-config")?;
        let mut request_builder = self.http_client.get(url);

        if let Some(etag) = self.latest_etag.lock().unwrap().as_ref() {
            request_builder = request_builder.header(header::IF_NONE_MATCH, etag)
        }

        let request = request_builder.build()?;
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

        if let StatusCode::NOT_MODIFIED = response.status() {
            return Ok(None);
        }

        if let Some(etag) = response.headers().get(header::ETAG) {
            Self::store_latest_etag(self.storage_path.as_path(), etag).await?;
            *self.latest_etag.lock().unwrap() = Some(etag.to_owned());
        }

        let body = response.json().await?;
        Ok(body)
    }
}
