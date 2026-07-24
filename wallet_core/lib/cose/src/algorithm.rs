use coset::RegisteredLabelWithPrivate;
use coset::iana;
use coset::iana::EnumI64;
use coset::iana::WithPrivateRange;
use serde::Deserialize;
use serde::Serialize;

/// A numeric COSE algorithm identifier suitable for use in protocol messages.
///
/// [`CoseAlgorithmIdentifier::Known`] means that this type explicitly models the registered identifier. It does not
/// imply that every signing or verification operation in this crate supports that algorithm. Identifiers not yet
/// modeled by this type, including registered identifiers, are preserved as [`CoseAlgorithmIdentifier::Unknown`].
#[derive(Debug, Clone, Copy, Eq, Serialize, Deserialize)]
#[serde(from = "i64", into = "i64")]
pub enum CoseAlgorithmIdentifier {
    Known(KnownCoseAlgorithmIdentifier),
    Unknown(i64),
}

impl CoseAlgorithmIdentifier {
    /// Whether this identifier is compatible with ECDSA using P-256 and SHA-256.
    ///
    /// COSE ES256 (-7) specifies ECDSA with SHA-256, but not the curve. When the caller knows that the key uses P-256,
    /// ES256 and fully specified ESP256 (-9) describe the same operation. This classification does not imply that every
    /// cryptographic operation in this crate accepts both identifiers on the wire.
    pub fn is_ecdsa_p256(&self) -> bool {
        matches!(
            self,
            Self::Known(KnownCoseAlgorithmIdentifier::Es256 | KnownCoseAlgorithmIdentifier::Esp256)
        )
    }
}

impl From<KnownCoseAlgorithmIdentifier> for CoseAlgorithmIdentifier {
    fn from(value: KnownCoseAlgorithmIdentifier) -> Self {
        Self::Known(value)
    }
}

/// Registered COSE algorithm identifiers explicitly recognized by [`CoseAlgorithmIdentifier`].
///
/// This list is deliberately not exhaustive, and recognition does not imply support by every cryptographic operation
/// in this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::FromRepr)]
#[repr(i64)]
pub enum KnownCoseAlgorithmIdentifier {
    Es256 = -7,
    Esp256 = -9,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CoseAlgorithmIdentifierConversionError {
    #[error("text COSE algorithm identifier cannot be represented as a numeric identifier: {0}")]
    TextIdentifier(String),
    #[error("numeric COSE algorithm identifier is not represented by this version of coset: {0}")]
    NotRepresentedByCoset(i64),
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
        match KnownCoseAlgorithmIdentifier::from_repr(value) {
            Some(identifier) => Self::Known(identifier),
            None => Self::Unknown(value),
        }
    }
}

impl PartialEq for CoseAlgorithmIdentifier {
    fn eq(&self, other: &Self) -> bool {
        i64::from(*self) == i64::from(*other)
    }
}

impl TryFrom<coset::Algorithm> for CoseAlgorithmIdentifier {
    type Error = CoseAlgorithmIdentifierConversionError;

    fn try_from(value: coset::Algorithm) -> Result<Self, Self::Error> {
        match value {
            RegisteredLabelWithPrivate::Assigned(identifier) => Ok(identifier.to_i64().into()),
            RegisteredLabelWithPrivate::PrivateUse(identifier) => Ok(identifier.into()),
            RegisteredLabelWithPrivate::Text(identifier) => {
                Err(CoseAlgorithmIdentifierConversionError::TextIdentifier(identifier))
            }
        }
    }
}

impl TryFrom<CoseAlgorithmIdentifier> for coset::Algorithm {
    type Error = CoseAlgorithmIdentifierConversionError;

    fn try_from(value: CoseAlgorithmIdentifier) -> Result<Self, Self::Error> {
        let identifier = i64::from(value);

        if let Some(algorithm) = iana::Algorithm::from_i64(identifier) {
            Ok(RegisteredLabelWithPrivate::Assigned(algorithm))
        } else if iana::Algorithm::is_private(identifier) {
            Ok(RegisteredLabelWithPrivate::PrivateUse(identifier))
        } else {
            Err(CoseAlgorithmIdentifierConversionError::NotRepresentedByCoset(
                identifier,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assigned_algorithm_roundtrips() {
        let algorithm = coset::Algorithm::Assigned(iana::Algorithm::ES256);
        let identifier = CoseAlgorithmIdentifier::try_from(algorithm.clone()).unwrap();

        assert_eq!(
            identifier,
            CoseAlgorithmIdentifier::Known(KnownCoseAlgorithmIdentifier::Es256)
        );
        assert_eq!(coset::Algorithm::try_from(identifier).unwrap(), algorithm);
    }

    #[test]
    fn unmodeled_assigned_and_private_use_algorithms_roundtrip() {
        for algorithm in [
            coset::Algorithm::Assigned(iana::Algorithm::EdDSA),
            coset::Algorithm::PrivateUse(-65_537),
        ] {
            let identifier = CoseAlgorithmIdentifier::try_from(algorithm.clone()).unwrap();

            assert!(matches!(identifier, CoseAlgorithmIdentifier::Unknown(_)));
            assert_eq!(coset::Algorithm::try_from(identifier).unwrap(), algorithm);
        }
    }

    #[test]
    fn conversions_reject_unrepresentable_identifiers() {
        assert_eq!(
            CoseAlgorithmIdentifier::try_from(coset::Algorithm::Text("example".to_owned())),
            Err(CoseAlgorithmIdentifierConversionError::TextIdentifier(
                "example".to_owned()
            ))
        );
        assert_eq!(
            coset::Algorithm::try_from(CoseAlgorithmIdentifier::Known(KnownCoseAlgorithmIdentifier::Esp256)),
            Err(CoseAlgorithmIdentifierConversionError::NotRepresentedByCoset(-9))
        );
    }
}
