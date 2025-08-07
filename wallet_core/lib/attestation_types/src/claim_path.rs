use std::fmt::Display;
use std::fmt::Formatter;
use std::num::ParseIntError;
use std::str::FromStr;

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

impl Display for ClaimPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ClaimPath::SelectByKey(key) => write!(f, "{key}"),
            ClaimPath::SelectAll => f.write_str("null"),
            ClaimPath::SelectByIndex(index) => write!(f, "{index}"),
        }
    }
}

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
