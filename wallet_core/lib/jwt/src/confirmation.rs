use jsonwebtoken::jwk::Jwk;
use p256::ecdsa::VerifyingKey;
use serde::Deserialize;
use serde::Serialize;

use crate::error::JwkConversionError;
use crate::jwk::jwk_from_p256;
use crate::jwk::jwk_to_p256;

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
    pub fn from_verifying_key(pubkey: &VerifyingKey) -> Result<Self, JwkConversionError> {
        let jwk = jwk_from_p256(pubkey)?;
        Ok(Self::Jwk(jwk))
    }

    pub fn verifying_key(&self) -> Result<VerifyingKey, JwkConversionError> {
        let Self::Jwk(jwk) = self;
        jwk_to_p256(jwk)
    }
}
