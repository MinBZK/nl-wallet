use derive_more::AsRef;
use http::header;
use http_utils::reqwest::bytes_with_max_response_size;
use http_utils::reqwest::parse_content_type;
use reqwest::ClientBuilder;
use reqwest::Response;
use url::Url;

use crate::status_list_token::StatusListFormat;
use crate::status_list_token::StatusListToken;
use crate::verification::client::StatusListClient;
use crate::verification::client::StatusListClientError;

/// The maximum size of a Status List Token response body, to avoid exhausting memory.
pub(crate) const MAX_STATUS_LIST_TOKEN_SIZE: usize = 5 * 1024 * 1024;

#[derive(Debug, Clone, AsRef)]
pub struct HttpStatusListClient(reqwest::Client);

impl HttpStatusListClient {
    pub fn new(client_builder: ClientBuilder) -> Result<Self, reqwest::Error> {
        let client = client_builder.build()?;

        Ok(Self(client))
    }
}

impl StatusListClient for HttpStatusListClient {
    async fn fetch(
        &self,
        url: Url,
        request_format: StatusListFormat,
    ) -> Result<StatusListToken, StatusListClientError> {
        // Accept either format, the requested format is listed first.
        let other_format = match request_format {
            StatusListFormat::Jwt => StatusListFormat::Cwt,
            StatusListFormat::Cwt => StatusListFormat::Jwt,
        };

        let accept = format!("{}, {}", request_format.media_type(), other_format.media_type());

        let mut response = self
            .as_ref()
            .get(url)
            .header(header::ACCEPT, accept)
            .send()
            .await?
            .error_for_status()?;

        // The server may respond in another format than requested, use the one it says it responds in, or fall back to
        // the requested format.
        let format = response_format(&response)?.unwrap_or(request_format);
        // Read the body in chunks rather than via `.bytes()`, so a status list host cannot exhaust memory by returning
        // an unbounded or never-ending response body.
        let body = bytes_with_max_response_size(&mut response, MAX_STATUS_LIST_TOKEN_SIZE)
            .await?
            .ok_or(StatusListClientError::TooLarge)?;
        let status_list_token = StatusListToken::from_bytes_as(format, &body).map_err(StatusListClientError::Parse)?;

        Ok(status_list_token)
    }
}

/// Determine the format of the Status List Token in `response` from its `Content-Type`, if present.
fn response_format(response: &Response) -> Result<Option<StatusListFormat>, StatusListClientError> {
    let Some(content_type) = response.headers().get(header::CONTENT_TYPE) else {
        return Ok(None);
    };

    parse_content_type(response)
        .and_then(|media_type| media_type.essence_str().parse().ok())
        .map(Some)
        .ok_or_else(|| {
            StatusListClientError::UnexpectedContentType(String::from_utf8_lossy(content_type.as_bytes()).into_owned())
        })
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use crypto::server_keys::generate::Ca;
    use http_utils::httpmock::httpmock_reqwest_client_builder;
    use httpmock::Method::GET;
    use httpmock::MockServer;
    use jwt::UnverifiedJwt;
    use rstest::rstest;

    use super::*;
    use crate::status_list_cwt::mock::mock_status_list_cwt;
    use crate::status_list_jwt::mock::mock_status_list_jwt;

    async fn mock_status_list_token_bytes(format: StatusListFormat) -> Vec<u8> {
        let keypair = Ca::generate_mock().generate_issuer_status_list_mock().unwrap();
        match format {
            StatusListFormat::Jwt => {
                let (_, _, jwt) = mock_status_list_jwt(&keypair, None, None).await;
                UnverifiedJwt::from(jwt).into_serialization().into_bytes()
            }
            StatusListFormat::Cwt => {
                let (cwt, _) = mock_status_list_cwt(&keypair, None).await;
                cose::cbor_serialize(&cwt).unwrap()
            }
        }
    }

    /// Start a server that responds with a token in the `served` format and `content_type`
    async fn start_mock_server(format: StatusListFormat, content_type: Option<&str>) -> MockServer {
        let body = mock_status_list_token_bytes(format).await;
        let server = MockServer::start_async().await;
        server
            .mock_async(|when, then| {
                when.method(GET).path("/statuslists/1");

                let then = then.status(200).body(body);
                if let Some(content_type) = content_type {
                    then.header("content-type", content_type);
                }
            })
            .await;

        server
    }

    #[rstest]
    #[case::jwt(StatusListFormat::Jwt, StatusListFormat::Jwt, Some("application/statuslist+jwt"))]
    #[case::cwt(StatusListFormat::Cwt, StatusListFormat::Cwt, Some("application/statuslist+cwt"))]
    #[case::jwt_for_cwt(StatusListFormat::Cwt, StatusListFormat::Jwt, Some("application/statuslist+jwt"))]
    #[case::without_content_type(StatusListFormat::Cwt, StatusListFormat::Cwt, None)]
    #[tokio::test]
    async fn test_fetch_content_type(
        #[case] requested: StatusListFormat,
        #[case] served: StatusListFormat,
        #[case] content_type: Option<&str>,
    ) {
        let server = start_mock_server(served, content_type).await;

        let token = HttpStatusListClient::new(httpmock_reqwest_client_builder())
            .unwrap()
            .fetch(server.url("/statuslists/1").parse().unwrap(), requested)
            .await
            .unwrap();

        let received = match token {
            StatusListToken::Jwt(_) => StatusListFormat::Jwt,
            StatusListToken::Cwt(_) => StatusListFormat::Cwt,
        };

        assert_eq!(received, served);
    }

    #[rstest]
    #[case::jwt(StatusListFormat::Jwt, "application/statuslist+jwt, application/statuslist+cwt")]
    #[case::cwt(StatusListFormat::Cwt, "application/statuslist+cwt, application/statuslist+jwt")]
    #[tokio::test]
    async fn test_fetch_accepts_both_formats(#[case] requested: StatusListFormat, #[case] accept: &str) {
        let body = mock_status_list_token_bytes(requested).await;
        let server = MockServer::start_async().await;
        server
            .mock_async(|when, then| {
                when.method(GET).path("/statuslists/1").header("accept", accept);
                then.status(200).body(body);
            })
            .await;

        HttpStatusListClient::new(httpmock_reqwest_client_builder())
            .unwrap()
            .fetch(server.url("/statuslists/1").parse().unwrap(), requested)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_fetch_rejects_unexpected_content_type() {
        let server = start_mock_server(StatusListFormat::Jwt, Some("application/jwt")).await;

        let error = HttpStatusListClient::new(httpmock_reqwest_client_builder())
            .unwrap()
            .fetch(server.url("/statuslists/1").parse().unwrap(), StatusListFormat::Jwt)
            .await
            .unwrap_err();

        assert_matches!(
            error,
            StatusListClientError::UnexpectedContentType(content_type) if content_type == "application/jwt"
        );
    }

    #[tokio::test]
    async fn test_fetch_rejects_too_large_response() {
        let server = MockServer::start_async().await;
        server
            .mock_async(|when, then| {
                when.method(GET).path("/statuslists/1");
                then.status(200)
                    .header("content-type", "application/statuslist+jwt")
                    .body(vec![b'a'; MAX_STATUS_LIST_TOKEN_SIZE + 1]);
            })
            .await;

        let error = HttpStatusListClient::new(httpmock_reqwest_client_builder())
            .unwrap()
            .fetch(server.url("/statuslists/1").parse().unwrap(), StatusListFormat::Jwt)
            .await
            .unwrap_err();

        assert_matches!(error, StatusListClientError::TooLarge);
    }
}
