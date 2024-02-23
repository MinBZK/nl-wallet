use nl_wallet_mdoc::{basic_sa_ext::UnsignedMdoc, server_state::SessionState};
use openid4vc::{
    issuer::{AttributeService, Created},
    token::{TokenErrorType, TokenRequest, TokenRequestGrantType},
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
    attrs_lookup: MockAttributesLookup,
}

impl MockPidAttributeService {
    pub async fn new(settings: &Issuer) -> Result<Self, Error> {
        Ok(MockPidAttributeService {
            openid_client: OpenIdClient::new(
                settings.digid.issuer_url.clone(),
                settings.digid.bsn_privkey.clone(),
                settings.digid.client_id.clone(),
            )?,
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
                code: token_request.code().clone(),
            },
            ..token_request
        };

        let access_token = self.openid_client.request_token(openid_token_request).await?;
        let bsn = self.openid_client.bsn(&access_token).await?;
        let unsigned_mdocs = self.attrs_lookup.attributes(&bsn).ok_or(Error::NoAttributesFound)?;

        Ok(unsigned_mdocs)
    }
}
