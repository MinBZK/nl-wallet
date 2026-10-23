use coset::iana;
use coset::iana::EnumI64;
use serde::Deserialize;
use serde::Serialize;

/// A numeric COSE algorithm identifier suitable for use in protocol messages.
///
/// [`CoseAlgorithmIdentifier::Known`] means that this type explicitly models the registered identifier. It does not
/// imply that every signing or verification operation in this crate supports that algorithm. Identifiers not yet
/// modeled by this type, including registered identifiers, are preserved as [`CoseAlgorithmIdentifier::Unknown`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "i64", into = "i64")]
pub enum CoseAlgorithmIdentifier {
    Known(iana::Algorithm),
    Unknown(i64),
}

impl CoseAlgorithmIdentifier {
    /// Whether this identifier is compatible with ECDSA using P-256 and SHA-256.
    ///
    /// COSE ES256 (-7) specifies ECDSA with SHA-256, but not the curve. When the caller knows that the key uses P-256,
    /// ES256 and fully specified ESP256 (-9) describe the same operation. This classification does not imply that every
    /// cryptographic operation in this crate accepts both identifiers on the wire.
    pub fn is_ecdsa_p256(&self) -> bool {
        matches!(self, Self::Known(iana::Algorithm::ES256 | iana::Algorithm::ESP256))
    }
}

impl From<CoseAlgorithmIdentifier> for i64 {
    fn from(value: CoseAlgorithmIdentifier) -> Self {
        match value {
            CoseAlgorithmIdentifier::Known(identifier) => identifier as i64,
            CoseAlgorithmIdentifier::Unknown(identifier) => identifier,
        }
    }
}

impl From<i64> for CoseAlgorithmIdentifier {
    fn from(value: i64) -> Self {
        iana::Algorithm::from_i64(value)
            .map(Into::into)
            .unwrap_or_else(|| Self::Unknown(value))
    }
}

impl From<iana::Algorithm> for CoseAlgorithmIdentifier {
    fn from(algorithm: iana::Algorithm) -> Self {
        match algorithm {
            iana::Algorithm::ES256 | iana::Algorithm::ESP256 => Self::Known(algorithm),
            algorithm => Self::Unknown(algorithm.to_i64()),
        }
    }
}
