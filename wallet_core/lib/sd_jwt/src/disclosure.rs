// Copyright 2020-2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::fmt::Display;
use std::str::FromStr;

use base64::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use serde_with::DeserializeFromStr;
use serde_with::SerializeDisplay;

use crate::error::Error;

/// A disclosable value.
/// Both object properties and array elements disclosures are supported.
///
/// See: https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-07.html#name-disclosures
// TODO: [PVW-4138] Update link and check spec changes
#[derive(Debug, Clone, Eq, SerializeDisplay, DeserializeFromStr)]
pub struct Disclosure {
    /// Indicates whether this disclosure is an object property or array element.
    pub content: DisclosureContent,

    /// Base64Url-encoded disclosure.
    encoded: String,
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
                    Error::InvalidDisclosure(format!("decoded disclosure could not be serialized as an array {s}"))
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
    pub key: Option<String>,
    pub value: serde_json::Value,
    pub error: serde_json::Error,
}

impl AsRef<str> for Disclosure {
    fn as_ref(&self) -> &str {
        &self.encoded
    }
}

impl Disclosure {
    /// Creates a new instance of [`Disclosure`].
    ///
    /// Use `.to_string()` to get the actual disclosure.
    pub(crate) fn try_new(content: DisclosureContent) -> Result<Self, DisclosureContentSerializationError> {
        let serialized = match serde_json::to_vec(&content) {
            Ok(serialized) => serialized,
            Err(error) => {
                return match content {
                    DisclosureContent::ObjectProperty(_, key, value) => Err(DisclosureContentSerializationError {
                        key: Some(key),
                        value,
                        error,
                    }),
                    DisclosureContent::ArrayElement(_, value) => Err(DisclosureContentSerializationError {
                        key: None,
                        value,
                        error,
                    }),
                };
            }
        };

        let encoded = BASE64_URL_SAFE_NO_PAD.encode(serialized.as_slice());
        Ok(Self { content, encoded })
    }

    pub fn as_str(&self) -> &str {
        self.as_ref()
    }

    pub fn claim_value(&self) -> &serde_json::Value {
        self.content.claim_value()
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
        /// The claim name, optional for array elements.
        String,
        /// The claim Value which can be of any type.
        serde_json::Value,
    ),
    ArrayElement(
        /// The salt value.
        String,
        /// The claim Value which can be of any type.
        serde_json::Value,
    ),
}

impl DisclosureContent {
    pub fn claim_value(&self) -> &serde_json::Value {
        match &self {
            DisclosureContent::ObjectProperty(_, _, value) => value,
            DisclosureContent::ArrayElement(_, value) => value,
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

    use crate::error::Error;

    use super::Disclosure;
    use super::DisclosureContent;

    // Test values from:
    // https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-07.html#appendix-A.2-7
    #[test]
    fn test_parsing_value() {
        let disclosure = Disclosure::try_new(DisclosureContent::ObjectProperty(
            "2GLC42sKQveCfGfryNRN9w".to_string(),
            "time".to_owned(),
            "2012-04-23T18:25Z".to_owned().into(),
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
            "US".to_owned().into(),
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
