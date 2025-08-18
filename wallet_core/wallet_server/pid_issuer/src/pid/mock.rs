use std::convert::Infallible;

use derive_more::Constructor;
use indexmap::IndexMap;

use attestation_data::attributes::Attribute;
use attestation_data::attributes::AttributeValue;
use attestation_data::constants::*;
use attestation_data::issuable_document::IssuableDocument;
use http_utils::urls::BaseUrl;
use openid4vc::issuer::AttributeService;
use openid4vc::oidc;
use openid4vc::token::TokenRequest;
use utils::vec_at_least::VecNonEmpty;

#[derive(Debug, Constructor)]
pub struct MockAttributeService(VecNonEmpty<IssuableDocument>);

pub fn mock_issuable_document_pid() -> IssuableDocument {
    IssuableDocument::try_new(
        PID_ATTESTATION_TYPE.to_string(),
        IndexMap::from_iter(vec![
            (
                PID_FAMILY_NAME.to_string(),
                Attribute::Single(AttributeValue::Text("De Bruijn".to_string())),
            ),
            (
                PID_GIVEN_NAME.to_string(),
                Attribute::Single(AttributeValue::Text("Willeke Liselotte".to_string())),
            ),
            (
                PID_BIRTH_DATE.to_string(),
                Attribute::Single(AttributeValue::Text("1997-05-10".to_string())),
            ),
            (
                PID_AGE_OVER_18.to_string(),
                Attribute::Single(AttributeValue::Bool(true)),
            ),
            (
                PID_BSN.to_string(),
                Attribute::Single(AttributeValue::Text("999991772".to_string())),
            ),
            (
                PID_RECOVERY_CODE.to_string(),
                Attribute::Single(AttributeValue::Text("123".to_string())),
            ),
        ])
        .into(),
    )
    .unwrap()
}

pub fn mock_issuable_document_address() -> IssuableDocument {
    IssuableDocument::try_new(
        ADDRESS_ATTESTATION_TYPE.to_string(),
        IndexMap::from_iter(vec![(
            PID_ADDRESS_GROUP.to_string(),
            Attribute::Nested(IndexMap::from_iter(vec![
                (
                    PID_RESIDENT_STREET.to_string(),
                    Attribute::Single(AttributeValue::Text("Turfmarkt".to_string())),
                ),
                (
                    PID_RESIDENT_HOUSE_NUMBER.to_string(),
                    Attribute::Single(AttributeValue::Text("147".to_string())),
                ),
                (
                    PID_RESIDENT_POSTAL_CODE.to_string(),
                    Attribute::Single(AttributeValue::Text("2511 DP".to_string())),
                ),
                (
                    PID_RESIDENT_CITY.to_string(),
                    Attribute::Single(AttributeValue::Text("Den Haag".to_string())),
                ),
                (
                    PID_RESIDENT_COUNTRY.to_string(),
                    Attribute::Single(AttributeValue::Text("Nederland".to_string())),
                ),
            ])),
        )])
        .into(),
    )
    .unwrap()
}

impl Default for MockAttributeService {
    fn default() -> Self {
        Self::new(
            vec![mock_issuable_document_pid(), mock_issuable_document_address()]
                .try_into()
                .unwrap(),
        )
    }
}

impl AttributeService for MockAttributeService {
    type Error = Infallible;

    async fn attributes(&self, _token_request: TokenRequest) -> Result<VecNonEmpty<IssuableDocument>, Self::Error> {
        let Self(documents) = self;

        Ok(documents.clone())
    }

    async fn oauth_metadata(&self, issuer_url: &BaseUrl) -> Result<oidc::Config, Self::Error> {
        Ok(oidc::Config::new_mock(issuer_url))
    }
}
