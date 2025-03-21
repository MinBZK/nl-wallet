// Copyright 2020-2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::fmt::Display;

use base64::prelude::*;
use serde_json::json;
use serde_json::Value;

use crate::error::Error;

/// A disclosable value.
/// Both object properties and array elements disclosures are supported.
///
/// See: https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-07.html#name-disclosures
#[derive(Debug, Clone, Eq)]
pub struct Disclosure {
    /// The salt value.
    pub salt: String,
    /// The claim name, optional for array elements.
    pub claim_name: Option<String>,
    /// The claim Value which can be of any type.
    pub claim_value: Value,
    /// Base64Url-encoded disclosure.
    unparsed: String,
}

impl AsRef<str> for Disclosure {
    fn as_ref(&self) -> &str {
        &self.unparsed
    }
}

impl Disclosure {
    /// Creates a new instance of [`Disclosure`].
    ///
    /// Use `.to_string()` to get the actual disclosure.
    pub(crate) fn new(salt: String, claim_name: Option<String>, claim_value: Value) -> Self {
        let string_encoded = {
            let json_input = if let Some(name) = claim_name.as_deref() {
                json!([salt, name, claim_value])
            } else {
                json!([salt, claim_value])
            };

            BASE64_URL_SAFE_NO_PAD.encode(json_input.to_string())
        };
        Self {
            salt,
            claim_name,
            claim_value,
            unparsed: string_encoded,
        }
    }

    /// Parses a Base64 encoded disclosure into a [`Disclosure`].
    ///
    /// ## Error
    ///
    /// Returns an [`Error::InvalidDisclosure`] if input is not a valid disclosure.
    pub fn parse(disclosure: &str) -> Result<Self, Error> {
        let decoded: Vec<Value> = BASE64_URL_SAFE_NO_PAD
            .decode(disclosure)
            .map_err(|_e| {
                Error::InvalidDisclosure(format!(
                    "Base64 decoding of the disclosure was not possible {}",
                    disclosure
                ))
            })
            .and_then(|data| {
                serde_json::from_slice(&data).map_err(|_e| {
                    Error::InvalidDisclosure(format!(
                        "decoded disclosure could not be serialized as an array {}",
                        disclosure
                    ))
                })
            })?;

        if decoded.len() == 2 {
            Ok(Self {
                salt: decoded
                    .first()
                    .ok_or(Error::InvalidDisclosure("invalid salt".to_string()))?
                    .as_str()
                    .ok_or(Error::InvalidDisclosure(
                        "salt could not be parsed as a string".to_string(),
                    ))?
                    .to_owned(),
                claim_name: None,
                claim_value: decoded
                    .get(1)
                    .ok_or(Error::InvalidDisclosure("invalid claim name".to_string()))?
                    .clone(),
                unparsed: disclosure.to_string(),
            })
        } else if decoded.len() == 3 {
            Ok(Self {
                salt: decoded
                    .first()
                    .ok_or(Error::InvalidDisclosure("invalid salt".to_string()))?
                    .as_str()
                    .ok_or(Error::InvalidDisclosure(
                        "salt could not be parsed as a string".to_string(),
                    ))?
                    .to_owned(),
                claim_name: Some(
                    decoded
                        .get(1)
                        .ok_or(Error::InvalidDisclosure("invalid claim name".to_string()))?
                        .as_str()
                        .ok_or(Error::InvalidDisclosure(
                            "claim name could not be parsed as a string".to_string(),
                        ))?
                        .to_owned(),
                ),
                claim_value: decoded
                    .get(2)
                    .ok_or(Error::InvalidDisclosure("invalid claim name".to_string()))?
                    .clone(),
                unparsed: disclosure.to_string(),
            })
        } else {
            Err(Error::InvalidDisclosure(format!(
                "deserialized array has an invalid length of {}",
                decoded.len()
            )))
        }
    }

    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

impl Display for Disclosure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.unparsed)
    }
}

impl PartialEq for Disclosure {
    fn eq(&self, other: &Self) -> bool {
        self.claim_name == other.claim_name && self.salt == other.salt && self.claim_value == other.claim_value
    }
}

#[cfg(test)]
mod test {
    use super::Disclosure;

    // Test values from:
    // https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-07.html#appendix-A.2-7
    #[test]
    fn test_parsing() {
        let disclosure = Disclosure::new(
            "2GLC42sKQveCfGfryNRN9w".to_string(),
            Some("time".to_owned()),
            "2012-04-23T18:25Z".to_owned().into(),
        );

        let parsed =
            Disclosure::parse("WyIyR0xDNDJzS1F2ZUNmR2ZyeU5STjl3IiwgInRpbWUiLCAiMjAxMi0wNC0yM1QxODoyNVoiXQ").unwrap();
        assert_eq!(parsed, disclosure);
    }
}
