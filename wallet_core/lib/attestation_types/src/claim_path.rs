use std::fmt::Display;
use std::fmt::Formatter;

use serde::Deserialize;
use serde::Serialize;

/// Element of a claims path pointer.
///
/// <https://openid.net/specs/openid-4-verifiable-presentations-1_0-28.html#name-claims-path-pointer>
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ClaimPath {
    /// Select a claim in an object.
    SelectByKey(String),

    /// Select all elements within an array.
    SelectAll,

    /// Select an element in an array.
    SelectByIndex(usize),
}

impl ClaimPath {
    pub fn matches_key_path<'a>(
        claim_path: impl IntoIterator<Item = &'a Self>,
        key_path: impl IntoIterator<Item = &'a str>,
    ) -> bool {
        itertools::equal(
            claim_path.into_iter().map(ClaimPath::try_key_path),
            key_path.into_iter().map(Some),
        )
    }

    pub fn try_key_path(&self) -> Option<&str> {
        match self {
            ClaimPath::SelectByKey(key) => Some(key.as_str()),
            _ => None,
        }
    }

    pub fn try_into_key_path(self) -> Option<String> {
        match self {
            ClaimPath::SelectByKey(key) => Some(key),
            _ => None,
        }
    }
}

/// A `ClaimPath` can be converted to a string only for error reporting and testing purposes. This implementation is NOT
/// used when serializing the type. Note that output values of `null` and string consisting only of digits are
/// ambiguous.
impl Display for ClaimPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ClaimPath::SelectByKey(key) => write!(f, "{key}"),
            ClaimPath::SelectAll => f.write_str("null"),
            ClaimPath::SelectByIndex(index) => write!(f, "{index}"),
        }
    }
}

#[cfg(feature = "mock")]
mod mock {
    use std::num::ParseIntError;
    use std::str::FromStr;

    use super::ClaimPath;

    /// Parse a `ClaimPath` entry from a string, for use in tests. Note that this makes it impossible to create a
    /// `SelectByKey` entry that contains either the string of `null` or one that contains solely of digits.
    impl FromStr for ClaimPath {
        type Err = ParseIntError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "null" => Ok(ClaimPath::SelectAll),
                s if s.chars().all(|c| c.is_ascii_digit()) => s.parse().map(ClaimPath::SelectByIndex),
                s => Ok(ClaimPath::SelectByKey(String::from(s))),
            }
        }
    }
}
