use std::error::Error;
use std::hash::Hash;
use std::io::ErrorKind;
use std::marker::PhantomData;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

use http::HeaderValue;
use http::StatusCode;
use http::header;
use http_utils::reqwest::IntoReqwestClient;
use http_utils::reqwest::ReqwestClientUrl;
use tokio::fs;
use tracing::warn;

use super::FileStorageError;
use super::Filename;
use super::HttpClientError;
use super::HttpResponse;
use crate::reqwest::CachedReqwestClient;

pub struct EtagHttpClient<T, B, E> {
    cached_client: CachedReqwestClient<B>,
    resource_identifier: Filename,
    etag_dir: PathBuf,
    _marker: PhantomData<(T, B, E)>, // data and error type to fetch and builder type
}

impl<T, B, E> EtagHttpClient<T, B, E> {
    pub fn new(resource_identifier: Filename, etag_dir: PathBuf) -> Self {
        Self {
            cached_client: CachedReqwestClient::new(),
            resource_identifier,
            etag_dir,
            _marker: PhantomData,
        }
    }

    fn etag_file(resource_identifier: Filename, etag_dir: &Path) -> PathBuf {
        let mut filename = resource_identifier.into_inner();
        filename.set_extension("etag");
        etag_dir.join(filename)
    }

    async fn read_latest_etag(resource_identifier: Filename, etag_dir: &Path) -> Option<HeaderValue> {
        let etag_file = Self::etag_file(resource_identifier, etag_dir);

        let content = match fs::read(&etag_file).await {
            Ok(content) => content,
            Err(err) if err.kind() == ErrorKind::NotFound => {
                return None;
            }
            Err(err) => {
                warn!("Could not read ETag header file from {}: {err}", etag_file.display());
                return None;
            }
        };
        match HeaderValue::from_bytes(&content) {
            Ok(etag) => Some(etag),
            Err(err) => {
                warn!("Invalid stored ETag header value, deleting stored ETag file: {err}");
                if let Err(err) = fs::remove_file(&etag_file).await {
                    warn!("Could not remove ETag file {}: {err}", etag_file.display());
                }
                None
            }
        }
    }

    async fn store_latest_etag(&self, etag: &HeaderValue) -> Result<(), FileStorageError> {
        let etag_file = Self::etag_file(self.resource_identifier.clone(), &self.etag_dir);
        fs::write(&etag_file, etag.as_bytes()).await?;
        Ok(())
    }
}

impl<T, B, E> EtagHttpClient<T, B, E>
where
    T: FromStr,
    T::Err: Error + Send + Sync + 'static,
    B: IntoReqwestClient + Clone + Hash,
    E: From<HttpClientError>,
{
    /// Fetches the resource, sending along the `ETag` of the last accepted response. If the resource has been
    /// modified, the response is parsed and passed to `accept`. Only if that succeeds, the `ETag` of the response is
    /// stored.
    pub async fn fetch_and_accept<R>(
        &self,
        client_builder: &B,
        accept: impl FnOnce(T) -> Result<R, E>,
    ) -> Result<HttpResponse<R>, E> {
        let client = self
            .cached_client
            .get_or_try_init(client_builder, IntoReqwestClient::try_into_client)
            .map_err(HttpClientError::Networking)?;

        let latest_etag = Self::read_latest_etag(self.resource_identifier.clone(), &self.etag_dir).await;

        let response = client
            .send_custom_get(
                ReqwestClientUrl::Relative(&self.resource_identifier.as_ref().to_string_lossy()),
                |request| match latest_etag {
                    Some(etag) => request.header(header::IF_NONE_MATCH, etag),
                    None => request,
                },
            )
            .await
            .map_err(HttpClientError::Networking)?;

        // Try to get the body from any 4xx or 5xx error responses, in order to create an Error::Response.
        let response = match response.error_for_status_ref() {
            Ok(_) => Ok(response),
            Err(error) => {
                let error = match response.text().await.ok() {
                    Some(body) => HttpClientError::Response(error, body),
                    None => HttpClientError::Networking(error),
                };

                Err(error)
            }
        }?;

        if let StatusCode::NOT_MODIFIED = response.status() {
            return Ok(HttpResponse::NotModified);
        }

        let etag = response.headers().get(header::ETAG).cloned();

        let body = response.text().await.map_err(|_| HttpClientError::EmptyBody)?;
        let parsed = body.parse().map_err(|e: T::Err| HttpClientError::Parse(e.into()))?;
        let accepted = accept(parsed)?;

        if let Some(etag) = etag {
            self.store_latest_etag(&etag).await.map_err(HttpClientError::EtagFile)?;
        }

        Ok(HttpResponse::Parsed(accepted))
    }
}

#[cfg(test)]
mod test {
    use std::assert_matches;
    use std::convert::Infallible;
    use std::str::FromStr;

    use http::header;
    use http_utils::client::TlsPinningConfig;
    use httpmock::Method::GET;
    use httpmock::MockServer;
    use tokio::fs;

    use super::EtagHttpClient;
    use crate::repository::Filename;
    use crate::repository::HttpClientError;
    use crate::repository::HttpResponse;

    struct Stub;

    impl FromStr for Stub {
        type Err = Infallible;

        fn from_str(_: &str) -> Result<Self, Self::Err> {
            Ok(Self {})
        }
    }

    #[tokio::test]
    async fn test_etag_http_client() {
        let tempdir = tempfile::tempdir().unwrap();
        let server = MockServer::start_async().await;
        let base_url = server.base_url().parse().unwrap();

        let mock_ok = server
            .mock_async(|when, then| {
                when.method(GET)
                    .header_missing(header::IF_NONE_MATCH.as_str())
                    .path("/config");

                then.status(200).header(header::ETAG.as_str(), "etag");
            })
            .await;

        let mock_not_modified = server
            .mock_async(|when, then| {
                when.method(GET)
                    .header(header::IF_NONE_MATCH.as_str(), "etag")
                    .path("/config");

                then.status(304);
            })
            .await;

        let client: EtagHttpClient<Stub, TlsPinningConfig, HttpClientError> =
            EtagHttpClient::new("config".parse().unwrap(), tempdir.path().to_path_buf());

        let client_builder = TlsPinningConfig::try_new_httpmock(base_url).unwrap();

        let response = client.fetch_and_accept(&client_builder, Ok).await.unwrap();
        assert!(matches!(response, HttpResponse::Parsed(_)));
        mock_ok.assert_async().await;

        let response = client.fetch_and_accept(&client_builder, Ok).await.unwrap();
        assert!(matches!(response, HttpResponse::NotModified));
        mock_not_modified.assert_async().await;
    }

    #[tokio::test]
    async fn test_etag_http_client_reads_latest_stored_etag() {
        let server = MockServer::start_async().await;
        let base_url = server.base_url().parse().unwrap();

        let mock_ok = server
            .mock_async(|when, then| {
                when.method(GET)
                    .header_missing(header::IF_NONE_MATCH.as_str())
                    .path("/config");

                then.status(200).header(header::ETAG.as_str(), "etag");
            })
            .await;

        let mock_not_modified = server
            .mock_async(|when, then| {
                when.method(GET)
                    .header(header::IF_NONE_MATCH.as_str(), "etag")
                    .path("/config");

                then.status(304);
            })
            .await;

        let etag_dir = tempfile::tempdir().unwrap();
        let client_builder = TlsPinningConfig::try_new_httpmock(base_url).unwrap();
        let client1: EtagHttpClient<Stub, TlsPinningConfig, HttpClientError> =
            EtagHttpClient::new("config".parse().unwrap(), etag_dir.path().to_path_buf());
        let client2: EtagHttpClient<Stub, TlsPinningConfig, HttpClientError> =
            EtagHttpClient::new("config".parse().unwrap(), etag_dir.path().to_path_buf());

        let response = client1.fetch_and_accept(&client_builder, Ok).await.unwrap();
        assert!(matches!(response, HttpResponse::Parsed(_)));
        mock_ok.assert_async().await;

        // The second client was created before the ETag was stored, but should still send it along.
        let response = client2.fetch_and_accept(&client_builder, Ok).await.unwrap();
        assert!(matches!(response, HttpResponse::NotModified));
        mock_not_modified.assert_async().await;
    }

    #[tokio::test]
    async fn test_etag_http_client_invalid_stored_etag() {
        let server = MockServer::start_async().await;
        let base_url = server.base_url().parse().unwrap();

        // Responding without an ETag ensures that the ETag file is not written again.
        let mock_ok = server
            .mock_async(|when, then| {
                when.method(GET)
                    .header_missing(header::IF_NONE_MATCH.as_str())
                    .path("/config");

                then.status(200);
            })
            .await;

        let tempdir = tempfile::tempdir().unwrap();
        let resource_identifier: Filename = "config".parse().unwrap();
        let etag_file = EtagHttpClient::<Stub, TlsPinningConfig, HttpClientError>::etag_file(
            resource_identifier.clone(),
            tempdir.path(),
        );
        fs::write(&etag_file, b"invalid\netag").await.unwrap();

        let client_builder = TlsPinningConfig::try_new_httpmock(base_url).unwrap();
        let client = EtagHttpClient::<Stub, TlsPinningConfig, HttpClientError>::new(
            resource_identifier,
            tempdir.path().to_path_buf(),
        );

        // The invalid ETag should not be sent along and should be deleted.
        let response = client.fetch_and_accept(&client_builder, Ok).await.unwrap();
        assert!(matches!(response, HttpResponse::Parsed(_)));
        mock_ok.assert_async().await;

        assert!(!fs::try_exists(&etag_file).await.unwrap());
    }

    #[tokio::test]
    async fn test_etag_http_client_mismatch() {
        let tempdir = tempfile::tempdir().unwrap();
        let server = MockServer::start_async().await;
        let base_url = server.base_url().parse().unwrap();

        let mock_ok1 = server
            .mock_async(|when, then| {
                when.method(GET)
                    .header_missing(header::IF_NONE_MATCH.as_str())
                    .path("/config");

                then.status(200).header(header::ETAG.as_str(), "etag");
            })
            .await;

        let mock_ok2 = server
            .mock_async(|when, then| {
                when.method(GET)
                    .header(header::IF_NONE_MATCH.as_str(), "etag")
                    .path("/config");

                then.status(200).header(header::ETAG.as_str(), "other etag");
            })
            .await;

        let client_builder = TlsPinningConfig::try_new_httpmock(base_url).unwrap();
        let client: EtagHttpClient<Stub, TlsPinningConfig, HttpClientError> =
            EtagHttpClient::new("config".parse().unwrap(), tempdir.as_ref().to_path_buf());

        let response = client.fetch_and_accept(&client_builder, Ok).await.unwrap();
        assert!(matches!(response, HttpResponse::Parsed(_)));
        mock_ok1.assert_async().await;

        let response = client.fetch_and_accept(&client_builder, Ok).await.unwrap();
        assert!(matches!(response, HttpResponse::Parsed(_)));
        mock_ok2.assert_async().await;
    }

    #[tokio::test]
    async fn test_etag_http_client_rejected() {
        let server = MockServer::start_async().await;
        let base_url = server.base_url().parse().unwrap();

        let mock_ok = server
            .mock_async(|when, then| {
                when.method(GET)
                    .header_missing(header::IF_NONE_MATCH.as_str())
                    .path("/config");

                then.status(200).header(header::ETAG.as_str(), "etag");
            })
            .await;

        let etag_dir = tempfile::tempdir().unwrap();
        let client_builder = TlsPinningConfig::try_new_httpmock(base_url).unwrap();
        let client: EtagHttpClient<Stub, TlsPinningConfig, HttpClientError> =
            EtagHttpClient::new("config".parse().unwrap(), etag_dir.path().to_path_buf());

        // Reject the response, which should not result in its ETag being stored.
        let result = client
            .fetch_and_accept(&client_builder, |_| Err::<Stub, _>(HttpClientError::EmptyBody))
            .await;
        let Err(error) = result else {
            panic!("rejecting the response should result in an error");
        };
        assert_matches!(error, HttpClientError::EmptyBody);
        assert!(
            fs::read_dir(etag_dir.path())
                .await
                .unwrap()
                .next_entry()
                .await
                .unwrap()
                .is_none()
        );

        // The next request should therefore not include the ETag, so the response is received and evaluated again.
        let response = client.fetch_and_accept(&client_builder, Ok).await.unwrap();
        assert!(matches!(response, HttpResponse::Parsed(_)));
        mock_ok.assert_calls_async(2).await;

        // Now that the response has been accepted, its ETag should have been stored.
        let etag_file = EtagHttpClient::<Stub, TlsPinningConfig, HttpClientError>::etag_file(
            "config".parse().unwrap(),
            etag_dir.path(),
        );
        assert_eq!(fs::read(&etag_file).await.unwrap(), b"etag");
    }
}
