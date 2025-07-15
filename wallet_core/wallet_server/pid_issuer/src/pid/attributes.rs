use std::num::NonZero;

use futures::future::try_join_all;

use attestation_data::attributes::Attribute;
use attestation_data::attributes::AttributeValue;
use attestation_data::attributes::Attributes;
use attestation_data::attributes::AttributesHandlingError;
use attestation_data::issuable_document::IssuableDocument;
use crypto::x509::CertificateError;
use hsm::service::HsmError;
use http_utils::tls::pinning::TlsPinningConfig;
use http_utils::urls::BaseUrl;
use openid4vc::issuer::AttributeService;
use openid4vc::oidc;
use openid4vc::token::TokenRequest;
use openid4vc::token::TokenRequestGrantType;
use sd_jwt_vc_metadata::ClaimPath;
use server_utils::keys::SecretKeyVariant;
use utils::vec_at_least::VecNonEmpty;

use crate::pid::brp::client::BrpClient;
use crate::pid::brp::client::BrpError;
use crate::pid::brp::client::HttpBrpClient;
use crate::pid::constants::PID_ATTESTATION_TYPE;
use crate::pid::constants::PID_BSN;
use crate::pid::constants::PID_RECOVERY_CODE;

use super::digid;
use super::digid::OpenIdClient;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("DigiD error: {0}")]
    Digid(#[from] digid::Error),
    #[error("could not find attributes for BSN")]
    NoAttributesFound,
    #[error("could not find issuer URI for doctype: {0}")]
    NoIssuerUriFound(String),
    #[error("error retrieving from BRP: {0}")]
    Brp(#[from] BrpError),
    #[error("error creating issuable documents")]
    InvalidIssuableDocuments,
    #[error("certificate error: {0}")]
    Certificate(#[from] CertificateError),
    #[error("unexpected number of SAN DNS names or URIs in issuer certificate; expected: 0, found {0}")]
    UnexpectedIssuerSanDnsNameOrUrisCount(NonZero<usize>),
    #[error("could not find BSN attribute")]
    NoBsnFound,
    #[error("could not find PID attestation")]
    NoPidAttestationFound,
    #[error("error retrieving BSN: {0}")]
    RetrievingBsn(#[source] AttributesHandlingError),
    #[error("BSN attribute had unexpected type (expected string)")]
    BsnUnexpectedType,
    #[error("failed to compute BSN HMAC: {0}")]
    Hmac(#[from] HsmError),
    #[error("error inserting recovery code: {0}")]
    InsertingRecoveryCode(#[source] AttributesHandlingError),
}

pub struct BrpPidAttributeService {
    brp_client: HttpBrpClient,
    openid_client: OpenIdClient,
    recovery_code_secret_key: SecretKeyVariant,
}

impl BrpPidAttributeService {
    pub fn try_new(
        brp_client: HttpBrpClient,
        bsn_privkey: &str,
        http_config: TlsPinningConfig,
        recovery_code_secret_key: SecretKeyVariant,
    ) -> Result<Self, Error> {
        Ok(Self {
            brp_client,
            openid_client: OpenIdClient::try_new(bsn_privkey, http_config)?,
            recovery_code_secret_key,
        })
    }
}

impl AttributeService for BrpPidAttributeService {
    type Error = Error;

    async fn attributes(&self, token_request: TokenRequest) -> Result<VecNonEmpty<IssuableDocument>, Error> {
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

        let attestations = person.into_issuable().into_inner();
        if !attestations
            .iter()
            .any(|(attestation_type, _)| attestation_type == PID_ATTESTATION_TYPE)
        {
            return Err(Error::NoPidAttestationFound);
        }

        let issuable_documents = try_join_all(attestations.into_iter().map(|(attestation_type, attributes)| async {
            let mut attributes = Attributes::from(attributes);

            if attestation_type == PID_ATTESTATION_TYPE {
                Self::insert_recovery_code(&mut attributes, &self.recovery_code_secret_key).await?;
            }

            IssuableDocument::try_new(attestation_type, attributes).map_err(|_| Error::InvalidIssuableDocuments)
        }))
        .await?
        .try_into()
        .unwrap(); // Safe because we iterated over a VecNonEmpty;

        Ok(issuable_documents)
    }

    async fn oauth_metadata(&self, issuer_url: &BaseUrl) -> Result<oidc::Config, Error> {
        let mut metadata = self.openid_client.discover_metadata().await?;
        metadata.token_endpoint = issuer_url.join_base_url("/token").as_ref().clone();
        Ok(metadata)
    }
}

impl BrpPidAttributeService {
    async fn insert_recovery_code(attributes: &mut Attributes, secret_key: &SecretKeyVariant) -> Result<(), Error> {
        let bsn = match attributes
            .get(&vec![ClaimPath::SelectByKey(PID_BSN.to_string())].try_into().unwrap())
            .map_err(Error::RetrievingBsn)?
            .ok_or(Error::NoBsnFound)?
        {
            AttributeValue::Text(str) => str,
            _ => return Err(Error::BsnUnexpectedType),
        };

        let recovery_code = AttributeValue::Text(hex::encode(secret_key.sign_hmac(bsn.as_bytes()).await?));

        attributes
            .insert(
                &vec![ClaimPath::SelectByKey(PID_RECOVERY_CODE.to_string())]
                    .try_into()
                    .unwrap(),
                Attribute::Single(recovery_code),
            )
            .map_err(Error::InsertingRecoveryCode)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use ring::hmac;
    use ring::hmac::HMAC_SHA256;

    use attestation_data::attributes::Attribute;
    use attestation_data::attributes::AttributeValue;
    use attestation_data::attributes::Attributes;
    use server_utils::keys::SecretKeyVariant;
    use server_utils::settings::SecretKey;

    use crate::pid::attributes::BrpPidAttributeService;

    fn attributes(bsn: &str) -> Attributes {
        IndexMap::from_iter([(
            "bsn".to_string(),
            Attribute::Single(AttributeValue::Text(bsn.to_string())),
        )])
        .into()
    }

    #[tokio::test]
    async fn test_recovery_code() {
        let bsn = "123";
        let key: Vec<_> = (0..32).collect();

        let mut attrs = attributes(bsn);

        let recovery_code_secret_key = SecretKeyVariant::from_settings(
            SecretKey::Software {
                secret_key: key.clone().try_into().unwrap(),
            },
            None,
        )
        .unwrap();

        BrpPidAttributeService::insert_recovery_code(&mut attrs, &recovery_code_secret_key)
            .await
            .unwrap();

        let hmac_key = &hmac::Key::new(HMAC_SHA256, &key);
        let expected_hmac = hex::encode(hmac::sign(hmac_key, bsn.as_bytes()));

        // The result should be the attributes we started with, with a recovery_code attribute added to it.
        let expected_attrs = Attributes::from(IndexMap::from_iter(attributes(bsn).into_inner().into_iter().chain([(
            "recovery_code".to_string(),
            Attribute::Single(AttributeValue::Text(expected_hmac)),
        )])));

        assert_eq!(attrs, expected_attrs);
    }
}
