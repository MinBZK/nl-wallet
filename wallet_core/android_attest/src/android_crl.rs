use std::collections::HashMap;

use chrono::NaiveDate;
use num_bigint::BigUint;
use num_traits::Num;
use nutype::nutype;
use reqwest::Client;
use reqwest::StatusCode;
use serde::Deserialize;
use x509_parser::certificate::X509Certificate;

/// A NewType for the serial number.
/// This type supports SerialNumbers of up to 20 bytes in accordance to
/// [the spec](https://datatracker.ietf.org/doc/html/rfc5280#section-4.1.2.2).
#[nutype(
    sanitize(trim, uppercase),
    validate(not_empty, len_char_max = 40, regex = "[a-fA-F0-9]+"),
    derive(Debug, Clone, Deserialize, PartialEq, Eq, Hash, AsRef)
)]
pub struct SerialNumber(String);

impl SerialNumber {
    fn serial(&self) -> BigUint {
        BigUint::from_str_radix(self.as_ref(), 16).expect("nutype validation applied")
    }
}

/// Root type of the schema as defined in: https://developer.android.com/privacy-and-security/security-key-attestation#certificate_status
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AndroidCrl {
    pub entries: HashMap<SerialNumber, AndroidCrlEntry>,
}

impl AndroidCrl {
    pub fn to_biguint_map(&self) -> HashMap<BigUint, AndroidCrlEntry> {
        self.entries
            .iter()
            .map(|(serial, entry)| (serial.serial(), entry.clone()))
            .collect()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AndroidCrlEntry {
    pub status: AndroidCrlStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub expires: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub reason: Option<AndroidCrlReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
pub enum AndroidCrlStatus {
    #[serde(rename = "REVOKED")]
    Revoked,
    #[serde(rename = "SUSPENDED")]
    Suspended,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
pub enum AndroidCrlReason {
    #[serde(rename = "UNSPECIFIED")]
    Unspecified,
    #[serde(rename = "KEY_COMPROMISE")]
    KeyCompromise,
    #[serde(rename = "CA_COMPROMISE")]
    CaCompromise,
    #[serde(rename = "SUPERSEDED")]
    Superseded,
    #[serde(rename = "SOFTWARE_FLAW")]
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

pub struct GoogleRevocationList {
    crl: String,
    client: Client,
}

impl GoogleRevocationList {
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

    pub async fn get(&self) -> Result<AndroidCrl, Error> {
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

/// Return all revoked certificates from [`certificate_chain`].
/// The CRL is provided by [`revocation_list`].
pub fn get_revoked_certificates<'a>(
    crl: &'a HashMap<BigUint, AndroidCrlEntry>,
    certificate_chain: &'a [X509Certificate<'a>],
) -> Result<Vec<(&'a X509Certificate<'a>, &'a AndroidCrlEntry)>, Error> {
    let revoked_certificates = certificate_chain
        .iter()
        .flat_map(move |cert| crl.get(&cert.serial).map(move |entry| (cert, entry)))
        .collect();
    Ok(revoked_certificates)
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
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
        let crl_provider = GoogleRevocationList::new_with_client(Default::default());
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
        let crl_provider = GoogleRevocationList::for_test(format!("{base_url}/status"), Client::default());
        let crl = crl_provider.get().await?.to_biguint_map();
        assert_eq!(crl.len(), 5);

        // verify certificate against the crl
        let actual = get_revoked_certificates(&crl, &certificates)?;

        assert_eq!(actual.len(), 1);
        let (_, status_entry) = &actual[0];

        assert_eq!(status_entry.status, AndroidCrlStatus::Suspended);
        assert_eq!(status_entry.reason, Some(AndroidCrlReason::KeyCompromise));
        Ok(())
    }

    // Deserialize example from: https://developer.android.com/privacy-and-security/security-key-attestation#certificate_status
    #[test]
    fn deserialize_example() {
        let actual: AndroidCrl = serde_json::from_slice(STATUS_TESTS_BYTES).unwrap();
        assert_eq!(actual.entries.len(), 3);

        let entry = &actual.entries[&SerialNumber::try_new("2c8cdddfd5e03bfc").unwrap()];

        // Verify first entry
        assert_eq!(entry.status, AndroidCrlStatus::Revoked);
        assert_eq!(
            entry.expires,
            Some(NaiveDate::from_ymd_opt(2020, 11, 13).expect("valid date"))
        );
        assert_eq!(entry.reason, Some(AndroidCrlReason::KeyCompromise));
        assert_eq!(entry.comment, Some("Key stored on unsecure system".to_string()));

        // Verify second entry
        let entry = &actual.entries[&SerialNumber::try_new("c8966fcb2fbb0d7a").unwrap()];
        assert_eq!(entry.status, AndroidCrlStatus::Suspended);
        assert_eq!(entry.expires, None);
        assert_eq!(entry.reason, Some(AndroidCrlReason::SoftwareFlaw));
        assert_eq!(
            entry.comment,
            Some("Bug in keystore causes this key malfunction b/555555".to_string())
        );

        let entry = &actual.entries[&SerialNumber::try_new("1").unwrap()];
        assert_eq!(entry.status, AndroidCrlStatus::Revoked);
        assert_eq!(entry.expires, None);
        assert_eq!(entry.reason, None);
        assert_eq!(entry.comment, None);
    }
}
