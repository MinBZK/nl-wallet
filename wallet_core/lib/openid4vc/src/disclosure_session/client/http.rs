use futures::TryFutureExt;
use reqwest::header::ACCEPT;
use reqwest::ClientBuilder;
use reqwest::Method;
use reqwest::Response;
use serde::de::DeserializeOwned;

use http_utils::reqwest::client_builder_accept_json;
use http_utils::urls::BaseUrl;
use jwt::Jwt;

use crate::errors::DisclosureErrorResponse;
use crate::errors::ErrorResponse;
use crate::errors::GetRequestErrorCode;
use crate::errors::PostAuthResponseErrorCode;
use crate::errors::VpAuthorizationErrorCode;
use crate::openid4vp::VpAuthorizationRequest;
use crate::openid4vp::VpResponse;
use crate::openid4vp::WalletRequest;
use crate::verifier::VpToken;

use super::VpMessageClient;
use super::VpMessageClientError;
use super::APPLICATION_OAUTH_AUTHZ_REQ_JWT;

#[derive(Debug, Clone)]
pub struct HttpVpMessageClient {
    http_client: reqwest::Client,
}

impl HttpVpMessageClient {
    pub fn new(client_builder: ClientBuilder) -> Result<Self, reqwest::Error> {
        let http_client = client_builder_accept_json(client_builder).build()?;

        let message_client = Self { http_client };

        Ok(message_client)
    }

    async fn get_body_from_response<T>(response: Response) -> Result<String, VpMessageClientError>
    where
        T: DeserializeOwned,
        DisclosureErrorResponse<T>: Into<VpMessageClientError>,
    {
        // If the HTTP response code is 4xx or 5xx, parse the JSON as an error
        let status = response.status();
        if status.is_client_error() || status.is_server_error() {
            let error = response.json::<DisclosureErrorResponse<T>>().await?;

            return Err(error.into());
        }
        let body = response.text().await?;

        Ok(body)
    }

    /// If the RP does not wish to specify a redirect URI, e.g. in case of cross device flows, then the spec does not
    /// say whether the RP should send an empty JSON object, i.e. `{}`, or no body at all. So this function accepts
    /// both.
    async fn handle_vp_response<T>(response: Response) -> Result<Option<BaseUrl>, VpMessageClientError>
    where
        T: DeserializeOwned,
        DisclosureErrorResponse<T>: Into<VpMessageClientError>,
    {
        let response_body = Self::get_body_from_response(response).await?;

        if response_body.is_empty() {
            return Ok(None);
        }
        let response: VpResponse = serde_json::from_str(&response_body)?;

        Ok(response.redirect_uri)
    }
}

impl VpMessageClient for HttpVpMessageClient {
    async fn get_authorization_request(
        &self,
        url: BaseUrl,
        wallet_nonce: Option<String>,
    ) -> Result<Jwt<VpAuthorizationRequest>, VpMessageClientError> {
        let method = match wallet_nonce {
            Some(_) => Method::POST,
            None => Method::GET,
        };

        let mut request = self
            .http_client
            .request(method, url.into_inner())
            .header(ACCEPT, APPLICATION_OAUTH_AUTHZ_REQ_JWT.as_ref());

        if wallet_nonce.is_some() {
            request = request.form(&WalletRequest { wallet_nonce });
        }

        request
            .send()
            .map_err(VpMessageClientError::from)
            .and_then(|response| async {
                let jwt = Self::get_body_from_response::<GetRequestErrorCode>(response)
                    .await?
                    .into();

                Ok(jwt)
            })
            .await
    }

    async fn send_authorization_response(
        &self,
        url: BaseUrl,
        jwe: String,
    ) -> Result<Option<BaseUrl>, VpMessageClientError> {
        self.http_client
            .post(url.into_inner())
            .form(&VpToken { vp_token: jwe })
            .send()
            .map_err(VpMessageClientError::from)
            .and_then(|response| async {
                let redirect_uri = Self::handle_vp_response::<PostAuthResponseErrorCode>(response).await?;

                Ok(redirect_uri)
            })
            .await
    }

    async fn send_error(
        &self,
        url: BaseUrl,
        error: ErrorResponse<VpAuthorizationErrorCode>,
    ) -> Result<Option<BaseUrl>, VpMessageClientError> {
        self.http_client
            .post(url.into_inner())
            .form(&error)
            .send()
            .map_err(VpMessageClientError::from)
            .and_then(|response| async {
                let redirect_uri = Self::handle_vp_response::<PostAuthResponseErrorCode>(response).await?;

                Ok(redirect_uri)
            })
            .await
    }
}
