use std::sync::Arc;

use attestation_data::attributes::Attribute;
use attestation_data::attributes::AttributeValue;
use attestation_data::attributes::Attributes;
use attestation_data::attributes::AttributesHandlingError;
use attestation_types::claim_path::ClaimPath;
use crypto::x509::CertificateError;
use hsm::service::HsmError;
use jwk_simple::Key;
use openid4vc::Format;
use openid4vc::issuable_document::IssuableDocument;
use openid4vc::issuer::AttributeService;
use openid4vc::token::TokenRequest;
use openid4vc::token::TokenRequestGrantType;
use server_utils::keys::SecretKeyVariant;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_nonempty;

use super::digid;
use super::digid::DigidMetadataCache;
use super::digid::OpenIdClient;
use crate::pid::brp::client::BrpClient;
use crate::pid::brp::client::BrpError;
use crate::pid::brp::client::HttpBrpClient;
use crate::pid::constants::PID_ATTESTATION_TYPE;
use crate::pid::constants::PID_BSN;
use crate::pid::constants::PID_RECOVERY_CODE;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("DigiD error: {0}")]
    Digid(#[source] digid::Error),

    #[error("could not find attributes for BSN")]
    NoAttributesFound,

    #[error("error retrieving from BRP: {0}")]
    Brp(#[source] BrpError),

    #[error("error creating issuable documents")]
    InvalidIssuableDocuments,

    #[error("certificate error: {0}")]
    Certificate(#[source] CertificateError),

    #[error("could not find BSN attribute")]
    NoBsnFound,

    #[error("error retrieving BSN: {0}")]
    RetrievingBsn(#[source] AttributesHandlingError),

    #[error("BSN attribute had unexpected type (expected string)")]
    BsnUnexpectedType,

    #[error("failed to compute BSN HMAC: {0}")]
    Hmac(#[source] HsmError),

    #[error("error inserting recovery code: {0}")]
    InsertingRecoveryCode(#[source] AttributesHandlingError),

    #[error("invalid grant type")]
    InvalidGrantType,
}

pub struct BrpPidAttributeService {
    brp_client: HttpBrpClient,
    openid_client: OpenIdClient,
    recovery_code_secret_key: SecretKeyVariant,
}

impl BrpPidAttributeService {
    pub fn try_new(
        brp_client: HttpBrpClient,
        bsn_privkey: &Key,
        client_id: impl Into<String>,
        digid_metadata_cache: Arc<DigidMetadataCache>,
        recovery_code_secret_key: SecretKeyVariant,
    ) -> Result<Self, Error> {
        Ok(Self {
            brp_client,
            openid_client: OpenIdClient::try_new(bsn_privkey, client_id, digid_metadata_cache).map_err(Error::Digid)?,
            recovery_code_secret_key,
        })
    }
}

impl AttributeService for BrpPidAttributeService {
    type Error = Error;

    async fn attributes(&self, token_request: TokenRequest) -> Result<VecNonEmpty<IssuableDocument>, Error> {
        let authorization_code = match token_request.grant_type {
            TokenRequestGrantType::AuthorizationCode { code } => code,
            _ => return Err(Error::InvalidGrantType),
        };

        // `code_verifier` carries the upstream code verifier here: the issuer's `/token` handler
        // verified the wallet's PKCE and substituted the upstream value into this field.
        let bsn = self
            .openid_client
            .bsn(
                authorization_code,
                token_request.code_verifier,
                token_request.redirect_uri,
            )
            .await
            .map_err(Error::Digid)?;
        let mut persons = self.brp_client.get_person_by_bsn(&bsn).await.map_err(Error::Brp)?;

        if persons.persons.len() != 1 {
            return Err(Error::NoAttributesFound);
        }

        let person = persons.persons.remove(0);

        let attributes = Self::insert_recovery_code(person.into_attributes(), &self.recovery_code_secret_key).await?;

        // Supply both a SD-JWT and Mdoc PID credential, based on the same set of attributes.
        let issuable_documents = vec_nonempty![
            IssuableDocument::try_new_with_random_id(
                Format::SdJwt,
                PID_ATTESTATION_TYPE.to_string(),
                attributes.clone()
            )
            .map_err(|_| Error::InvalidIssuableDocuments)?,
            IssuableDocument::try_new_with_random_id(Format::MsoMdoc, PID_ATTESTATION_TYPE.to_string(), attributes)
                .map_err(|_| Error::InvalidIssuableDocuments)?,
        ];

        Ok(issuable_documents)
    }
}

impl BrpPidAttributeService {
    async fn insert_recovery_code(
        mut attributes: Attributes,
        secret_key: &SecretKeyVariant,
    ) -> Result<Attributes, Error> {
        let bsn = match attributes
            .get(&vec_nonempty![ClaimPath::SelectByKey(PID_BSN.to_string())])
            .map_err(Error::RetrievingBsn)?
            .ok_or(Error::NoBsnFound)?
        {
            AttributeValue::Text(str) => str,
            _ => return Err(Error::BsnUnexpectedType),
        };

        let recovery_code = AttributeValue::Text(hex::encode(
            secret_key.sign_hmac(bsn.as_bytes()).await.map_err(Error::Hmac)?,
        ));

        attributes
            .insert(
                &vec_nonempty![ClaimPath::SelectByKey(PID_RECOVERY_CODE.to_string())],
                Attribute::Single(recovery_code),
            )
            .map_err(Error::InsertingRecoveryCode)?;

        Ok(attributes)
    }
}

#[cfg(test)]
mod tests {
    use attestation_data::attributes::Attribute;
    use attestation_data::attributes::AttributeValue;
    use attestation_data::attributes::Attributes;
    use indexmap::IndexMap;
    use ring::hmac;
    use ring::hmac::HMAC_SHA256;
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

        let attrs = attributes(bsn);

        let recovery_code_secret_key = SecretKeyVariant::from_settings(
            SecretKey::Software {
                secret_key: key.clone().try_into().unwrap(),
            },
            None,
        )
        .unwrap();

        let attrs = BrpPidAttributeService::insert_recovery_code(attrs, &recovery_code_secret_key)
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
