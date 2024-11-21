use std::path::Path;
use std::path::PathBuf;

use http::header;
use http::HeaderValue;
use http::StatusCode;
use parking_lot::Mutex;
use tokio::fs;

use wallet_common::config::wallet_config::WalletConfiguration;
use wallet_common::jwt::validations;
use wallet_common::jwt::EcdsaDecodingKey;
use wallet_common::jwt::Jwt;
use wallet_common::reqwest::RequestBuilder;

use crate::config::ConfigurationError;

use super::FileStorageError;

pub struct HttpConfigurationClient<C> {
    http_config: C,
    signing_public_key: EcdsaDecodingKey,
    storage_path: PathBuf,
    latest_etag: Mutex<Option<HeaderValue>>,
}

const ETAG_FILENAME: &str = "latest-configuration-etag.txt";

impl<C> HttpConfigurationClient<C>
where
    C: RequestBuilder,
{
    pub async fn new(
        http_config: C,
        signing_public_key: EcdsaDecodingKey,
        storage_path: PathBuf,
    ) -> Result<Self, ConfigurationError> {
        let initial_etag = Self::read_latest_etag(storage_path.as_path()).await?;

        let client = Self {
            http_config,
            signing_public_key,
            storage_path,
            latest_etag: Mutex::new(initial_etag),
        };

        Ok(client)
    }

    async fn read_latest_etag(storage_path: &Path) -> Result<Option<HeaderValue>, FileStorageError> {
        let path = Self::path_for_etag_file(storage_path);

        if !fs::try_exists(&path).await? {
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
        let (http_client, mut request_builder) = self.http_config.get(Path::new("wallet-config"));

        if let Some(etag) = self.latest_etag.lock().as_ref() {
            request_builder = request_builder.header(header::IF_NONE_MATCH, etag);
        }

        let request = request_builder.build()?;
        let response = http_client.execute(request).await?;

        // Try to get the body from any 4xx or 5xx error responses,
        // in order to create an ConfigurationError::Response.
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
            *self.latest_etag.lock() = Some(etag.to_owned());
        }

        let body = response.text().await?;
        let wallet_config = Jwt::from(body).parse_and_verify(&self.signing_public_key, &validations())?;

        Ok(Some(wallet_config))
    }
}
