use std::borrow::Borrow;

use nutype::nutype;
use serde::Deserialize;
use serde_with::base64::Base64;
use serde_with::serde_as;

use crate::p256_der::DerVerifyingKey;

/// Kid is a wrapper around a string that represents a key ID.
///
/// Note that the regex chosen is to make guarantees for the HSM label and URL path.
/// If changing this regex, check if it still matches the expectations for these purposes.
#[nutype(
    derive(Debug, Clone, TryFrom, AsRef, Hash, PartialEq, Eq, Deserialize, Serialize),
    validate(regex = r"^[A-Za-z0-9_-]+$")
)]
pub struct Kid(String);

impl Borrow<str> for Kid {
    fn borrow(&self) -> &str {
        self.as_ref()
    }
}

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct VerifyingKeyWithKid {
    pub kid: Kid,
    #[serde_as(as = "Base64")]
    pub public_key: DerVerifyingKey,
}
