use async_trait::async_trait;
use futures::TryFutureExt;
use mime::APPLICATION_WWW_FORM_URLENCODED;
use nl_wallet_mdoc::{basic_sa_ext::UnsignedMdoc, server_state::SessionState};
use openid4vc::ErrorResponse;
use reqwest::{header::CONTENT_TYPE, Client};

use openid4vc::token::{TokenErrorType, TokenRequest, TokenRequestGrantType, TokenResponse};

use openid4vc::issuer::{AttributeService, Created};

use crate::settings::Issuer;

use super::{
    digid::{self, OpenIdClient},
    mock::MockAttributesLookup,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    TransportError(#[from] reqwest::Error),
    #[error("error requesting token: {0:?}")]
    TokenRequest(ErrorResponse<TokenErrorType>),
    #[error(transparent)]
    Digid(#[from] digid::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    UrlEncoding(#[from] serde_urlencoded::ser::Error),
}

/// Given a BSN, determine the attributes to be issued. Contract for the BRP query.
pub trait AttributesLookup {
    fn attributes(&self, bsn: &str) -> Vec<UnsignedMdoc>;
}

pub struct MockPidAttributeService {
    openid_client: OpenIdClient,
    http_client: reqwest::Client,
    attrs_lookup: MockAttributesLookup,
}

#[async_trait]
impl AttributeService for MockPidAttributeService {
    type Error = Error;
    type Settings = Issuer;

    async fn new(settings: &Self::Settings) -> Result<Self, Error> {
        Ok(MockPidAttributeService {
            openid_client: OpenIdClient::new(&settings.digid).await?,
            http_client: reqwest_client(),
            attrs_lookup: MockAttributesLookup::from(settings.mock_data.clone().unwrap_or_default()),
        })
    }

    async fn attributes(
        &self,
        _session: &SessionState<Created>,
        token_request: TokenRequest,
    ) -> Result<Vec<UnsignedMdoc>, Error> {
        let openid_token_request = serde_urlencoded::to_string(TokenRequest {
            grant_type: TokenRequestGrantType::AuthorizationCode {
                code: token_request.code(),
            },
            ..token_request
        })?;

        let openid_token_response: TokenResponse = self
            .http_client
            .post(self.openid_client.openid_client.config().token_endpoint.clone())
            .header(CONTENT_TYPE, APPLICATION_WWW_FORM_URLENCODED.as_ref())
            .body(dbg!(openid_token_request))
            .send()
            .map_err(Error::from)
            .and_then(|response| async {
                // If the HTTP response code is 4xx or 5xx, parse the JSON as an error
                let status = response.status();
                if status.is_client_error() || status.is_server_error() {
                    let error = response.json::<ErrorResponse<TokenErrorType>>().await?;
                    Err(Error::TokenRequest(error))
                } else {
                    let text = response.json().await?;
                    Ok(text)
                }
            })
            .await?;

        let bsn = self.openid_client.bsn(&openid_token_response.access_token).await?;
        let unsigned_mdocs = self.attrs_lookup.attributes(&bsn);

        Ok(unsigned_mdocs)
    }
}

pub fn reqwest_client() -> Client {
    let client_builder = Client::builder();
    #[cfg(feature = "disable_tls_validation")]
    let client_builder = client_builder.danger_accept_invalid_certs(true);
    client_builder.build().unwrap()
}
