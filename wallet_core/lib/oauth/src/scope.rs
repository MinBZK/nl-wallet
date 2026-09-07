use std::str::FromStr;

use derive_more::AsRef;
use derive_more::Display;
use serde_with::DeserializeFromStr;
use serde_with::SerializeDisplay;

#[derive(Debug, thiserror::Error)]
#[error("scope is empty or contains invalid characters: {0}")]
pub struct ScopeInvalid(String);

/// An individual scope token as defined by RFC 6749. The scope field in OAuth 2.0 payloads may contain one or more of
/// these, delimited by a space character.
#[derive(Debug, Clone, PartialEq, Eq, Hash, AsRef, Display, SerializeDisplay, DeserializeFromStr)]
#[as_ref(str)]
pub struct Scope(String);

impl Scope {
    fn is_valid_char(character: char) -> bool {
        let codepoint = u32::from(character);

        codepoint == 0x21 || (0x23..=0x5b).contains(&codepoint) || (0x5d..=0x7e).contains(&codepoint)
    }

    pub fn try_new(scope: impl Into<String>) -> Result<Self, ScopeInvalid> {
        let scope = scope.into();

        // According to RFC 6749 a scope token is defined as:
        //
        // scope-token = 1*NQCHAR
        // NQCHAR      = %x21 / %x23-5B / %x5D-7E
        //
        // See: <https://datatracker.ietf.org/doc/html/rfc6749#appendix-A.4>
        if scope.is_empty() || scope.chars().any(|character| !Self::is_valid_char(character)) {
            return Err(ScopeInvalid(scope));
        }

        Ok(Self(scope))
    }

    pub fn into_inner(self) -> String {
        let Self(inner) = self;

        inner
    }
}

impl FromStr for Scope {
    type Err = ScopeInvalid;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_new(s)
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use rstest::rstest;

    use super::Scope;
    use super::ScopeInvalid;

    #[rstest]
    #[case("x", true)]
    #[case("scope", true)]
    #[case("SCOPE", true)]
    #[case("1234567890", true)]
    #[case("!#$%&'()*+,-./:;<=>?@[]^_`{|}~", true)]
    #[case("", false)]
    #[case(r"back\slash", false)]
    #[case("\"quoted\"", false)]
    #[case("enquête", false)]
    #[case("スコープ", false)]
    #[case("🪪", false)]
    fn test_scope(#[case] input: &str, #[case] should_be_valid: bool) {
        let result = input.parse::<Scope>();

        if should_be_valid {
            let scope = result.expect("input scope should be valid");

            assert_eq!(scope.as_ref(), input);
            assert_eq!(scope.into_inner(), input);
        } else {
            let error = result.expect_err("input scope should not be valid");

            assert_matches!(error, ScopeInvalid(scope) if scope == input);
        }
    }
}
