use std::num::NonZeroU8;
use std::ops::Add;

use chrono::Days;
use chrono::Utc;
use indexmap::IndexMap;

use nl_wallet_mdoc::unsigned::UnsignedAttributesError;
use openid4vc::attributes::IssuableDocuments;
use openid4vc::issuer::AttributeService;
use openid4vc::issuer::IssuableCredential;
use openid4vc::oidc;
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
    #[error("error parsing unsigned attributes: {0}")]
    UnsignedAttributes(#[from] UnsignedAttributesError),
    #[error("error signing metadata: {0}")]
    MetadataSigning(#[from] TypeMetadataError),
}

pub struct BrpPidAttributeService {
    brp_client: HttpBrpClient,
    openid_client: OpenIdClient<TlsPinningConfig>,
    metadata_by_doctype: IndexMap<String, TypeMetadata>,
    valid_days: Days,
    copy_count: NonZeroU8,
}

impl BrpPidAttributeService {
    pub fn new(
        brp_client: HttpBrpClient,
        bsn_privkey: &str,
        http_config: TlsPinningConfig,
        metadata_by_doctype: IndexMap<String, TypeMetadata>,
        valid_days: Days,
        copy_count: NonZeroU8,
    ) -> Result<Self, Error> {
        Ok(Self {
            brp_client,
            openid_client: OpenIdClient::new(bsn_privkey, http_config)?,
            metadata_by_doctype,
            valid_days,
            copy_count,
        })
    }
}

impl AttributeService for BrpPidAttributeService {
    type Error = Error;

    async fn attributes(&self, token_request: TokenRequest) -> Result<VecNonEmpty<IssuableCredential>, Error> {
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
        let issuable_attributes: IssuableDocuments = person.into();

        issuable_attributes
            .into_inner()
            .into_iter()
            .map(|document| {
                let metadata = self
                    .metadata_by_doctype
                    .get(document.attestation_type())
                    .ok_or(Error::NoMetadataFound(document.attestation_type().to_string()))?;
                let metadata_chain = TypeMetadataChain::create(metadata.clone(), vec![])?;

                Ok(IssuableCredential {
                    document,
                    metadata_chain,
                    valid_from: Utc::now(),
                    valid_until: Utc::now().add(self.valid_days),
                    copy_count: self.copy_count,
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|vec| vec.try_into().unwrap()) // safe because issuable_attributes is non-empty
    }

    async fn oauth_metadata(&self, issuer_url: &BaseUrl) -> Result<oidc::Config, Error> {
        let mut metadata = self.openid_client.discover_metadata().await?;
        metadata.token_endpoint = issuer_url.join_base_url("/token").as_ref().clone();
        Ok(metadata)
    }
}
