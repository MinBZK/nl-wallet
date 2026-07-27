use ciborium::value::Value;
use coset::AsCborValue;
use error_category::ErrorCategory;
use serde::Deserialize;
use serde::Serialize;
use serde::de;
use serde::de::DeserializeOwned;
use serde::de::Deserializer;
use serde::ser;
use serde::ser::Serializer;

use crate::TypedCose;

#[derive(thiserror::Error, Debug, ErrorCategory)]
#[category(pd)]
pub enum CborError {
    #[error("deserialization failed: {0}")]
    Deserialization(#[from] ciborium::de::Error<std::io::Error>),
    #[error("serialization failed: {0}")]
    Serialization(#[from] ciborium::ser::Error<std::io::Error>),
}

pub(crate) fn cbor_deserialize<T: DeserializeOwned, R: std::io::Read>(reader: R) -> Result<T, CborError> {
    Ok(ciborium::de::from_reader(reader)?)
}

pub(crate) fn cbor_serialize<T: Serialize>(value: &T) -> Result<Vec<u8>, CborError> {
    let mut bytes = Vec::new();
    ciborium::ser::into_writer(value, &mut bytes)?;
    Ok(bytes)
}

pub(crate) fn serialize_as_cbor_value<T: Clone + AsCborValue, S: Serializer>(
    value: &T,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    value
        .clone()
        .to_cbor_value()
        .map_err(ser::Error::custom)?
        .serialize(serializer)
}

pub(crate) fn deserialize_as_cbor_value<'de, T: AsCborValue, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<T, D::Error> {
    T::from_cbor_value(Value::deserialize(deserializer)?).map_err(de::Error::custom)
}

impl<C, T> Serialize for TypedCose<C, T>
where
    C: AsCborValue + Clone,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_as_cbor_value(self.as_ref(), serializer)
    }
}

impl<'de, C, T> Deserialize<'de> for TypedCose<C, T>
where
    C: AsCborValue,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(deserialize_as_cbor_value::<C, _>(deserializer)?.into())
    }
}
