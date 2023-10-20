use chrono::Duration;
pub use indexmap::IndexMap;
use p256::pkcs8::der::{asn1::Utf8StringRef, Decode, Encode, SliceReader};
use rcgen::CustomExtension;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none, DurationSeconds};
use url::Url;
use x509_parser::der_parser::Oid;

use super::x509::{Certificate, CertificateError};

/// oid: 2.1.123.1
/// root: {joint-iso-itu-t(2) asn1(1) examples(123)}
/// suffix: 1, unofficial id for Reader Authentication
const OID_EXT_READER_AUTH: &[u64] = &[2, 1, 123, 1];

#[skip_serializing_none]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReaderRegistration {
    pub id: String,
    pub name: LocalizedStrings,
    pub purpose_statement: LocalizedStrings,
    pub retention_policy: RetentionPolicy,
    pub sharing_policy: SharingPolicy,
    pub deletion_policy: DeletionPolicy,
    pub organization: Organization,
}

impl ReaderRegistration {
    pub fn from_certificate(source: &Certificate) -> Result<Option<Self>, CertificateError> {
        // unwrap() is safe here, because we process a fixed value
        let oid = Oid::from(OID_EXT_READER_AUTH).unwrap();
        let x509_cert = source.to_x509()?;
        let ext = x509_cert.iter_extensions().find(|ext| ext.oid == oid);
        let registration = match ext {
            Some(ext) => {
                let mut reader = SliceReader::new(ext.value)?;
                let json = Utf8StringRef::decode(&mut reader)?;
                let registration = serde_json::from_str(json.as_str())?;
                Some(registration)
            }
            None => None,
        };
        Ok(registration)
    }

    pub fn to_custom_ext(&self) -> Result<CustomExtension, CertificateError> {
        let json_string = serde_json::to_string(self)?;
        let string = Utf8StringRef::new(&json_string)?;
        let ext = CustomExtension::from_oid_content(OID_EXT_READER_AUTH, string.to_der()?);
        Ok(ext)
    }
}

#[skip_serializing_none]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Organization {
    pub display_name: LocalizedStrings,
    pub legal_name: LocalizedStrings,
    pub description: LocalizedStrings,
    pub logo: Option<Image>,
    pub web_url: Option<Url>,
    pub city: Option<LocalizedStrings>,
    pub country: Option<String>,
    pub privacy_policy_url: Option<Url>,
}

/// Encapsulates an image.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    /// Media Type of the image, expected to start with: `image/`.
    mime_type: String,
    /// String encoded data of the image, f.e. XML text for `image/xml+svg`, or Base64 encoded binary data for
    /// `image/png`.
    image_data: String,
}

type Language = String;

/// Holds multiple translations of the same field
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalizedStrings(pub IndexMap<Language, String>);

/// Allows convenient definitions of [`LocalizedStrings`] in Rust code.
impl From<Vec<(&str, &str)>> for LocalizedStrings {
    fn from(source: Vec<(&str, &str)>) -> Self {
        let mut map = IndexMap::new();
        for (language, value) in source.into_iter() {
            map.insert(language.to_owned(), value.to_owned());
        }
        LocalizedStrings(map)
    }
}

#[serde_as]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetentionPolicy {
    pub intent_to_retain: bool,
    #[serde_as(as = "Option<DurationSeconds<i64>>")]
    #[serde(rename = "maxDurationInSeconds")]
    pub max_duration: Option<Duration>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SharingPolicy {
    pub intent_to_share: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletionPolicy {
    pub deleteable: bool,
}
