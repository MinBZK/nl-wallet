use nutype::nutype;
use serde::Deserialize;

#[nutype(
    derive(Debug, Clone, TryFrom, AsRef, Hash, PartialEq, Eq, Deserialize),
    validate(regex = r"^[\w-]+$")
)]
pub struct Kid(String);

/// A pair of key identifiers ("kid"s) for a symmetric encryption key: the one currently used to encrypt, and
/// optionally the previous one, still needed to decrypt data that hasn't been re-encrypted yet as part of a
/// key rollover.
#[derive(Debug, Clone, Deserialize)]
pub struct KidPair {
    pub current: Kid,
    #[serde(default)]
    pub previous: Option<Kid>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown key kid: {0:?}")]
pub struct UnknownKid(pub Kid);

impl KidPair {
    /// Errors if `kid` is neither the current nor the previous kid.
    pub fn validate(&self, kid: &Kid) -> Result<(), UnknownKid> {
        if *kid == self.current || self.previous.as_ref().is_some_and(|previous| previous == kid) {
            Ok(())
        } else {
            Err(UnknownKid(kid.clone()))
        }
    }
}
