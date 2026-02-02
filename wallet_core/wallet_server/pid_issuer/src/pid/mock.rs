use std::convert::Infallible;

use derive_more::Constructor;

use attestation_data::attributes::AttributeValue;
use attestation_data::attributes::Attributes;
use attestation_data::issuable_document::IssuableDocument;
use http_utils::urls::BaseUrl;
use openid4vc::issuer::AttributeService;
use openid4vc::oidc;
use openid4vc::token::TokenRequest;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;

use crate::pid::constants::PID_ADDRESS_GROUP;
use crate::pid::constants::PID_AGE_OVER_18;
use crate::pid::constants::PID_ATTESTATION_TYPE;
use crate::pid::constants::PID_BIRTH_DATE;
use crate::pid::constants::PID_BSN;
use crate::pid::constants::PID_FAMILY_NAME;
use crate::pid::constants::PID_GIVEN_NAME;
use crate::pid::constants::PID_RECOVERY_CODE;
use crate::pid::constants::PID_RESIDENT_CITY;
use crate::pid::constants::PID_RESIDENT_COUNTRY;
use crate::pid::constants::PID_RESIDENT_HOUSE_NUMBER;
use crate::pid::constants::PID_RESIDENT_POSTAL_CODE;
use crate::pid::constants::PID_RESIDENT_STREET;

#[derive(Debug, Constructor)]
pub struct MockAttributeService(VecNonEmpty<IssuableDocument>);

pub fn mock_issuable_document_pid() -> IssuableDocument {
    IssuableDocument::try_new_with_random_id(PID_ATTESTATION_TYPE.to_string(), eudi_nl_pid_example()).unwrap()
}

impl Default for MockAttributeService {
    fn default() -> Self {
        Self::new(vec![mock_issuable_document_pid()].try_into().unwrap())
    }
}

impl AttributeService for MockAttributeService {
    type Error = Infallible;

    async fn attributes(&self, _token_request: TokenRequest) -> Result<VecNonEmpty<IssuableDocument>, Self::Error> {
        let Self(documents) = self;

        // Create a copy of the document having a new random id, ensuring unique batch_ids
        let documents = documents
            .nonempty_iter()
            .map(|document| {
                let (attestation_id, attributes, _) = document.clone().into();
                IssuableDocument::try_new_with_random_id(attestation_id, attributes).unwrap()
            })
            .collect();

        Ok(documents)
    }

    async fn oauth_metadata(&self, issuer_url: &BaseUrl) -> Result<oidc::Config, Self::Error> {
        Ok(oidc::Config::new_mock(issuer_url))
    }
}

/// Represents a single card with both PID and address claims
pub fn eudi_nl_pid_example() -> Attributes {
    Attributes::example([
        (
            vec![PID_GIVEN_NAME],
            AttributeValue::Text("Willeke Liselotte".to_string()),
        ),
        (vec![PID_FAMILY_NAME], AttributeValue::Text("De Bruijn".to_string())),
        (vec![PID_BIRTH_DATE], AttributeValue::Text("1997-05-10".to_string())),
        (vec![PID_AGE_OVER_18], AttributeValue::Bool(true)),
        (vec![PID_BSN], AttributeValue::Text("999991772".to_string())),
        (vec![PID_RECOVERY_CODE], AttributeValue::Text("123".to_string())),
        (
            vec![PID_ADDRESS_GROUP, PID_RESIDENT_STREET],
            AttributeValue::Text("Turfmarkt".to_string()),
        ),
        (
            vec![PID_ADDRESS_GROUP, PID_RESIDENT_HOUSE_NUMBER],
            AttributeValue::Text("147".to_string()),
        ),
        (
            vec![PID_ADDRESS_GROUP, PID_RESIDENT_POSTAL_CODE],
            AttributeValue::Text("2511 DP".to_string()),
        ),
        (
            vec![PID_ADDRESS_GROUP, PID_RESIDENT_CITY],
            AttributeValue::Text("Den Haag".to_string()),
        ),
        (
            vec![PID_ADDRESS_GROUP, PID_RESIDENT_COUNTRY],
            AttributeValue::Text("Nederland".to_string()),
        ),
    ])
}
