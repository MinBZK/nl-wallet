use ciborium::value::Value;
use coset::AsCborValue;
use coset::CoseKeyBuilder;
use coset::Label;
use coset::iana;
use derive_more::AsRef;
use derive_more::From;
use derive_more::Into;
use error_category::ErrorCategory;
use p256::ecdsa::VerifyingKey;
use serde::Deserialize;
use serde::Serialize;
use serde::de::Deserializer;
use serde::ser::Serializer;

use crate::serialization::deserialize_as_cbor_value;
use crate::serialization::serialize_as_cbor_value;

const P256_COORDINATE_LENGTH: usize = 32;

/// A serde-compatible wrapper around [`coset::CoseKey`].
#[derive(Debug, Clone, PartialEq, AsRef, From, Into)]
pub struct CoseKey(coset::CoseKey);

impl AsCborValue for CoseKey {
    fn from_cbor_value(value: Value) -> coset::Result<Self> {
        Ok(coset::CoseKey::from_cbor_value(value)?.into())
    }

    fn to_cbor_value(self) -> coset::Result<Value> {
        let key: coset::CoseKey = self.into();
        key.to_cbor_value()
    }
}

impl Serialize for CoseKey {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_as_cbor_value(self.as_ref(), serializer)
    }
}

impl<'de> Deserialize<'de> for CoseKey {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(deserialize_as_cbor_value::<coset::CoseKey, _>(deserializer)?.into())
    }
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(pd)]
pub enum CoseKeyConversionError {
    #[error("missing coordinate in conversion to a P-256 public key")]
    #[category(critical)]
    MissingCoordinate,
    #[error("unsupported COSE key type: expected EC2")]
    #[category(critical)]
    UnsupportedKeyType,
    #[error("missing curve in EC2 COSE key")]
    #[category(critical)]
    MissingCurve,
    #[error("unsupported COSE EC2 curve: expected P-256")]
    #[category(critical)]
    UnsupportedCurve,
    #[error("COSE key coordinate must be a byte string")]
    #[category(critical)]
    InvalidCoordinate,
    #[error("invalid COSE P-256 key coordinate length: expected {expected} bytes, got {actual}")]
    #[category(critical)]
    InvalidCoordinateLength { expected: usize, actual: usize },
    #[error("failed to construct P-256 verifying key: {0}")]
    VerifyingKeyConstruction(#[from] p256::ecdsa::Error),
}

impl TryFrom<&VerifyingKey> for CoseKey {
    type Error = CoseKeyConversionError;

    fn try_from(key: &VerifyingKey) -> Result<Self, Self::Error> {
        let encoded_point = key.to_encoded_point(false);
        let x = encoded_point
            .x()
            .ok_or(CoseKeyConversionError::MissingCoordinate)?
            .to_vec();
        let y = encoded_point
            .y()
            .ok_or(CoseKeyConversionError::MissingCoordinate)?
            .to_vec();

        Ok(CoseKeyBuilder::new_ec2_pub_key(iana::EllipticCurve::P_256, x, y)
            .build()
            .into())
    }
}

impl TryFrom<&CoseKey> for VerifyingKey {
    type Error = CoseKeyConversionError;

    fn try_from(key: &CoseKey) -> Result<Self, Self::Error> {
        let key = key.as_ref();
        if key.kty != coset::RegisteredLabel::Assigned(iana::KeyType::EC2) {
            return Err(CoseKeyConversionError::UnsupportedKeyType);
        }

        let curve = parameter(key, iana::Ec2KeyParameter::Crv).ok_or(CoseKeyConversionError::MissingCurve)?;
        if curve != &Value::from(iana::EllipticCurve::P_256 as u64) {
            return Err(CoseKeyConversionError::UnsupportedCurve);
        }

        let x = coordinate(key, iana::Ec2KeyParameter::X)?;
        let y = coordinate(key, iana::Ec2KeyParameter::Y)?;
        let mut encoded_point = Vec::with_capacity(1 + x.len() + y.len());
        encoded_point.push(0x04);
        encoded_point.extend_from_slice(x);
        encoded_point.extend_from_slice(y);

        VerifyingKey::from_sec1_bytes(&encoded_point).map_err(CoseKeyConversionError::VerifyingKeyConstruction)
    }
}

fn parameter(key: &coset::CoseKey, parameter: iana::Ec2KeyParameter) -> Option<&Value> {
    key.params
        .iter()
        .find(|(label, _)| label == &Label::Int(parameter as i64))
        .map(|(_, value)| value)
}

fn coordinate(key: &coset::CoseKey, parameter_name: iana::Ec2KeyParameter) -> Result<&[u8], CoseKeyConversionError> {
    let coordinate = parameter(key, parameter_name)
        .ok_or(CoseKeyConversionError::MissingCoordinate)?
        .as_bytes()
        .map(Vec::as_slice)
        .ok_or(CoseKeyConversionError::InvalidCoordinate)?;

    if coordinate.len() != P256_COORDINATE_LENGTH {
        return Err(CoseKeyConversionError::InvalidCoordinateLength {
            expected: P256_COORDINATE_LENGTH,
            actual: coordinate.len(),
        });
    }

    Ok(coordinate)
}

#[cfg(test)]
mod tests {
    use coset::CoseKeyBuilder;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;

    use super::*;
    use crate::serialization::cbor_deserialize;
    use crate::serialization::cbor_serialize;

    #[test]
    fn p256_key_and_cbor_round_trip() {
        let signing_key = SigningKey::random(&mut OsRng);
        let cose_key = CoseKey::try_from(signing_key.verifying_key()).unwrap();

        let encoded = cbor_serialize(&cose_key).unwrap();
        let decoded: CoseKey = cbor_deserialize(encoded.as_slice()).unwrap();
        let verifying_key = VerifyingKey::try_from(&decoded).unwrap();

        assert_eq!(decoded, cose_key);
        assert_eq!(&verifying_key, signing_key.verifying_key());
    }

    #[test]
    fn p256_conversion_does_not_depend_on_parameter_order() {
        let signing_key = SigningKey::random(&mut OsRng);
        let mut cose_key = CoseKey::try_from(signing_key.verifying_key()).unwrap();
        cose_key.0.params.reverse();

        assert_eq!(VerifyingKey::try_from(&cose_key).unwrap(), *signing_key.verifying_key());
    }

    #[test]
    fn p256_conversion_rejects_coordinates_with_invalid_lengths() {
        let signing_key = SigningKey::random(&mut OsRng);
        let encoded_point = signing_key.verifying_key().to_encoded_point(false);
        let x = encoded_point.x().unwrap().as_slice();
        let y = encoded_point.y().unwrap().as_slice();

        for (x, y, invalid_length) in [
            (x[..31].to_vec(), [&x[31..], y].concat(), 31),
            ([x, &y[..1]].concat(), y[1..].to_vec(), 33),
            (x.to_vec(), y[..31].to_vec(), 31),
            (x.to_vec(), [y, &[0]].concat(), 33),
        ] {
            let cose_key: CoseKey = CoseKeyBuilder::new_ec2_pub_key(iana::EllipticCurve::P_256, x, y)
                .build()
                .into();

            assert!(matches!(
                VerifyingKey::try_from(&cose_key),
                Err(CoseKeyConversionError::InvalidCoordinateLength {
                    expected: P256_COORDINATE_LENGTH,
                    actual,
                }) if actual == invalid_length
            ));
        }
    }

    #[test]
    fn non_ec2_key_is_rejected() {
        let cose_key: CoseKey = CoseKeyBuilder::new_symmetric_key(vec![0; 32]).build().into();

        assert!(matches!(
            VerifyingKey::try_from(&cose_key),
            Err(CoseKeyConversionError::UnsupportedKeyType)
        ));
    }
}
