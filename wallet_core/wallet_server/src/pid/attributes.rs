use indexmap::IndexMap;
use url::Url;

use nl_wallet_mdoc::{server_state::SessionState, utils::x509::Certificate};
use openid4vc::{
    issuer::{AttributeService, Created},
    token::{AttestationPreview, TokenErrorCode, TokenRequest, TokenRequestGrantType},
    ErrorResponse,
};
use wallet_common::nonempty::NonEmpty;

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
    #[error("missing certificate for issuance of doctype {0}")]
    MissingCertificate(String),
}

pub struct MockPidAttributeService {
    openid_client: OpenIdClient,
    attrs_lookup: MockAttributesLookup,
    certificates: IndexMap<String, Certificate>,
}

impl MockPidAttributeService {
    pub fn new(
        issuer_url: Url,
        bsn_privkey: String,
        trust_anchors: Vec<reqwest::Certificate>,
        mock_data: Option<Vec<MockAttributes>>,
        certificates: IndexMap<String, Certificate>,
    ) -> Result<Self, Error> {
        Ok(MockPidAttributeService {
            openid_client: OpenIdClient::new(issuer_url, bsn_privkey, trust_anchors)?,
            attrs_lookup: MockAttributesLookup::from(mock_data.unwrap_or_default()),
            certificates,
        })
    }
}

impl AttributeService for MockPidAttributeService {
    type Error = Error;

    async fn attributes(
        &self,
        _session: &SessionState<Created>,
        token_request: TokenRequest,
    ) -> Result<NonEmpty<Vec<AttestationPreview>>, Error> {
        let openid_token_request = TokenRequest {
            grant_type: TokenRequestGrantType::AuthorizationCode {
                code: token_request.code().clone(),
            },
            ..token_request
        };

        let bsn = self.openid_client.bsn(openid_token_request).await?;

        self.attrs_lookup
            .attributes(&bsn)
            .ok_or(Error::NoAttributesFound)?
            .into_iter()
            .map(|unsigned| {
                let preview = AttestationPreview::MsoMdoc {
                    issuer: self
                        .certificates
                        .get(&unsigned.doc_type)
                        .ok_or(Error::MissingCertificate(unsigned.doc_type.clone()))?
                        .clone(),
                    unsigned_mdoc: unsigned,
                };
                Ok(preview)
            })
            .collect::<Result<Vec<_>, Error>>()
            .and_then(|r| NonEmpty::try_from(r).map_err(|_| Error::NoAttributesFound))
    }
}
