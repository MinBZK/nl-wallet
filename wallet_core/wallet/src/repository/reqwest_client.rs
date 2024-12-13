use std::error::Error;
use std::marker::PhantomData;
use std::str::FromStr;

use wallet_common::reqwest::RequestBuilder;
use wallet_common::urls::Filename;

use super::HttpClient;
use super::HttpClientError;
use super::HttpResponse;

pub struct ReqwestHttpClient<T, B> {
    resource_identifier: Filename,
    _marker: PhantomData<(T, B)>, // data type to fetch and builder type
}

impl<T, B> ReqwestHttpClient<T, B> {
    pub fn new(resource_identifier: Filename) -> Self {
        Self {
            resource_identifier,
            _marker: PhantomData,
        }
    }
}

impl<T, B> HttpClient<T, B> for ReqwestHttpClient<T, B>
where
    B: RequestBuilder + Send + Sync,
    T: FromStr + Send + Sync,
    T::Err: Error + Send + Sync + 'static,
{
    type Error = HttpClientError;

    async fn fetch(&self, client_builder: &B) -> Result<HttpResponse<T>, Self::Error> {
        let (client, request_builder) = client_builder.get(self.resource_identifier.as_ref());
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

        match response.text().await.ok() {
            Some(b) => {
                let parsed = HttpResponse::Parsed(b.parse().map_err(|e: T::Err| HttpClientError::Parse(e.into()))?);
                Ok(parsed)
            }
            _ => Err(HttpClientError::EmptyBody)?,
        }
    }
}
