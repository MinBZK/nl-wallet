use std::collections::HashMap;

use chrono::NaiveDate;
use num_bigint::BigUint;
use num_traits::Num;
use nutype::nutype;
use reqwest::Client;
use reqwest::StatusCode;
use serde::Deserialize;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use serde_with::FromInto;
use x509_parser::certificate::X509Certificate;

/// A NewType for the serial number.
/// This type supports SerialNumbers of up to 20 bytes in accordance to
/// [the spec](https://datatracker.ietf.org/doc/html/rfc5280#section-4.1.2.2).
#[nutype(
    sanitize(trim, lowercase),
    validate(not_empty, len_char_max = 40, regex = "^[a-f1-9][a-f0-9]*$"),
    derive(Debug, Clone, Deserialize, PartialEq, Eq, Hash, AsRef)
)]
pub struct SerialNumber(String);

impl From<SerialNumber> for BigUint {
    fn from(value: SerialNumber) -> Self {
        BigUint::from_str_radix(value.as_ref(), 16).expect("nutype validation applied")
    }
}

/// Root type of the schema as defined in: https://developer.android.com/privacy-and-security/security-key-attestation#certificate_status
#[serde_as]
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RevocationStatusList {
    #[serde_as(as = "FromInto<RevocationStatusEntries>")]
    pub entries: HashMap<BigUint, RevocationStatusEntry>,
}

impl RevocationStatusList {
    /// Return all revoked certificates from [`certificate_chain`], together with the reason as a tuple.
    pub fn get_revoked_certificates<'a>(
        &'a self,
        certificate_chain: &'a [X509Certificate<'a>],
    ) -> Result<Vec<(&'a X509Certificate<'a>, &'a RevocationStatusEntry)>, Error> {
        let revoked_certificates = certificate_chain
            .iter()
            .flat_map(move |cert| self.entries.get(&cert.serial).map(move |entry| (cert, entry)))
            .collect();
        Ok(revoked_certificates)
    }
}

/// Intermediate representation of all revocation status entries.
/// Will be converted into a [`HashMap<BigUint, RevocationStatusEntry>`] using `serde_as`.
#[nutype(derive(Debug, Clone, Deserialize, PartialEq, Eq, AsRef))]
struct RevocationStatusEntries(HashMap<SerialNumber, RevocationStatusEntry>);

impl From<RevocationStatusEntries> for HashMap<BigUint, RevocationStatusEntry> {
    fn from(crl: RevocationStatusEntries) -> Self {
        crl.into_inner()
            .into_iter()
            .map(|(serial, entry)| (serial.into(), entry))
            .collect()
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct RevocationStatusEntry {
    pub status: AndroidCrlStatus,
    #[serde(default)]
    pub expires: Option<NaiveDate>,
    #[serde(default)]
    pub reason: Option<AndroidCrlReason>,
    #[serde(default)]
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AndroidCrlStatus {
    Revoked,
    Suspended,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AndroidCrlReason {
    Unspecified,
    KeyCompromise,
    CaCompromise,
    Superseded,
    SoftwareFlaw,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("http status code {0}, with message: {1}")]
    HttpFailure(StatusCode, String),
}

const ANDROID_CRL: &str = "https://android.googleapis.com/attestation/status";

pub struct GoogleRevocationListClient {
    crl: String,
    client: Client,
}

impl GoogleRevocationListClient {
    /// Constructor with [`client`].
    /// It is recommended to use a caching middleware, like `http-cache-reqwest`.
    pub fn new_with_client(client: Client) -> Self {
        Self {
            crl: String::from(ANDROID_CRL),
            client,
        }
    }

    #[cfg(test)]
    pub(crate) fn for_test(crl: String, client: Client) -> Self {
        Self { crl, client }
    }

    pub async fn get(&self) -> Result<RevocationStatusList, Error> {
        let response = self.client.get(&self.crl).send().await?;

        // Check if status is success.
        let status = response.status();
        if !status.is_success() {
            return Err(Error::HttpFailure(status, response.text().await?));
        }

        let crl_data = response.json().await?;

        Ok(crl_data)
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use rstest::rstest;
    use wiremock::matchers::method;
    use wiremock::matchers::path;
    use wiremock::Mock;
    use wiremock::MockServer;
    use wiremock::ResponseTemplate;
    use x509_parser::pem;
    use x509_parser::prelude::FromDer;
    use x509_parser::prelude::X509Certificate;

    use super::*;

    // status.json is taken from repo: https://github.com/google/android-key-attestation.git
    const STATUS_TESTS_BYTES: &[u8] = include_bytes!("../test-assets/status-tests.json");

    // status.json is taken from repo: https://github.com/google/android-key-attestation.git
    const TEST_ASSETS_STATUS_BYTES: &[u8] = include_bytes!("../test-assets/status.json");

    // example certificate taken from repo: https://github.com/google/android-key-attestation.git
    // this certificate is suspended according to status.json
    const TEST_ASSETS_SUSPENDED_CERT: &[u8] = include_bytes!("../test-assets/suspended-cert.pem");

    async fn start_google_crl_server() -> MockServer {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/status"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(TEST_ASSETS_STATUS_BYTES)
                    .append_header("Cache-Control", "max-age=3600"),
            )
            .expect(1)
            .mount(&server)
            .await;

        server
    }

    /// This test just exists to check `GoogleRevocationList` against the official google URL.
    /// Since this requires network, it is disabled by default, enable with feature "network_test".
    #[cfg(feature = "network_test")]
    #[tokio::test]
    async fn test_google_crl_network() {
        let crl_provider = GoogleRevocationListClient::new_with_client(Default::default());
        let crl = crl_provider.get().await.unwrap();
        assert!(!crl.entries.is_empty());
    }

    #[tokio::test]
    async fn test_check_revoked_certificates() -> Result<(), Box<dyn std::error::Error>> {
        let crl_server = start_google_crl_server().await;
        let base_url = crl_server.uri();

        // prepare test certificate list
        let (_, cert_pem) = pem::parse_x509_pem(TEST_ASSETS_SUSPENDED_CERT)?;
        let (_, cert) = X509Certificate::from_der(&cert_pem.contents)?;
        let certificates = [cert];

        // create caching CRL provider, and verify entries are read
        let crl_provider = GoogleRevocationListClient::for_test(format!("{base_url}/status"), Client::default());
        let crl = crl_provider.get().await?;
        assert_eq!(crl.entries.len(), 5);

        // verify certificate against the crl
        let actual = crl.get_revoked_certificates(&certificates)?;

        assert_eq!(actual.len(), 1);
        let (_, status_entry) = &actual[0];

        assert_eq!(status_entry.status, AndroidCrlStatus::Suspended);
        assert_eq!(status_entry.reason, Some(AndroidCrlReason::KeyCompromise));
        Ok(())
    }

    // Deserialize example from: https://developer.android.com/privacy-and-security/security-key-attestation#certificate_status
    #[test]
    fn deserialize_example() {
        let actual: RevocationStatusList = serde_json::from_slice(STATUS_TESTS_BYTES).unwrap();
        assert_eq!(actual.entries.len(), 3);

        let entry = &actual.entries[&BigUint::parse_bytes(b"2c8cdddfd5e03bfc", 16).unwrap()];

        // Verify first entry
        assert_eq!(entry.status, AndroidCrlStatus::Revoked);
        assert_eq!(
            entry.expires,
            Some(NaiveDate::from_ymd_opt(2020, 11, 13).expect("valid date"))
        );
        assert_eq!(entry.reason, Some(AndroidCrlReason::KeyCompromise));
        assert_eq!(entry.comment, Some("Key stored on unsecure system".to_string()));

        // Verify second entry
        let entry = &actual.entries[&BigUint::parse_bytes(b"c8966fcb2fbb0d7a", 16).unwrap()];
        assert_eq!(entry.status, AndroidCrlStatus::Suspended);
        assert_eq!(entry.expires, None);
        assert_eq!(entry.reason, Some(AndroidCrlReason::SoftwareFlaw));
        assert_eq!(
            entry.comment,
            Some("Bug in keystore causes this key malfunction b/555555".to_string())
        );

        let entry = &actual.entries[&BigUint::parse_bytes(b"1", 16).unwrap()];
        assert_eq!(entry.status, AndroidCrlStatus::Revoked);
        assert_eq!(entry.expires, None);
        assert_eq!(entry.reason, None);
        assert_eq!(entry.comment, None);
    }

    #[rstest]
    #[case(r#""1""#, b"1")]
    #[case(r#""100""#, b"100")]
    #[case(r#""2c8cdddfd5e03bfc""#, b"2c8cdddfd5e03bfc")]
    #[case(r#""c8966fcb2fbb0d7a""#, b"c8966fcb2fbb0d7a")]
    fn deserialize_serialnumber_success(#[case] json_biguint: String, #[case] bytes_biguint: &[u8]) {
        let actual: SerialNumber = serde_json::from_str(&json_biguint).unwrap();
        assert_eq!(BigUint::from(actual), BigUint::parse_bytes(bytes_biguint, 16).unwrap());
    }

    #[rstest]
    #[case(r#""""#)]
    #[case(r#""02c8cdddfd5e03bfc""#)]
    #[case(r#""xyz""#)]
    #[case(r#""2c8cdddfd5e03bfc2c8cdddfd5e03bfc2c8cdddfd5e03bfc""#)]
    fn deserialize_serialnumber_failure(#[case] json_biguint: String) {
        serde_json::from_str::<SerialNumber>(&json_biguint).expect_err("should fail");
    }
}
