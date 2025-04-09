// Copyright 2020-2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::fmt::Display;

use base64::prelude::*;
use serde_json::json;
use serde_json::Value;

use wallet_common::vec_at_least::VecNonEmpty;

use crate::error::Error;

/// A disclosable value.
/// Both object properties and array elements disclosures are supported.
///
/// See: https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-07.html#name-disclosures
// TODO: [PVW-4138] Update link and check spec changes
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
        let decoded: VecNonEmpty<Value> = BASE64_URL_SAFE_NO_PAD
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

        if ![2, 3].contains(&decoded.len().get()) {
            return Err(Error::InvalidDisclosure(format!(
                "deserialized array has an invalid length of {}",
                decoded.len()
            )));
        }

        let salt = decoded
            .first()
            .as_str()
            .ok_or(Error::InvalidDisclosure(
                "salt could not be parsed as a string".to_string(),
            ))?
            .to_owned();

        let claim_value = decoded.last().clone();

        let claim_name = if decoded.len().get() == 3 {
            Some(
                decoded[1]
                    .as_str()
                    .ok_or(Error::InvalidDisclosure(
                        "claim name could not be parsed as a string".to_string(),
                    ))?
                    .to_owned(),
            )
        } else {
            None
        };

        Ok(Self {
            salt,
            claim_name,
            claim_value,
            unparsed: disclosure.to_string(),
        })
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
    use assert_matches::assert_matches;
    use base64::prelude::BASE64_URL_SAFE_NO_PAD;
    use base64::Engine;
    use serde_json::json;

    use crypto::utils::random_bytes;

    use crate::error::Error;

    use super::Disclosure;

    // Test values from:
    // https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-07.html#appendix-A.2-7
    #[test]
    fn test_parsing_value() {
        let disclosure = Disclosure::new(
            "2GLC42sKQveCfGfryNRN9w".to_string(),
            Some("time".to_owned()),
            "2012-04-23T18:25Z".to_owned().into(),
        );

        let parsed =
            Disclosure::parse("WyIyR0xDNDJzS1F2ZUNmR2ZyeU5STjl3IiwgInRpbWUiLCAiMjAxMi0wNC0yM1QxODoyNVoiXQ").unwrap();
        assert_eq!(parsed, disclosure);
    }

    #[test]
    fn test_parsing_array_value() {
        let disclosure = Disclosure::new("lklxF5jMYlGTPUovMNIvCA".to_string(), None, "US".to_owned().into());

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
