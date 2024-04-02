use indexmap::IndexMap;

use nl_wallet_mdoc::{server_state::SessionState, unsigned::UnsignedMdoc, utils::x509::Certificate};
use openid4vc::{
    issuer::{AttributeService, Created},
    token::{AttestationPreview, TokenErrorCode, TokenRequest, TokenRequestGrantType},
    ErrorResponse,
};
use wallet_common::config::wallet_config::BaseUrl;

use crate::pid::brp::{
    client::{BrpClient, BrpError, HttpBrpClient},
    data::BrpDataError,
};

use super::digid::{self, OpenIdClient};

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
    #[error("error retrieving from BRP")]
    Brp(#[from] BrpError),
    #[error("error mapping BRP data to PID data")]
    BrpData(#[from] BrpDataError),
}

pub struct AttributeCertificates {
    certificates: IndexMap<String, Certificate>,
}

impl AttributeCertificates {
    pub fn new(certificates: IndexMap<String, Certificate>) -> Self {
        Self { certificates }
    }

    pub fn try_unsigned_mdoc_to_attestion_preview(&self, unsigned: UnsignedMdoc) -> Result<AttestationPreview, Error> {
        let preview = AttestationPreview::MsoMdoc {
            issuer: self
                .certificates
                .get(&unsigned.doc_type)
                .ok_or(Error::MissingCertificate(unsigned.doc_type.clone()))?
                .clone(),
            unsigned_mdoc: unsigned,
        };
        Ok(preview)
    }
}

pub struct BrpPidAttributeService {
    brp_client: HttpBrpClient,
    openid_client: OpenIdClient,
    certificates: AttributeCertificates,
}

impl BrpPidAttributeService {
    pub fn new(
        brp_client: HttpBrpClient,
        issuer_url: BaseUrl,
        bsn_privkey: String,
        trust_anchors: Vec<reqwest::Certificate>,
        certificates: IndexMap<String, Certificate>,
    ) -> Result<Self, Error> {
        Ok(Self {
            brp_client,
            openid_client: OpenIdClient::new(issuer_url, bsn_privkey, trust_anchors)?,
            certificates: AttributeCertificates::new(certificates),
        })
    }
}

impl AttributeService for BrpPidAttributeService {
    type Error = Error;

    async fn attributes(
        &self,
        _session: &SessionState<Created>,
        token_request: TokenRequest,
    ) -> Result<Vec<AttestationPreview>, Error> {
        let openid_token_request = TokenRequest {
            grant_type: TokenRequestGrantType::AuthorizationCode {
                code: token_request.code().clone(),
            },
            ..token_request
        };

        let bsn = self.openid_client.bsn(openid_token_request).await?;
        let persons = self.brp_client.get_person_by_bsn(bsn).await?;

        persons
            .persons
            .first()
            .map(|person| {
                let unsigned_mdocs: Vec<UnsignedMdoc> = person.try_into()?;
                let previews = unsigned_mdocs
                    .into_iter()
                    .map(|unsigned| self.certificates.try_unsigned_mdoc_to_attestion_preview(unsigned))
                    .collect::<Result<Vec<AttestationPreview>, Error>>()?;
                Ok(previews)
            })
            .unwrap_or(Err(Error::NoAttributesFound))
    }
}
