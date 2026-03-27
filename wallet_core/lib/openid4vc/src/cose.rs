use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Eq, Serialize, Deserialize)]
#[serde(from = "i64", into = "i64")]
pub enum CoseAlgorithmIdentifier {
    Known(KnownCoseAlgorithmIdentifier),

    // Allow the issuer to COSE algorithm identifiers that the wallet doesn't support.
    Unknown(i64),
}

impl CoseAlgorithmIdentifier {
    /// Returns `true` if this algorithm identifier accepts an ECDSA-P256/SHA-256 signature,
    /// i.e. it is `ES256` (-7, per https://www.iana.org/assignments/cose/cose.xhtml)
    /// or the fully-specified `ESP256` (-9, per https://datatracker.ietf.org/doc/html/rfc9864#section-2.1).
    pub fn is_ecdsa_p256(&self) -> bool {
        match &self {
            CoseAlgorithmIdentifier::Known(KnownCoseAlgorithmIdentifier::Es256)
            | CoseAlgorithmIdentifier::Known(KnownCoseAlgorithmIdentifier::Esp256) => true,
            CoseAlgorithmIdentifier::Unknown(_) => false,
        }
    }
}

impl From<KnownCoseAlgorithmIdentifier> for CoseAlgorithmIdentifier {
    fn from(value: KnownCoseAlgorithmIdentifier) -> Self {
        Self::Known(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::FromRepr)]
#[repr(i64)]
pub enum KnownCoseAlgorithmIdentifier {
    Es256 = -7,
    Esp256 = -9,
}

impl From<CoseAlgorithmIdentifier> for i64 {
    fn from(value: CoseAlgorithmIdentifier) -> Self {
        match value {
            CoseAlgorithmIdentifier::Known(known_identifier) => known_identifier as i64,
            CoseAlgorithmIdentifier::Unknown(identifier) => identifier,
        }
    }
}

impl From<i64> for CoseAlgorithmIdentifier {
    fn from(value: i64) -> Self {
        match KnownCoseAlgorithmIdentifier::from_repr(value) {
            Some(known_identifier) => Self::Known(known_identifier),
            None => Self::Unknown(value),
        }
    }
}

impl PartialEq for CoseAlgorithmIdentifier {
    fn eq(&self, other: &Self) -> bool {
        i64::from(*self) == i64::from(*other)
    }
}
