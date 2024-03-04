use nl_wallet_mdoc::{basic_sa_ext::UnsignedMdoc, server_state::SessionState};
use openid4vc::{
    issuer::{AttributeService, Created},
    token::{TokenErrorCode, TokenRequest, TokenRequestGrantType},
    ErrorResponse,
};
use url::Url;

use crate::settings::MockAttributes;

use super::{
    digid::{self, OpenIdClient},
    mock::MockAttributesLookup,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("networking error: {0}")]
    TransportError(#[from] reqwest::Error),
    #[error("error requesting token: {0:?}")]
    TokenRequest(ErrorResponse<TokenErrorCode>),
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
    pub fn new(issuer_url: Url, bsn_privkey: String, mock_data: Option<Vec<MockAttributes>>) -> Result<Self, Error> {
        Ok(MockPidAttributeService {
            openid_client: OpenIdClient::new(issuer_url, bsn_privkey)?,
            attrs_lookup: MockAttributesLookup::from(mock_data.unwrap_or_default()),
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

        let bsn = self.openid_client.bsn(openid_token_request).await?;
        let unsigned_mdocs = self.attrs_lookup.attributes(&bsn).ok_or(Error::NoAttributesFound)?;

        Ok(unsigned_mdocs)
    }
}
