use indexmap::IndexMap;

use nl_wallet_mdoc::unsigned::UnsignedMdoc;
use nl_wallet_mdoc::utils::x509::BorrowingCertificate;
use openid4vc::issuer::AttributeService;
use openid4vc::issuer::Created;
use openid4vc::oidc;
use openid4vc::server_state::SessionState;
use openid4vc::token::CredentialPreview;
use openid4vc::token::TokenRequest;
use openid4vc::token::TokenRequestGrantType;
use openid4vc::ErrorResponse;
use openid4vc::TokenErrorCode;
use sd_jwt::metadata::TypeMetadata;
use sd_jwt::metadata::TypeMetadataChain;
use sd_jwt::metadata::TypeMetadataError;
use wallet_common::config::http::TlsPinningConfig;
use wallet_common::urls::BaseUrl;
use wallet_common::vec_at_least::VecNonEmpty;

use crate::pid::brp::client::BrpClient;
use crate::pid::brp::client::BrpError;
use crate::pid::brp::client::HttpBrpClient;

use super::digid;
use super::digid::OpenIdClient;

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
    #[error("could not find type metadata for doctype {0}")]
    NoMetadataFound(String),
    #[error("missing certificate for issuance of doctype {0}")]
    MissingCertificate(String),
    #[error("error retrieving from BRP: {0}")]
    Brp(#[from] BrpError),
    #[error("error signing metadata: {0}")]
    MetadataSigning(#[from] TypeMetadataError),
}

pub struct AttributeCertificates {
    certificates: IndexMap<String, BorrowingCertificate>,
}

impl AttributeCertificates {
    pub fn new(certificates: IndexMap<String, BorrowingCertificate>) -> Self {
        Self { certificates }
    }

    pub fn try_unsigned_mdoc_to_attestion_preview(
        &self,
        unsigned_mdoc: UnsignedMdoc,
        metadata_chain: TypeMetadataChain,
    ) -> Result<CredentialPreview, Error> {
        let preview = CredentialPreview::MsoMdoc {
            issuer: self
                .certificates
                .get(&unsigned_mdoc.doc_type)
                .ok_or(Error::MissingCertificate(unsigned_mdoc.doc_type.clone()))?
                .clone(),
            unsigned_mdoc,
            metadata_chain,
        };
        Ok(preview)
    }
}

pub struct BrpPidAttributeService {
    brp_client: HttpBrpClient,
    openid_client: OpenIdClient<TlsPinningConfig>,
    certificates: AttributeCertificates,
    metadata_by_doctype: IndexMap<String, TypeMetadata>,
}

impl BrpPidAttributeService {
    pub fn new(
        brp_client: HttpBrpClient,
        bsn_privkey: &str,
        http_config: TlsPinningConfig,
        certificates: IndexMap<String, BorrowingCertificate>,
        metadata_by_doctype: IndexMap<String, TypeMetadata>,
    ) -> Result<Self, Error> {
        Ok(Self {
            brp_client,
            openid_client: OpenIdClient::new(bsn_privkey, http_config)?,
            certificates: AttributeCertificates::new(certificates),
            metadata_by_doctype,
        })
    }
}

impl AttributeService for BrpPidAttributeService {
    type Error = Error;

    async fn attributes(
        &self,
        _session: &SessionState<Created>,
        token_request: TokenRequest,
    ) -> Result<VecNonEmpty<CredentialPreview>, Error> {
        let openid_token_request = TokenRequest {
            grant_type: TokenRequestGrantType::AuthorizationCode {
                code: token_request.code().clone(),
            },
            ..token_request
        };

        let bsn = self.openid_client.bsn(openid_token_request).await?;
        let mut persons = self.brp_client.get_person_by_bsn(&bsn).await?;

        if persons.persons.len() != 1 {
            return Err(Error::NoAttributesFound);
        }

        let person = persons.persons.remove(0);
        let unsigned_mdocs: Vec<UnsignedMdoc> = person.into();
        let previews = unsigned_mdocs
            .into_iter()
            .map(|unsigned| {
                let metadata = self
                    .metadata_by_doctype
                    .get(unsigned.doc_type.as_str())
                    .ok_or(Error::NoMetadataFound(String::from(unsigned.doc_type.as_str())))?;
                let metadata_chain = TypeMetadataChain::create(metadata.clone(), vec![])?;
                self.certificates
                    .try_unsigned_mdoc_to_attestion_preview(unsigned, metadata_chain)
            })
            .collect::<Result<Vec<CredentialPreview>, Error>>()?;
        previews.try_into().map_err(|_| Error::NoAttributesFound)
    }

    async fn oauth_metadata(&self, issuer_url: &BaseUrl) -> Result<oidc::Config, Error> {
        let mut metadata = self.openid_client.discover_metadata().await?;
        metadata.token_endpoint = issuer_url.join_base_url("/token").as_ref().clone();
        Ok(metadata)
    }
}
