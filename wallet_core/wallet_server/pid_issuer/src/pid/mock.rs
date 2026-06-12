use std::fs;
use std::path::PathBuf;

use attestation_data::attributes::AttributeValue;
use attestation_data::attributes::Attributes;
use crypto::utils::random_string;
use openid4vc::pkce::S256PkcePair;
use openid4vc::token::AuthorizationCode;
use serde::Serialize;
use url::Url;
use utils::path::prefix_local_path;

use crate::pid::brp::client::BrpClient;
use crate::pid::brp::client::BrpError;
use crate::pid::brp::data::BrpPersons;
use crate::pid::constants::PID_ADDRESS_GROUP;
use crate::pid::constants::PID_AGE_OVER_18;
use crate::pid::constants::PID_BIRTH_DATE;
use crate::pid::constants::PID_BSN;
use crate::pid::constants::PID_FAMILY_NAME;
use crate::pid::constants::PID_GIVEN_NAME;
use crate::pid::constants::PID_NATIONALITY;
use crate::pid::constants::PID_RECOVERY_CODE;
use crate::pid::constants::PID_RESIDENT_CITY;
use crate::pid::constants::PID_RESIDENT_COUNTRY;
use crate::pid::constants::PID_RESIDENT_HOUSE_NUMBER;
use crate::pid::constants::PID_RESIDENT_POSTAL_CODE;
use crate::pid::constants::PID_RESIDENT_STREET;
use crate::pid::digid::DigidClient;
use crate::pid::digid::Error as DigidError;

/// The BSN of the standard mock test subject, present in the haal-centraal-examples BRP test data.
pub const MOCK_BSN: &str = "999991772";

/// Mock [`DigidClient`] that uses a fixed BSN.
///
/// [`UpstreamOidcAuthorizationCodeFlow`]: crate::pid::auth_code_flow::UpstreamOidcAuthorizationCodeFlow
pub struct MockDigidClient {
    bsn: String,
}

impl MockDigidClient {
    pub fn new(bsn: impl Into<String>) -> Self {
        Self { bsn: bsn.into() }
    }
}

impl Default for MockDigidClient {
    fn default() -> Self {
        Self::new(MOCK_BSN)
    }
}

impl DigidClient for MockDigidClient {
    async fn authorization_request(
        &self,
        _client_id: String,
        mut redirect_uri: Url,
        state: String,
        _pkce_pair: &S256PkcePair,
    ) -> Result<Url, DigidError> {
        // "Authenticate" instantly: bounce the user-agent back to the issuer's callback with a
        // fake upstream code (ignored by `bsn` below) and the issuer's state.

        #[derive(Serialize)]
        struct RedirectQuery<'a> {
            code: &'a str,
            state: &'a str,
        }

        let query = serde_qs::to_string(&RedirectQuery {
            code: random_string(32).as_str(),
            state: state.as_str(),
        })
        .expect("encoding (code, state) query string should never fail");
        dbg!(&query);
        redirect_uri.set_query(Some(&query));
        Ok(redirect_uri)
    }

    async fn bsn(
        &self,
        _code: AuthorizationCode,
        _code_verifier: String,
        _redirect_uri: Url,
    ) -> Result<String, DigidError> {
        Ok(self.bsn.clone())
    }
}

/// Mock [`BrpClient`] that serves a fixed haal-centraal persons JSON document. Defaults to the fixture for
/// [`MOCK_BSN`]: Frouke Jansen, whose BRP-derived attributes match [`mock_pid_example`].
pub struct MockBrpClient {
    persons_json: String,
}

impl MockBrpClient {
    pub fn new(persons_json: impl Into<String>) -> Self {
        Self {
            persons_json: persons_json.into(),
        }
    }

    pub fn from_fixture(name: &str) -> MockBrpClient {
        let persons_json = fs::read_to_string(prefix_local_path(PathBuf::from(format!(
            "resources/test/haal-centraal-examples/{name}.json"
        ))))
        .unwrap();

        MockBrpClient::new(persons_json)
    }
}

impl Default for MockBrpClient {
    fn default() -> Self {
        Self::new(include_str!("../../resources/test/haal-centraal-examples/frouke.json"))
    }
}

impl BrpClient for MockBrpClient {
    async fn get_person_by_bsn(&self, _bsn: &str) -> Result<BrpPersons, BrpError> {
        Ok(serde_json::from_str(&self.persons_json)?)
    }
}

/// The PID attribute set the mocked flow issues for [`MOCK_BSN`]: the BRP-derived attributes of
/// the `frouke.json` fixture, plus a fixed recovery code (the real flow inserts an HMAC over the BSN instead).
pub fn mock_pid_example() -> Attributes {
    Attributes::example([
        (vec![PID_FAMILY_NAME], AttributeValue::Text("Jansen".to_string())),
        (vec![PID_GIVEN_NAME], AttributeValue::Text("Frouke".to_string())),
        (vec![PID_BIRTH_DATE], AttributeValue::Text("2000-03-24".to_string())),
        (vec![PID_AGE_OVER_18], AttributeValue::Bool(true)),
        (vec![PID_BSN], AttributeValue::Text("999991772".to_string())),
        (
            vec![PID_NATIONALITY],
            AttributeValue::Array(vec![
                AttributeValue::Text("Nederlandse".to_string()),
                AttributeValue::Text("Belgische".to_string()),
            ]),
        ),
        (vec![PID_RECOVERY_CODE], AttributeValue::Text("1234567".to_string())),
        (
            vec![PID_ADDRESS_GROUP, PID_RESIDENT_STREET],
            AttributeValue::Text("Van Wijngaerdenstraat".to_string()),
        ),
        (
            vec![PID_ADDRESS_GROUP, PID_RESIDENT_HOUSE_NUMBER],
            AttributeValue::Text("1".to_string()),
        ),
        (
            vec![PID_ADDRESS_GROUP, PID_RESIDENT_POSTAL_CODE],
            AttributeValue::Text("2596TW".to_string()),
        ),
        (
            vec![PID_ADDRESS_GROUP, PID_RESIDENT_CITY],
            AttributeValue::Text("Toetsoog".to_string()),
        ),
        (
            vec![PID_ADDRESS_GROUP, PID_RESIDENT_COUNTRY],
            AttributeValue::Text("Nederland".to_string()),
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ensures that data returned by [`MockBrpClient`] and [`mock_pid_example`] are identical, because some tests rely
    /// on this.
    #[tokio::test]
    async fn mock_brp_person_attributes_match_mock_pid_example() {
        let mut persons = MockBrpClient::default().get_person_by_bsn(MOCK_BSN).await.unwrap();
        let attributes = persons.persons.remove(0).into_attributes();

        // The BRP person yields the example attribute set minus the recovery code,
        // which is not BRP data but inserted later by the flow.
        let serde_json::Value::Object(mut expected) = serde_json::to_value(mock_pid_example()).unwrap() else {
            panic!("example attributes should serialize to a JSON object");
        };
        expected.remove(PID_RECOVERY_CODE);

        assert_eq!(
            serde_json::to_value(attributes).unwrap(),
            serde_json::Value::Object(expected)
        );
    }
}
