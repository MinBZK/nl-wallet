use futures::TryFutureExt;
use reqwest::Client;

use nl_wallet_mdoc::{basic_sa_ext::UnsignedMdoc, server_state::SessionState};
use openid4vc::{
    issuer::{AttributeService, Created},
    token::{TokenErrorType, TokenRequest, TokenRequestGrantType, TokenResponse},
    ErrorResponse,
};

use crate::settings::Issuer;

use super::{
    digid::{self, OpenIdClient},
    mock::MockAttributesLookup,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("networking error: {0}")]
    TransportError(#[from] reqwest::Error),
    #[error("error requesting token: {0:?}")]
    TokenRequest(ErrorResponse<TokenErrorType>),
    #[error("DigiD error: {0}")]
    Digid(#[from] digid::Error),
    #[error("JSON error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("URL encoding error: {0}")]
    UrlEncoding(#[from] serde_urlencoded::ser::Error),
    #[error("could not find attributes for BSN")]
    NoAttributesFound,
}

pub struct MockPidAttributeService {
    openid_client: OpenIdClient,
    http_client: reqwest::Client,
    attrs_lookup: MockAttributesLookup,
}

impl MockPidAttributeService {
    pub async fn new(settings: &Issuer) -> Result<Self, Error> {
        Ok(MockPidAttributeService {
            openid_client: OpenIdClient::new(
                settings.digid.issuer_url.clone(),
                settings.digid.bsn_privkey.clone(),
                settings.digid.client_id.clone(),
            )
            .await?,
            http_client: reqwest_client(),
            attrs_lookup: MockAttributesLookup::from(settings.mock_data.clone().unwrap_or_default()),
        })
    }
}

impl AttributeService for MockPidAttributeService {
    type Error = Error;

    async fn attributes(
        &self,
        _session: &SessionState<Created>,
        token_request: TokenRequest,
    ) -> Result<Vec<UnsignedMdoc>, Error> {
        let openid_token_request = TokenRequest {
            grant_type: TokenRequestGrantType::AuthorizationCode {
                code: token_request.code().to_string(),
            },
            ..token_request
        };

        let openid_token_response: TokenResponse = self
            .http_client
            .post(self.openid_client.openid_client.config().token_endpoint.clone())
            .form(&openid_token_request)
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
        let unsigned_mdocs = self.attrs_lookup.attributes(&bsn).ok_or(Error::NoAttributesFound)?;

        Ok(unsigned_mdocs)
    }
}

pub fn reqwest_client() -> Client {
    let client_builder = Client::builder();
    #[cfg(feature = "disable_tls_validation")]
    let client_builder = client_builder.danger_accept_invalid_certs(true);
    client_builder.build().unwrap()
}
