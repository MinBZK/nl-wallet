//! CBOR serialization: wrapper types that modify serialization and specialized (de)serialization implementations.

use ciborium::{tag, value::Value};
use core::fmt::Debug;
use coset::AsCborValue;
use indexmap::IndexMap;
use serde::{de, de::Deserializer, ser, ser::Serializer, Deserialize, Serialize};
use serde_bytes::ByteBuf;
use std::borrow::Cow;

use crate::{
    cose::{CoseKey, MdocCose},
    iso::*,
};
use fieldnames::FieldNames;

const CBOR_TAG_ENC_CBOR: u64 = 24;

#[derive(thiserror::Error, Debug)]
pub enum CborError {
    #[error("deserialization failed")]
    Deserialization(#[from] ciborium::de::Error<std::io::Error>),
    #[error("serialization failed")]
    Serialization(#[from] ciborium::ser::Error<std::io::Error>),
}

// Wrapper for [`ciborium::de::from_reader`] returning our own error type.
pub fn cbor_deserialize<'de, T: de::Deserialize<'de>, R: std::io::Read>(reader: R) -> Result<T, CborError> {
    Ok(ciborium::de::from_reader(reader)?)
}

pub fn cbor_serialize<T: Serialize>(o: &T) -> Result<Vec<u8>, CborError> {
    let mut bts: Vec<u8> = Vec::new();
    ciborium::ser::into_writer(o, &mut bts)?;
    Ok(bts)
}

/// Wrapper for `T` that serializes as `#6.24(bstr .cbor T)`: a tagged CBOR byte sequence, in which the byte sequence
/// is the CBOR-serialization of `T`.
#[derive(Debug, Clone)]
pub struct TaggedBytes<T>(pub T);
impl<T> From<T> for TaggedBytes<T> {
    fn from(val: T) -> Self {
        TaggedBytes(val)
    }
}

impl<T> Serialize for TaggedBytes<T>
where
    T: Serialize,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let buf = cbor_serialize(&self.0).map_err(ser::Error::custom)?;
        tag::Required::<ByteBuf, CBOR_TAG_ENC_CBOR>(ByteBuf::from(buf)).serialize(serializer)
    }
}
impl<'de, T> Deserialize<'de> for TaggedBytes<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let buf = tag::Required::<ByteBuf, CBOR_TAG_ENC_CBOR>::deserialize(deserializer)?.0;
        Ok(TaggedBytes(cbor_deserialize(buf.as_ref()).map_err(de::Error::custom)?))
    }
}

fn serialize_as_cbor_value<T: Clone + AsCborValue, S: Serializer>(val: &T, serializer: S) -> Result<S::Ok, S::Error> {
    val.clone()
        .to_cbor_value()
        .map_err(ser::Error::custom)?
        .serialize(serializer)
}

fn deserialize_as_cbor_value<'de, T: AsCborValue, D: Deserializer<'de>>(deserializer: D) -> Result<T, D::Error> {
    T::from_cbor_value(Value::deserialize(deserializer)?).map_err(de::Error::custom)
}

impl Serialize for CoseKey {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_as_cbor_value(&self.0, serializer)
    }
}
impl<'de> Deserialize<'de> for CoseKey {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(deserialize_as_cbor_value::<coset::CoseKey, _>(deserializer)?.into())
    }
}

impl<C, T> Serialize for MdocCose<C, T>
where
    T: Serialize,
    C: AsCborValue + Clone,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_as_cbor_value(&self.0, serializer)
    }
}
impl<'de, C, T> Deserialize<'de> for MdocCose<C, T>
where
    T: Deserialize<'de>,
    C: AsCborValue,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(deserialize_as_cbor_value::<C, _>(deserializer)?.into())
    }
}

/// Wrapper for structs that serializes from/to CBOR sequences (i.e., not including field names).
/// Used to be able to refer by name instead of by an integer to refer to the contents of the
/// data structure.
#[derive(Debug, Clone)]
pub struct CborSeq<T>(pub T);
impl<T> From<T> for CborSeq<T> {
    fn from(val: T) -> Self {
        CborSeq(val)
    }
}

/// Wrapper for structs that serializes from/to CBOR maps that have
/// integers as keys instead of field names.
/// Used to be able to refer by name instead of by an integer to refer to the contents of the
/// data structure.
#[derive(Debug, Clone)]
pub struct CborIntMap<T, const STRING: bool = false>(pub T);
impl<T> From<T> for CborIntMap<T> {
    fn from(val: T) -> Self {
        CborIntMap(val)
    }
}

impl<'de, T> Serialize for CborSeq<T>
where
    T: Serialize + Deserialize<'de>,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match Value::serialized(&self.0).map_err(ser::Error::custom)? {
            Value::Map(map) => map
                .iter()
                .map(|entry| &entry.1)
                .collect::<Vec<&Value>>()
                .serialize(serializer),
            _ => panic!("struct serialization failed"),
        }
    }
}
impl<'de, T> Deserialize<'de> for CborSeq<T>
where
    T: Serialize + Deserialize<'de> + FieldNames,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let values = Vec::<Value>::deserialize(deserializer)?;

        Value::Map(
            T::field_names()
                .iter()
                .zip(values)
                .map(|v| (Value::Text(v.0.into()), v.1))
                .collect(),
        )
        .deserialized()
        .map(|v| CborSeq(v))
        .map_err(|e| de::Error::custom(format!("MapAsCborSeq::deserialize failed: {}", e)))
    }
}

impl<'de, T, const STRING: bool> Serialize for CborIntMap<T, STRING>
where
    T: Serialize + Deserialize<'de> + FieldNames,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let field_name_indices: IndexMap<String, Value> = T::field_names()
            .iter()
            .enumerate()
            .map(|(index, field_name)| {
                (
                    field_name.clone(),
                    if !STRING {
                        Value::Integer(index.into())
                    } else {
                        Value::Text(format!("{}", index))
                    },
                )
            })
            .collect();

        match Value::serialized(&self.0).map_err(ser::Error::custom)? {
            Value::Map(map) => Value::Map(
                map.iter()
                    .filter(|(_, val)| !val.is_null())
                    .map(|(key, val)| {
                        (
                            field_name_indices.get(key.as_text().unwrap()).unwrap().clone(),
                            val.clone(),
                        )
                    })
                    .collect(),
            )
            .serialize(serializer),
            _ => panic!("struct serialization failed"),
        }
    }
}
impl<'de, T, const STRING: bool> Deserialize<'de> for CborIntMap<T, STRING>
where
    T: Serialize + Deserialize<'de> + FieldNames,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let ordered_map = match Value::deserialize(deserializer)? {
            Value::Map(v) => Ok(v),
            _ => Err(de::Error::custom("not a map")),
        }?;

        let zipped: Vec<(Value, Value)> = ordered_map
            .iter()
            .map(|(_, val)| val)
            .zip(T::field_names())
            .map(|(key, val)| (Value::Text(val), key.clone()))
            .collect();

        Value::Map(zipped)
            .deserialized()
            .map(|v| CborIntMap(v))
            .map_err(|e| de::Error::custom(format!("MapAsCborMap::deserialize failed: {e}")))
    }
}

impl Serialize for Handover {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Handover::QRHandover => Value::Null.serialize(serializer),
            Handover::NFCHandover(handover) => match &handover.handover_request_message {
                None => vec![handover.handover_select_message.clone()],
                Some(_) => {
                    vec![
                        handover.handover_select_message.clone(),
                        handover.handover_request_message.clone().unwrap(),
                    ]
                }
            }
            .serialize(serializer),
        }
    }
}
impl<'de> Deserialize<'de> for Handover {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // TODO check: will this correctly deserialize `null`?
        match Option::<Vec<ByteBuf>>::deserialize(deserializer)? {
            None => Ok(Handover::QRHandover),
            Some(bts_vec) => match bts_vec.len() {
                1 => Ok(Handover::NFCHandover(NFCHandover {
                    handover_select_message: bts_vec[0].clone(),
                    handover_request_message: None,
                })),
                2 => Ok(Handover::NFCHandover(NFCHandover {
                    handover_select_message: bts_vec[0].clone(),
                    handover_request_message: Some(bts_vec[1].clone()),
                })),
                _ => Err(de::Error::custom("unexpected amount of byte sequences found")),
            },
        }
    }
}

/// Wrapper around `T`, which must implement `RequiredValueTrait`.
/// Implements serde (de)serialization as follows:
/// * During serialization, always serializes to `T::required_value()`.
/// * During deserialization, accepts only `T::required_value()`.
#[derive(Debug, Clone)]
pub struct RequiredValue<T: RequiredValueTrait>(T::Type);

impl<T: RequiredValueTrait> Default for RequiredValue<T> {
    fn default() -> Self {
        Self(T::REQUIRED_VALUE)
    }
}

pub trait RequiredValueTrait {
    type Type;
    const REQUIRED_VALUE: Self::Type;
}

impl<'de, T> Deserialize<'de> for RequiredValue<T>
where
    T: RequiredValueTrait,
    T::Type: Debug + Deserialize<'de> + PartialEq,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let found = T::Type::deserialize(deserializer)?;
        if found == T::REQUIRED_VALUE {
            Ok(RequiredValue::<T>(T::REQUIRED_VALUE))
        } else {
            Err(de::Error::custom(format!(
                "value was {:?}, expected {:?}",
                found,
                T::REQUIRED_VALUE
            )))
        }
    }
}
impl<T: RequiredValueTrait> Serialize for RequiredValue<T>
where
    T: RequiredValueTrait,
    T::Type: Serialize,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        T::REQUIRED_VALUE.serialize(serializer)
    }
}

#[derive(Debug, Clone)]
pub struct NullCborValue;
impl RequiredValueTrait for NullCborValue {
    type Type = Value;
    const REQUIRED_VALUE: Value = Value::Null;
}

#[derive(Debug, Clone)]
pub struct DeviceAuthenticationString;
impl RequiredValueTrait for DeviceAuthenticationString {
    // We can't use &'static str directly here, because then the deserialization implementation of
    // RequiredValue<DeviceAuthenticationString> would have to be able to deserialize into &'static str which is impossible.
    // Also can't use String because those can't be constructed compiletime. So we use Cow which sits in between.
    type Type = Cow<'static, str>;
    const REQUIRED_VALUE: Cow<'static, str> = Cow::Borrowed("DeviceAuthentication");
}

#[derive(Debug, Clone)]
pub struct ReaderAuthenticationString;
impl RequiredValueTrait for ReaderAuthenticationString {
    type Type = Cow<'static, str>;
    const REQUIRED_VALUE: Cow<'static, str> = Cow::Borrowed("ReaderAuthentication");
}

#[cfg(test)]
mod tests {
    use ciborium::value::Value;
    use hex_literal::hex;
    use serde_bytes::ByteBuf;

    use crate::serialization::*;

    #[test]
    fn tagged_bytes() {
        let original: TaggedBytes<Vec<u8>> = vec![0, 1, 42].into();
        let encoded = cbor_serialize(&original).unwrap();
        assert_eq!(encoded, hex!("D81845830001182A"));

        let decoded: TaggedBytes<Vec<u8>> = cbor_deserialize(encoded.as_slice()).unwrap();
        assert_eq!(original.0, decoded.0);
    }

    #[test]
    fn message_type() {
        // Use `RequestEndSessionMessage` as an example of a message that should have a `messageType` field
        let msg = RequestEndSessionMessage {
            e_session_id: ByteBuf::from("session_id").into(),
        };

        // Explicitly assert CBOR structure of the serialized data
        assert_eq!(
            Value::serialized(&msg).unwrap(),
            Value::Map(vec![
                (
                    Value::Text("messageType".into()),
                    Value::Text(REQUEST_END_SESSION_MSG_TYPE.into()),
                ),
                (Value::Text("eSessionId".into()), Value::Bytes(b"session_id".to_vec())),
            ])
        );
    }
}
