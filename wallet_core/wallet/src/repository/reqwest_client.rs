use std::error::Error;
use std::hash::Hash;
use std::marker::PhantomData;
use std::str::FromStr;

use http_utils::reqwest::IntoPinnedReqwestClient;
use http_utils::reqwest::ReqwestClientUrl;

use crate::reqwest::CachedReqwestClient;

use super::Filename;
use super::HttpClient;
use super::HttpClientError;
use super::HttpResponse;

pub struct ReqwestHttpClient<T, B> {
    cached_client: CachedReqwestClient<B>,
    resource_identifier: Filename,
    _marker: PhantomData<(T, B)>, // data type to fetch and builder type
}

impl<T, B> ReqwestHttpClient<T, B> {
    pub fn new(resource_identifier: Filename) -> Self {
        Self {
            cached_client: CachedReqwestClient::new(),
            resource_identifier,
            _marker: PhantomData,
        }
    }
}

impl<T, B> HttpClient<T, B> for ReqwestHttpClient<T, B>
where
    B: IntoPinnedReqwestClient + Clone + Hash + Send + Sync,
    T: FromStr + Send + Sync,
    T::Err: Error + Send + Sync + 'static,
{
    type Error = HttpClientError;

    async fn fetch(&self, client_builder: &B) -> Result<HttpResponse<T>, Self::Error> {
        let client = self
            .cached_client
            .get_or_try_init(client_builder, IntoPinnedReqwestClient::try_into_client)?;
        let response = client
            .send_get(ReqwestClientUrl::Relative(
                &self.resource_identifier.as_ref().to_string_lossy(),
            ))
            .await?;

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

        match response.text().await.ok() {
            Some(b) => {
                let parsed = HttpResponse::Parsed(b.parse().map_err(|e: T::Err| HttpClientError::Parse(e.into()))?);
                Ok(parsed)
            }
            _ => Err(HttpClientError::EmptyBody)?,
        }
    }
}
