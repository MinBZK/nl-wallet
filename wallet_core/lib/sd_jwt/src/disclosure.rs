use std::fmt::Display;
use std::str::FromStr;

use base64::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use serde_with::DeserializeFromStr;
use serde_with::SerializeDisplay;

use crate::claims::ArrayClaim;
use crate::claims::ClaimName;
use crate::claims::ClaimValue;
use crate::claims::HashType;
use crate::error::Error;

/// A disclosable value.
/// Both object properties and array elements disclosures are supported.
///
/// See: https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-07.html#name-disclosures
#[derive(Debug, Clone, Eq, SerializeDisplay, DeserializeFromStr)]
pub struct Disclosure {
    /// Indicates whether this disclosure is an object property or array element.
    pub content: DisclosureContent,

    /// Base64Url-encoded disclosure.
    pub encoded: String,
}

impl Display for Disclosure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.encoded)
    }
}

/// Parses a Base64 encoded disclosure into a [`Disclosure`].
impl FromStr for Disclosure {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let content: DisclosureContent = BASE64_URL_SAFE_NO_PAD
            .decode(s)
            .map_err(|_| Error::InvalidDisclosure(format!("Base64 decoding of the disclosure was not possible {s}")))
            .and_then(|data| {
                serde_json::from_slice(&data).map_err(|_| {
                    Error::InvalidDisclosure(format!("decoded disclosure could not be deserialized as JSON {s}"))
                })
            })?;

        Ok(Self {
            content,
            encoded: s.to_owned(),
        })
    }
}

#[derive(Debug)]
pub(crate) struct DisclosureContentSerializationError {
    pub(crate) content: Box<DisclosureContent>,
    pub(crate) error: serde_json::Error,
}

impl Disclosure {
    /// Creates a new instance of [`Disclosure`].
    ///
    /// Use `.to_string()` to get the actual disclosure.
    pub(crate) fn try_new(content: DisclosureContent) -> Result<Self, DisclosureContentSerializationError> {
        let serialized = match serde_json::to_vec(&content) {
            Ok(serialized) => serialized,
            Err(error) => {
                return Err(DisclosureContentSerializationError {
                    content: Box::new(content),
                    error,
                });
            }
        };

        let encoded = BASE64_URL_SAFE_NO_PAD.encode(serialized.as_slice());

        Ok(Self { content, encoded })
    }

    pub fn hash_type(&self) -> HashType {
        self.content.hash_type()
    }

    pub fn digests(&self) -> Vec<(String, HashType)> {
        match &self.content {
            DisclosureContent::ObjectProperty(_, _, claim_value) => claim_value.digests(),
            DisclosureContent::ArrayElement(_, array_claim) => match array_claim {
                ArrayClaim::Hash(digest) => vec![(digest.clone(), HashType::Array)],
                ArrayClaim::Value(value) => value.digests(),
            },
        }
    }

    pub fn encoded(&self) -> &str {
        &self.encoded
    }
}

impl PartialEq for Disclosure {
    fn eq(&self, other: &Self) -> bool {
        self.content == other.content
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DisclosureContent {
    ObjectProperty(
        /// The salt value.
        String,
        /// The claim name.
        ClaimName,
        /// The claim value.
        ClaimValue,
    ),
    ArrayElement(
        /// The salt value.
        String,
        /// The array value which can be a an array hash, or a claim value.
        ArrayClaim,
    ),
}

impl DisclosureContent {
    pub fn hash_type(&self) -> HashType {
        match &self {
            DisclosureContent::ObjectProperty(..) => HashType::Object,
            DisclosureContent::ArrayElement(..) => HashType::Array,
        }
    }

    pub fn try_as_object_property(&self, digest: &str) -> Result<(&String, &ClaimName, &ClaimValue), Error> {
        if let DisclosureContent::ObjectProperty(salt, name, value) = self {
            Ok((salt, name, value))
        } else {
            let expected_hash_type = HashType::Object;
            let actual_hash_type = self.hash_type();
            Err(Error::DataTypeMismatch(format!(
                "Expected an {expected_hash_type:?} element, but got an {actual_hash_type:?} element for digest \
                 `{digest}`"
            )))
        }
    }

    pub fn try_as_array_element(&self, digest: &str) -> Result<(&String, &ArrayClaim), Error> {
        if let DisclosureContent::ArrayElement(salt, value) = self {
            Ok((salt, value))
        } else {
            let expected_hash_type = HashType::Array;
            let actual_hash_type = self.hash_type();
            Err(Error::DataTypeMismatch(format!(
                "Expected an {expected_hash_type:?} element, but got an {actual_hash_type:?} element for digest \
                 `{digest}`"
            )))
        }
    }
}

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;
    use base64::Engine;
    use base64::prelude::BASE64_URL_SAFE_NO_PAD;
    use serde_json::json;

    use crypto::utils::random_bytes;

    use crate::claims::ClaimValue;
    use crate::error::Error;

    use super::Disclosure;
    use super::DisclosureContent;

    // Test values from:
    // https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-07.html#appendix-A.2-7
    #[test]
    fn test_parsing_value() {
        let disclosure = Disclosure::try_new(DisclosureContent::ObjectProperty(
            "2GLC42sKQveCfGfryNRN9w".to_string(),
            "time".parse().unwrap(),
            ClaimValue::String("2012-04-23T18:25Z".to_string()),
        ))
        .unwrap();

        let parsed: Disclosure = "WyIyR0xDNDJzS1F2ZUNmR2ZyeU5STjl3IiwgInRpbWUiLCAiMjAxMi0wNC0yM1QxODoyNVoiXQ"
            .parse()
            .unwrap();
        assert_eq!(parsed, disclosure);
    }

    #[test]
    fn test_parsing_array_value() {
        let disclosure = Disclosure::try_new(DisclosureContent::ArrayElement(
            "lklxF5jMYlGTPUovMNIvCA".to_string(),
            ClaimValue::String("US".to_string()).into(),
        ))
        .unwrap();

        let parsed: Disclosure = "WyJsa2x4RjVqTVlsR1RQVW92TU5JdkNBIiwgIlVTIl0".parse().unwrap();
        assert_eq!(parsed, disclosure);
    }

    #[test]
    fn test_parsing_error_empty_disclosure() {
        let salt = BASE64_URL_SAFE_NO_PAD.encode(random_bytes(32));
        let disclosure = serde_json::to_vec(&json!([salt])).unwrap();
        let encoded = BASE64_URL_SAFE_NO_PAD.encode(disclosure);

        assert_matches!(&encoded.parse::<Disclosure>(), Err(Error::InvalidDisclosure(_)));
    }
}
