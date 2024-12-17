use cache_control::CacheControl;
use chrono::NaiveDate;
use indexmap::IndexMap;
use num_bigint::BigUint;
use num_traits::Num;
use nutype::nutype;
use reqwest::Client;
use reqwest::StatusCode;
use serde::Deserialize;
use x509_parser::certificate::X509Certificate;

use crate::expiring_cache::ExpiringValue;
use crate::expiring_cache::Provider;

/// A NewType for the serial number.
/// This type supports SerialNumbers of up to 20 bytes in accordance to
/// [the spec](https://datatracker.ietf.org/doc/html/rfc5280#section-4.1.2.2).
#[nutype(
    sanitize(trim, uppercase),
    validate(not_empty, len_char_max = 40, regex = "[a-fA-F0-9]+"),
    default = "0",
    derive(Debug, Clone, Default, Deserialize, PartialEq, Eq, Hash, AsRef, Deref)
)]
pub struct SerialNumber(String);

impl SerialNumber {
    fn serial(&self) -> BigUint {
        BigUint::from_str_radix(self, 16).expect("nutype validation applied")
    }
}

impl TryFrom<BigUint> for SerialNumber {
    type Error = __nutype_SerialNumber__::SerialNumberError;

    fn try_from(value: BigUint) -> Result<Self, Self::Error> {
        let hex = value.to_str_radix(16);
        SerialNumber::try_new(hex)
    }
}

/// Root type of the schema as defined in: https://developer.android.com/privacy-and-security/security-key-attestation#certificate_status
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AndroidCrl {
    pub entries: IndexMap<SerialNumber, AndroidCrlEntry>,
}

impl AndroidCrl {
    pub fn as_biguint_map(&self) -> IndexMap<BigUint, AndroidCrlEntry> {
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

const ANDROID_CRL: &str = "https://android.googleapis.com/attestation/status";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("http status code {0}, with message: {1}")]
    HttpFailure(StatusCode, String),
    #[error("Cache-Control header is missing from response")]
    MissingCacheControlHeader,
    #[error("Cache-Control header cannot be represented as str")]
    InvalidStrCacheControlHeader(#[from] reqwest::header::ToStrError),
    #[error("Cache-Control header could not be parsed")]
    InvalidCacheControlHeader,
    #[error("Cache-Control header does not contain (valid) `max-age` part")]
    MissingMaxAge,
}

impl Provider<ExpiringValue<AndroidCrl>> for Client {
    type Error = Error;

    async fn provide(&self) -> Result<ExpiringValue<AndroidCrl>, Self::Error> {
        let response = self.get(ANDROID_CRL).send().await?;

        // Check if status is success.
        let status = response.status();
        if !status.is_success() {
            return Err(Error::HttpFailure(status, response.text().await?));
        }

        // Parse max_age from the `Cache-Control` header.
        let cache_control_header = response
            .headers()
            .get("Cache-Control")
            .ok_or(Error::MissingCacheControlHeader)?
            .to_str()?;
        let max_age = CacheControl::from_value(cache_control_header)
            .ok_or(Error::InvalidCacheControlHeader)?
            .max_age
            .ok_or(Error::MissingMaxAge)?;
        let crl_data = response.json().await?;

        Ok(ExpiringValue::new(crl_data, max_age))
    }
}

/// Return all revoked certificates from [`certificate_chain`].
/// The certificate chain is provided by [`provider`].
async fn get_revoked_certificates<'a, P, E>(
    provider: P,
    certificate_chain: &'a [X509Certificate<'a>],
) -> Result<Vec<(&'a X509Certificate<'a>, AndroidCrlEntry)>, E>
where
    P: Provider<ExpiringValue<AndroidCrl>, Error = E>,
    E: Into<Error>,
{
    let crl = provider.provide().await?.as_biguint_map(); // TODO: can this be implemented as Provider<IndexMap<BigUint, AndroidCrlEntry>>?

    let revoked_certificates = certificate_chain
        .iter()
        .flat_map(move |cert| crl.get(&cert.serial).map(move |entry| (cert, entry.clone())))
        .collect();
    Ok(revoked_certificates)
}

#[cfg(any(test, feature = "mock"))]
mod mock {
    use std::time::Duration;

    use super::*;

    struct MockAndroidCrl;

    const TEST_ASSETS_STATUS_BYTES: &[u8] = include_bytes!("../test-assets/status.json");

    impl Provider<ExpiringValue<AndroidCrl>> for MockAndroidCrl {
        type Error = serde_json::Error;

        async fn provide(&self) -> Result<ExpiringValue<AndroidCrl>, Self::Error> {
            let crl = serde_json::from_slice(TEST_ASSETS_STATUS_BYTES)?;
            let result = ExpiringValue::new(crl, Duration::from_secs(24 * 60 * 60));
            Ok(result)
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    // Deserialize example from: https://developer.android.com/privacy-and-security/security-key-attestation#certificate_status
    #[test]
    fn deserialize_example() {
        let example_json = r#"
          {
            "entries": {
              "2c8cdddfd5e03bfc": {
                "status": "REVOKED",
                "expires": "2020-11-13",
                "reason": "KEY_COMPROMISE",
                "comment": "Key stored on unsecure system"
              },
              "c8966fcb2fbb0d7a": {
                "status": "SUSPENDED",
                "reason": "SOFTWARE_FLAW",
                "comment": "Bug in keystore causes this key malfunction b/555555"
              }
            }
          }
        "#;
        let actual: AndroidCrl = serde_json::from_str(example_json).unwrap();
        assert_eq!(actual.entries.len(), 2);
        let mut iter = actual.entries.into_iter();

        // Verify first entry
        if let Some((key, entry)) = iter.next() {
            assert_eq!(key, SerialNumber::try_new("2c8cdddfd5e03bfc").unwrap());
            assert_eq!(entry.status, AndroidCrlStatus::Revoked);
            assert_eq!(
                entry.expires,
                Some(NaiveDate::from_ymd_opt(2020, 11, 13).expect("valid date"))
            );
            assert_eq!(entry.reason, Some(AndroidCrlReason::KeyCompromise));
            assert_eq!(entry.comment, Some("Key stored on unsecure system".to_string()));
        } else {
            panic!("Should not happen, because of len check above");
        }

        // Verify second entry
        if let Some((key, entry)) = iter.next() {
            assert_eq!(key, SerialNumber::try_new("c8966fcb2fbb0d7a").unwrap());
            assert_eq!(entry.status, AndroidCrlStatus::Suspended);
            assert_eq!(entry.expires, None);
            assert_eq!(entry.reason, Some(AndroidCrlReason::SoftwareFlaw));
            assert_eq!(
                entry.comment,
                Some("Bug in keystore causes this key malfunction b/555555".to_string())
            );
        } else {
            panic!("Should not happen, because of len check above");
        }
    }

    #[tokio::test]
    async fn test_client() {
        let client = Client::default();
        let crl = client.provide().await.unwrap();
        assert!(!crl.entries.is_empty());
    }

    #[test]
    fn test_serial_number() {
        let serial = SerialNumber::try_new("e24e5301403dcb9bad30918083fa15c7").unwrap();
        println!("serial: {serial:?}");
        println!("biguint: {}", serial.serial());

        let biguint = BigUint::new(vec![8, 0, 0, 0]);
        let serial = SerialNumber::try_from(biguint.clone()).unwrap();
        println!("serial: {serial:?}");
        assert_eq!(serial.serial(), biguint);
    }
}
