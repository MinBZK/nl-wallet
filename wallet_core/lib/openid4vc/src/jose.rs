use serde::Deserialize;
use serde::Serialize;

/// Algorithms that the Issuer supports for a proof, as defined in [IANA.JOSE]. The Wallet uses one of them to sign the
/// proof.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JwsAlgorithm {
    ES256,

    // Allow the issuer to announce algorithms that the wallet doesn't support.
    #[serde(untagged)]
    Other(String),
}
