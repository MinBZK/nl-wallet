use std::convert::Infallible;

use attestation_data::attributes::AttributeValue;
use attestation_data::attributes::Attributes;
use openid4vc::Format;
use openid4vc::authorization::VciAuthorizationRequest;
use openid4vc::authorization_code_flow::AuthorizationCodeFlow;
use openid4vc::authorization_code_flow::AuthorizeOutcome;
use openid4vc::issuable_document::IssuableDocument;
use openid4vc::token::TokenRequest;
use url::Url;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_nonempty;
use uuid::Uuid;

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

/// Mock [`AuthorizationCodeFlow`] for pid having a preloaded redirect URL and issuable documents.
pub struct MockPidAuthorizationCodeFlow {
    redirect_to: Url,
    documents: VecNonEmpty<IssuableDocument>,
}

impl MockPidAuthorizationCodeFlow {
    pub fn new(redirect_to: Url, documents: VecNonEmpty<IssuableDocument>) -> Self {
        Self { redirect_to, documents }
    }
}

pub fn mock_issuable_documents_pid() -> VecNonEmpty<IssuableDocument> {
    vec_nonempty![
        IssuableDocument::try_new_with_random_id(
            Format::SdJwt,
            PID_ATTESTATION_TYPE.to_string(),
            eudi_nl_pid_example()
        )
        .unwrap(),
        IssuableDocument::try_new_with_random_id(
            Format::MsoMdoc,
            PID_ATTESTATION_TYPE.to_string(),
            eudi_nl_pid_example()
        )
        .unwrap(),
    ]
}

impl Default for MockPidAuthorizationCodeFlow {
    fn default() -> Self {
        Self::new(
            Url::parse("https://upstream.example.com/authorize").unwrap(),
            mock_issuable_documents_pid(),
        )
    }
}

impl AuthorizationCodeFlow for MockPidAuthorizationCodeFlow {
    type Error = Infallible;

    async fn authorize(&self, request: VciAuthorizationRequest) -> Result<AuthorizeOutcome, Self::Error> {
        // Encode the (untransformed) authorization request as the redirect URL's query string,
        // matching what a real upstream-OIDC flow does. Integration test wallet helpers parse
        // `redirect_uri` and `state` out of this URL to complete the fake OIDC dance.
        let query_string =
            serde_urlencoded::to_string(&request).expect("VciAuthorizationRequest should always urlencode-serialize");
        let mut redirect_url = self.redirect_to.clone();
        redirect_url.set_query(Some(&query_string));
        Ok(AuthorizeOutcome::RedirectTo(redirect_url))
    }

    async fn issuables(&self, _token_request: TokenRequest) -> Result<VecNonEmpty<IssuableDocument>, Self::Error> {
        // Return copies with fresh ids so each call yields unique batch_ids.
        let documents = self
            .documents
            .nonempty_iter()
            .cloned()
            .map(|mut document| {
                document.id = Uuid::new_v4();
                document
            })
            .collect();

        Ok(documents)
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
        (vec![PID_RECOVERY_CODE], AttributeValue::Text("1234567".to_string())),
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
