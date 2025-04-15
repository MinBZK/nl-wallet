use std::convert::Infallible;

use derive_more::Constructor;
use indexmap::IndexMap;

use http_utils::urls::BaseUrl;
use openid4vc::attributes::Attribute;
use openid4vc::attributes::AttributeValue;
use openid4vc::issuable_document::IssuableDocument;
use openid4vc::issuer::AttributeService;
use openid4vc::oidc;
use openid4vc::token::TokenRequest;
use utils::vec_at_least::VecNonEmpty;

use crate::pid::constants::*;

#[derive(Debug, Constructor)]
pub struct MockAttributeService(VecNonEmpty<IssuableDocument>);

impl Default for MockAttributeService {
    fn default() -> Self {
        let pid = IssuableDocument::try_new(
            MOCK_PID_DOCTYPE.to_string(),
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
            ]),
        )
        .unwrap();

        let address = IssuableDocument::try_new(
            MOCK_ADDRESS_DOCTYPE.to_string(),
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
            )]),
        )
        .unwrap();

        Self::new(vec![pid, address].try_into().unwrap())
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
