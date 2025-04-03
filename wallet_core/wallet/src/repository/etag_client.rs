use std::error::Error;
use std::marker::PhantomData;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

use http::header;
use http::HeaderValue;
use http::StatusCode;
use parking_lot::Mutex;
use tokio::fs;

use http_utils::reqwest::RequestBuilder;

use super::FileStorageError;
use super::Filename;
use super::HttpClient;
use super::HttpClientError;
use super::HttpResponse;

pub struct EtagHttpClient<T, B, E> {
    resource_identifier: Filename,
    etag_dir: PathBuf,
    latest_etag: Mutex<Option<HeaderValue>>,
    _marker: PhantomData<(T, B, E)>, // data and error type to fetch and builder type
}

impl<T, B, E> EtagHttpClient<T, B, E>
where
    E: From<FileStorageError> + Error,
{
    pub async fn new(resource_identifier: Filename, etag_dir: PathBuf) -> Result<Self, E> {
        let initial_etag = Self::read_latest_etag(resource_identifier.clone(), &etag_dir).await?;

        let client = Self {
            resource_identifier,
            etag_dir,
            latest_etag: Mutex::new(initial_etag),
            _marker: PhantomData,
        };

        Ok(client)
    }
}

impl<T, B, E> EtagHttpClient<T, B, E> {
    fn etag_file(resource_identifier: Filename, etag_dir: &Path) -> PathBuf {
        let mut filename = resource_identifier.into_inner();
        filename.set_extension("etag");
        etag_dir.join(filename)
    }

    async fn read_latest_etag(
        resource_identifier: Filename,
        etag_dir: &Path,
    ) -> Result<Option<HeaderValue>, FileStorageError> {
        let etag_file = Self::etag_file(resource_identifier, etag_dir);
        if !fs::try_exists(&etag_file).await? {
            return Ok(None);
        }

        let content = fs::read(&etag_file).await?;
        Ok(Some(HeaderValue::from_bytes(&content).unwrap()))
    }

    async fn store_latest_etag(&self, etag: &HeaderValue) -> Result<(), FileStorageError> {
        let etag_file = Self::etag_file(self.resource_identifier.clone(), &self.etag_dir);
        fs::write(&etag_file, etag.as_bytes()).await?;
        Ok(())
    }
}

impl<T, B, E> HttpClient<T, B> for EtagHttpClient<T, B, E>
where
    T: FromStr + Send + Sync,
    T::Err: Error + Send + Sync + 'static,
    B: RequestBuilder + Send + Sync,
    E: From<HttpClientError> + Error + Send + Sync + 'static,
{
    type Error = E;

    async fn fetch(&self, client_builder: &B) -> Result<HttpResponse<T>, Self::Error> {
        let (client, mut request_builder) = client_builder.get(self.resource_identifier.as_ref());

        if let Some(etag) = self.latest_etag.lock().as_ref() {
            request_builder = request_builder.header(header::IF_NONE_MATCH, etag);
        }

        let request = request_builder.build().map_err(HttpClientError::Networking)?;
        let response = client.execute(request).await.map_err(HttpClientError::Networking)?;

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

        if let Some(etag) = response.headers().get(header::ETAG) {
            self.store_latest_etag(etag).await.map_err(HttpClientError::EtagFile)?;
            *self.latest_etag.lock() = Some(etag.to_owned());
        }

        match response.text().await.ok() {
            Some(b) => {
                let parsed = HttpResponse::Parsed(b.parse().map_err(|e: T::Err| HttpClientError::Parse(e.into()))?);
                Ok(parsed)
            }
            _ => Err(HttpClientError::EmptyBody)?,
        }
    }
}

#[cfg(test)]
mod test {
    use std::convert::Infallible;
    use std::str::FromStr;

    use http::header;
    use wiremock::matchers::header;
    use wiremock::matchers::header_exists;
    use wiremock::matchers::method;
    use wiremock::matchers::path;
    use wiremock::Mock;
    use wiremock::MockServer;
    use wiremock::ResponseTemplate;

    use http_utils::http::test::HttpConfig;

    use crate::repository::HttpClient;
    use crate::repository::HttpClientError;
    use crate::repository::HttpResponse;

    use super::EtagHttpClient;

    struct Stub;

    impl FromStr for Stub {
        type Err = Infallible;

        fn from_str(_: &str) -> Result<Self, Self::Err> {
            Ok(Self {})
        }
    }

    #[tokio::test]
    async fn test_etag_http_client() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/config"))
            .respond_with(ResponseTemplate::new(200).append_header(header::ETAG, "etag"))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/config"))
            .and(header(header::IF_NONE_MATCH, "etag"))
            .respond_with(ResponseTemplate::new(304))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client: EtagHttpClient<Stub, HttpConfig, HttpClientError> =
            EtagHttpClient::new("config".parse().unwrap(), tempfile::tempdir().unwrap().into_path())
                .await
                .unwrap();

        let client_builder = HttpConfig {
            base_url: mock_server.uri().parse().unwrap(),
        };

        let response = client.fetch(&client_builder).await.unwrap();
        assert!(matches!(response, HttpResponse::Parsed(_)));

        let response = client.fetch(&client_builder).await.unwrap();
        assert!(matches!(response, HttpResponse::NotModified));
    }

    #[tokio::test]
    async fn test_etag_http_client_mismatch() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/config"))
            .respond_with(ResponseTemplate::new(200).append_header(header::ETAG, "etag"))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/config"))
            .and(header_exists(header::IF_NONE_MATCH))
            .respond_with(ResponseTemplate::new(200).append_header(header::ETAG, "other etag"))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client_builder = HttpConfig {
            base_url: mock_server.uri().parse().unwrap(),
        };
        let client: EtagHttpClient<Stub, HttpConfig, HttpClientError> =
            EtagHttpClient::new("config".parse().unwrap(), tempfile::tempdir().unwrap().into_path())
                .await
                .unwrap();

        let response = client.fetch(&client_builder).await.unwrap();
        assert!(matches!(response, HttpResponse::Parsed(_)));

        let response = client.fetch(&client_builder).await.unwrap();
        assert!(matches!(response, HttpResponse::Parsed(_)));
    }
}
