use serde::Deserialize;

/// Algorithms as defined in [IANA.JOSE].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, strum::Display, serde_with::SerializeDisplay)]
pub enum JwsAlgorithm {
    ES256,

    // Allow the issuer to announce algorithms that the wallet doesn't support.
    #[serde(untagged)]
    Other(String),
}
