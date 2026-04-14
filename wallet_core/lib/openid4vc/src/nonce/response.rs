use serde::Deserialize;
use serde::Serialize;

use jwt::nonce::Nonce;

/// Issuer response containing a fresh `c_nonce` value.
/// See: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-7.2>.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonceResponse {
    /// String containing a challenge to be used when creating a proof of possession of the key (see Section 8.2). It
    /// is at the discretion of the Credential Issuer when to return a new challenge value as opposed to the one
    /// returned in the previous request. New challenge values MUST be unpredictable.
    pub c_nonce: Nonce,
}
