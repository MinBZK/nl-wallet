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
    pub fn matches_key_path<'a, 'b>(
        claim_path: impl IntoIterator<Item = &'a Self>,
        key_path: impl IntoIterator<Item = &'b str>,
    ) -> bool {
        let mut key_path = key_path.into_iter();

        // Every element of the claim path has to be a key that matches the next key, with no keys left over.
        let all_match = claim_path
            .into_iter()
            .all(|claim| claim.try_key_path().is_some_and(|key| key_path.next() == Some(key)));

        all_match && key_path.next().is_none()
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

    use utils::vec_at_least::VecNonEmpty;

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

    impl ClaimPath {
        pub fn select_by_keys(keys: &[&str]) -> VecNonEmpty<ClaimPath> {
            keys.iter()
                .map(|key| ClaimPath::SelectByKey(String::from(*key)))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap()
        }
    }
}
