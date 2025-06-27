// Copyright 2020-2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::fmt::Display;

use base64::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::error::Error;

/// A disclosable value.
/// Both object properties and array elements disclosures are supported.
///
/// See: https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-07.html#name-disclosures
// TODO: [PVW-4138] Update link and check spec changes
#[derive(Debug, Clone, Eq)]
pub struct Disclosure {
    /// Indicates whether this disclosure is an object property or array element.
    pub content: DisclosureContent,

    /// Base64Url-encoded disclosure.
    encoded: String,
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
    pub(crate) fn try_new(content: DisclosureContent) -> Result<Self, Error> {
        let encoded = BASE64_URL_SAFE_NO_PAD.encode(serde_json::to_vec(&content)?.as_slice());
        Ok(Self { content, encoded })
    }

    /// Parses a Base64 encoded disclosure into a [`Disclosure`].
    ///
    /// ## Error
    ///
    /// Returns an [`Error::InvalidDisclosure`] if input is not a valid disclosure.
    pub fn parse(disclosure: &str) -> Result<Self, Error> {
        let disclosure_type: DisclosureContent = BASE64_URL_SAFE_NO_PAD
            .decode(disclosure)
            .map_err(|_e| {
                Error::InvalidDisclosure(format!(
                    "Base64 decoding of the disclosure was not possible {disclosure}"
                ))
            })
            .and_then(|data| {
                serde_json::from_slice(&data).map_err(|_e| {
                    Error::InvalidDisclosure(format!(
                        "decoded disclosure could not be serialized as an array {disclosure}"
                    ))
                })
            })?;

        Ok(Self {
            content: disclosure_type,
            encoded: disclosure.to_string(),
        })
    }

    pub fn as_str(&self) -> &str {
        self.as_ref()
    }

    pub fn claim_value(&self) -> &Value {
        match &self.content {
            DisclosureContent::ObjectProperty(_, _, value) => value,
            DisclosureContent::ArrayElement(_, value) => value,
        }
    }
}

impl Display for Disclosure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.encoded)
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
        Value,
    ),
    ArrayElement(
        /// The salt value.
        String,
        /// The claim Value which can be of any type.
        Value,
    ),
}

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;
    use base64::prelude::BASE64_URL_SAFE_NO_PAD;
    use base64::Engine;
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

        let parsed =
            Disclosure::parse("WyIyR0xDNDJzS1F2ZUNmR2ZyeU5STjl3IiwgInRpbWUiLCAiMjAxMi0wNC0yM1QxODoyNVoiXQ").unwrap();
        assert_eq!(parsed, disclosure);
    }

    #[test]
    fn test_parsing_array_value() {
        let disclosure = Disclosure::try_new(DisclosureContent::ArrayElement(
            "lklxF5jMYlGTPUovMNIvCA".to_string(),
            "US".to_owned().into(),
        ))
        .unwrap();

        let parsed = Disclosure::parse("WyJsa2x4RjVqTVlsR1RQVW92TU5JdkNBIiwgIlVTIl0").unwrap();
        assert_eq!(parsed, disclosure);
    }

    #[test]
    fn test_parsing_error_empty_disclosure() {
        let salt = BASE64_URL_SAFE_NO_PAD.encode(random_bytes(32));
        let disclosure = serde_json::to_vec(&json!([salt])).unwrap();
        let encoded = BASE64_URL_SAFE_NO_PAD.encode(disclosure);

        assert_matches!(Disclosure::parse(&encoded), Err(Error::InvalidDisclosure(_)));
    }
}
