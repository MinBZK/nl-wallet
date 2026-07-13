use crypto::PublicKey;
use jsonwebtoken::jwk::Jwk;
use serde::Deserialize;
use serde::Serialize;

use crate::error::JwkConversionError;
use crate::jwk::jwk_from_public_key;
use crate::jwk::jwk_to_public_key;

/// Proof of possession of a given key.
///
/// Currently, only Jwk is supported. See [RFC7800](https://www.rfc-editor.org/rfc/rfc7800.html#section-3)
/// details.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmationClaim {
    /// Json Web Key (JWK).
    Jwk(Jwk),
}

impl ConfirmationClaim {
    pub fn try_from_public_key(pubkey: &PublicKey) -> Result<Self, JwkConversionError> {
        Ok(Self::Jwk(jwk_from_public_key(pubkey)?))
    }

    pub fn try_to_public_key(&self) -> Result<PublicKey, JwkConversionError> {
        let Self::Jwk(jwk) = self;
        jwk_to_public_key(jwk)
    }
}
