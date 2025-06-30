//! CBOR serialization: wrapper types that modify serialization and specialized (de)serialization implementations.

use base64::prelude::*;
use ciborium::tag;
use ciborium::value::Value;
use core::fmt::Debug;
use coset::AsCborValue;
use indexmap::IndexMap;
use nutype::nutype;
use serde::de;
use serde::de::DeserializeOwned;
use serde::de::Deserializer;
use serde::ser;
use serde::ser::Serializer;
use serde::Deserialize;
use serde::Serialize;
use serde_aux::serde_introspection::serde_introspect;
use serde_bytes::ByteBuf;
use std::borrow::Cow;
use url::Url;

use error_category::ErrorCategory;

use crate::iso::*;
use crate::utils::cose::CoseKey;
use crate::utils::cose::MdocCose;
const CBOR_TAG_ENC_CBOR: u64 = 24;

#[derive(thiserror::Error, Debug, ErrorCategory)]
#[category(pd)] // might leak PII
pub enum CborError {
    #[error("deserialization failed: {0}")]
    Deserialization(#[from] ciborium::de::Error<std::io::Error>),
    #[error("serialization failed: {0}")]
    Serialization(#[from] ciborium::ser::Error<std::io::Error>),
    #[error("encoding or decoding CBOR value failed: {0}")]
    Value(#[from] ciborium::value::Error),
}

/// Wrapper for [`ciborium::de::from_reader`] returning our own error type.
pub fn cbor_deserialize<T: DeserializeOwned, R: std::io::Read>(reader: R) -> Result<T, CborError> {
    let deserialized = ciborium::de::from_reader(reader)?;
    Ok(deserialized)
}

/// Wrapper for [`ciborium::ser::into_writer`] returning our own error type.
pub fn cbor_serialize<T: Serialize>(o: &T) -> Result<Vec<u8>, CborError> {
    let mut bts: Vec<u8> = Vec::new();
    ciborium::ser::into_writer(o, &mut bts)?;
    Ok(bts)
}

/// Wrapper for `T` that serializes as `#6.24(bstr .cbor T)`: a tagged CBOR byte sequence, in which the byte sequence
/// is the CBOR-serialization of `T`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaggedBytes<T>(pub T);
impl<T> From<T> for TaggedBytes<T> {
    fn from(val: T) -> Self {
        TaggedBytes(val)
    }
}

impl<T> Default for TaggedBytes<T>
where
    T: Default,
{
    fn default() -> Self {
        TaggedBytes(T::default())
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
    T: DeserializeOwned,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let buf = tag::Required::<ByteBuf, CBOR_TAG_ENC_CBOR>::deserialize(deserializer)?.0;
        let result = TaggedBytes(cbor_deserialize(buf.as_ref()).map_err(de::Error::custom)?);
        Ok(result)
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
        let key = deserialize_as_cbor_value::<coset::CoseKey, _>(deserializer)?.into();
        Ok(key)
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
        let cose = deserialize_as_cbor_value::<C, _>(deserializer)?.into();
        Ok(cose)
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
pub struct CborIntMap<T>(pub T);
impl<T> From<T> for CborIntMap<T> {
    fn from(val: T) -> Self {
        CborIntMap(val)
    }
}

impl<T> Serialize for CborSeq<T>
where
    T: Serialize,
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
    T: Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let values = Vec::<Value>::deserialize(deserializer)?;

        Value::Map(
            serde_introspect::<T>()
                .iter()
                .zip(values)
                .map(|v| (Value::Text(v.0.to_string()), v.1))
                .collect(),
        )
        .deserialized()
        .map(|v| CborSeq(v))
        .map_err(|e| de::Error::custom(format!("CborSeq::deserialize failed: {e}")))
    }
}

impl<'de, T> Serialize for CborIntMap<T>
where
    T: Serialize + Deserialize<'de>,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let field_name_indices: IndexMap<String, Value> = serde_introspect::<T>()
            .iter()
            .enumerate()
            .map(|(index, field_name)| (field_name.to_string(), Value::Integer(index.into())))
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
impl<'de, T> Deserialize<'de> for CborIntMap<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let ordered_map = match Value::deserialize(deserializer)? {
            Value::Map(v) => Ok(v),
            _ => Err(de::Error::custom("CborIntMap::deserialize failed: not a map")),
        }?;

        let fieldnames = serde_introspect::<T>();
        let with_fieldnames: Vec<(Value, Value)> = ordered_map
            .into_iter()
            .map(|(index, val)| {
                let index: usize = index
                    .as_integer()
                    .ok_or(de::Error::custom(
                        "CborIntMap::deserialize failed: key was not an integer",
                    ))?
                    .try_into()
                    .map_err(|e| de::Error::custom(format!("CborIntMap::deserialize failed: {e}")))?;
                let fieldname = fieldnames
                    .get(index)
                    .ok_or(de::Error::custom("CborIntMap::deserialize failed: index out of bounds"))?;
                Ok((Value::Text(fieldname.to_string()), val))
            })
            .collect::<Result<_, _>>()?;

        Value::Map(with_fieldnames)
            .deserialized()
            .map(|v| CborIntMap(v))
            .map_err(|e| de::Error::custom(format!("CborIntMap::deserialize failed: {e}")))
    }
}

// We can't derive `Deserialize` with the `untagged` Serde enum deserializer, because unfortunately it is not able to
// distinguish between the `NfcHandover` and `Oid4vpHandover` variants.
// For the other direction (serializing), however, the `untagged` enum serializer is used and works fine.
// Note that this implementation is only ever used to deserialize the examples from the spec in `examples.rs`.
// For each variant a unit test is included to check that serializing and deserializing agree with each other.
#[cfg(any(test, feature = "examples"))]
impl<'de> Deserialize<'de> for Handover {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let val = Value::deserialize(deserializer)?;
        match val {
            Value::Null => Ok(Handover::QrHandover),
            Value::Array(ref bts_vec) => match bts_vec.len() {
                1 | 2 => Ok(Handover::NfcHandover(val.deserialized().map_err(de::Error::custom)?)),
                3 => Ok(Handover::Oid4vpHandover(val.deserialized().map_err(de::Error::custom)?)),
                _ => Err(de::Error::custom("CBOR array had unexpected length")),
            },
            _ => Err(de::Error::custom("CBOR value of unexpected type")),
        }
    }
}

/// Wrapper around `T`, representing a fixed constant. `T` which must implement `RequiredValueTrait`.
/// Implements serde (de)serialization as follows:
/// * During serialization, always serializes to `T::required_value()`.
/// * During deserialization, accepts only `T::required_value()`.
#[derive(Debug, Clone)]
pub struct RequiredValue<T: RequiredValueTrait>(pub T::Type);

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
            let val = RequiredValue::<T>(T::REQUIRED_VALUE);
            Ok(val)
        } else {
            let err = de::Error::custom(format!("value was {:?}, expected {:?}", found, T::REQUIRED_VALUE));
            Err(err)
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
    // RequiredValue<DeviceAuthenticationString> would have to be able to deserialize into &'static str which is
    // impossible. Also can't use String because those can't be constructed compiletime. So we use Cow which sits in
    // between.
    type Type = Cow<'static, str>;
    const REQUIRED_VALUE: Cow<'static, str> = Cow::Borrowed("DeviceAuthentication");
}

#[derive(Debug, Clone)]
pub struct ReaderAuthenticationString;
impl RequiredValueTrait for ReaderAuthenticationString {
    type Type = Cow<'static, str>;
    const REQUIRED_VALUE: Cow<'static, str> = Cow::Borrowed("ReaderAuthentication");
}

#[derive(Serialize, Deserialize)]
struct OriginInfoTypeSerialized {
    #[serde(rename = "type")]
    typ: u64,
    #[serde(rename = "Details")] // This is capitalized in the standard for unknown reasons
    details: Value,
}

#[derive(Serialize, Deserialize)]
struct OriginInfoWebsiteDetails {
    #[serde(rename = "ReferrerUrl")]
    referrer_url: Url,
}

// Don't (de)serialize the CBOR tag when we serialize to JSON
impl Serialize for Tdate {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            self.0 .0.serialize(serializer)
        } else {
            self.0.serialize(serializer)
        }
    }
}
impl<'de> Deserialize<'de> for Tdate {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            Ok(Tdate(tag::Required::<String, 0>(String::deserialize(deserializer)?)))
        } else {
            Ok(Tdate(tag::Required::<String, 0>::deserialize(deserializer)?))
        }
    }
}

/// Wrapper type that (de)serializes to/from URL-safe-no-pad Base64 containing the CBOR-serialized value.
#[derive(Clone, Debug)]
pub struct CborBase64<T>(pub T);

impl<T> From<T> for CborBase64<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T: Serialize> Serialize for CborBase64<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let bts = cbor_serialize(&self.0).map_err(serde::ser::Error::custom)?;
        BASE64_URL_SAFE_NO_PAD.encode(bts).serialize(serializer)
    }
}

impl<'de, T: DeserializeOwned> Deserialize<'de> for CborBase64<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bts = BASE64_URL_SAFE_NO_PAD
            .decode(String::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)?;
        let val: T = cbor_deserialize(bts.as_slice()).map_err(serde::de::Error::custom)?;
        Ok(CborBase64(val))
    }
}

fn json_serializable_value(value: Value) -> Value {
    match value {
        Value::Integer(int) => Value::Integer(int),
        Value::Float(float) => Value::Float(float),
        Value::Bool(bool) => Value::Bool(bool),
        Value::Text(text) => Value::Text(text),
        Value::Null => Value::Null,

        Value::Bytes(bytes) => Value::Text(BASE64_STANDARD.encode(bytes)),
        Value::Tag(_, val) => json_serializable_value(*val),
        Value::Array(arr) => Value::Array(arr.into_iter().map(json_serializable_value).collect()),
        Value::Map(map) => Value::Map(
            map.into_iter()
                .map(|(key, val)| (json_serializable_value(key), json_serializable_value(val)))
                .collect(),
        ),

        // Value is a non-exhaustive enum
        _ => panic!("unknown CBOR value type"),
    }
}

/// A newtype around [`ciborium::Value`] that, when used during serialization, converts CBOR-types
/// that have no JSON equivalent (tagged values, byte sequences) to something more natural in JSON.
#[nutype(
    sanitize(with = json_serializable_value),
    derive(Debug, Clone, From, Into, Serialize, Deserialize),
)]
pub struct JsonCborValue(Value);

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use ciborium::cbor;
    use ciborium::value::Value::Array;
    use ciborium::value::Value::Bytes;
    use ciborium::value::Value::Null;
    use ciborium::value::Value::Text;
    use hex_literal::hex;
    use serde_json::json;
    use serde_with::serde_as;
    use serde_with::FromInto;

    use crate::examples::Example;

    use super::*;

    #[test]
    fn tagged_bytes() {
        let original: TaggedBytes<Vec<u8>> = vec![0, 1, 42].into();
        let encoded = cbor_serialize(&original).unwrap();
        assert_eq!(encoded, hex!("D81845830001182A"));

        let decoded: TaggedBytes<Vec<u8>> = cbor_deserialize(encoded.as_slice()).unwrap();
        assert_eq!(original.0, decoded.0);
    }

    // For each of the `Handover` variants, we manually construct the CBOR structure as defined by the specs
    // (ISO 18013-5 and OpenID4VP), and check that (1) this correctly deserializes to the expected
    // variant and (2) serializing it back yields identical CBOR. This tests not only that the manual Deserialize
    // implementation agrees with the derived Serialize implementation but also that both of these align with
    // the specs.

    #[test]
    fn test_handover_serialization_qr() {
        // The QR handover is just null.
        let qr_handover: Handover = Null.deserialized().unwrap();
        assert_matches!(qr_handover, Handover::QrHandover);
        assert_eq!(Value::serialized(&qr_handover).unwrap(), Null);
    }

    #[test]
    fn test_handover_serialization_nfc() {
        // The example `DeviceAuthentication` contains an NFC handover, retrieving the example deserializes it.
        let TaggedBytes(CborSeq(device_auth)) = DeviceAuthenticationBytes::example();
        assert_matches!(
            &device_auth.session_transcript.0.handover,
            Handover::NfcHandover(CborSeq(h)) if h.handover_request_message.is_some()
        );

        // Also construct NFC handovers directly and check that they can be (de)serialized.
        let nfc_handovers = [
            Array(vec![Bytes(b"bytes1".to_vec()), Bytes(b"bytes2".to_vec())]),
            Array(vec![Bytes(b"bytes1".to_vec()), Null]),
        ];
        for nfc_handover_cbor in nfc_handovers {
            let nfc_handover: Handover = nfc_handover_cbor.deserialized().unwrap();
            assert_matches!(nfc_handover, Handover::NfcHandover(..));
            assert_eq!(Value::serialized(&nfc_handover).unwrap(), nfc_handover_cbor);
        }
    }

    #[test]
    fn test_handover_serialization_openid4vp() {
        // The OpenID4VP handover is structured as an array of bytes/bytes/text.
        let oid4vp_handover_cbor = Array(vec![
            Bytes(b"bytes1".to_vec()),
            Bytes(b"bytes2".to_vec()),
            Text("nonce".to_string()),
        ]);

        let oid4vp_handover: Handover = oid4vp_handover_cbor.deserialized().unwrap();
        assert_matches!(
            &oid4vp_handover,
            Handover::Oid4vpHandover(CborSeq(OID4VPHandover { nonce, ..})) if nonce == "nonce"
        );

        assert_eq!(Value::serialized(&oid4vp_handover).unwrap(), oid4vp_handover_cbor);
    }

    #[test]
    fn test_json_cbor_serialization() {
        #[serde_as]
        #[derive(Serialize)]
        pub struct Test {
            #[serde_as(as = "FromInto<JsonCborValue>")]
            pub value: ciborium::Value,
        }

        let bytes = hex::decode("DEADBEEF").unwrap();

        let testvalue = cbor!({
            "int" => 42,
            "float" => 2.818281828,
            "text" => "Hello, world!",
            "bool" => true,
            "null" => null,
            "array" => ["foo"],
            "map" => {"recursive" => ciborium::Value::Bytes(bytes.clone())},
            "tagged_date" => ciborium::Value::Tag(1004, ciborium::Value::Text("2020-01-01".to_string()).into()),
            "bytes" => ciborium::Value::Bytes(bytes.clone())
        })
        .unwrap();

        let serialized = serde_json::to_string(&Test { value: testvalue }).unwrap();
        let deserialized: serde_json::Value = serde_json::from_str(&serialized).unwrap();

        let expected = json!({
            "value": {
                "int": 42,
                "float": 2.818281828,
                "text": "Hello, world!",
                "bool": true,
                "null": null,
                "array": ["foo"],
                "map": {"recursive": BASE64_STANDARD.encode(bytes.clone())},
                "tagged_date": "2020-01-01",
                "bytes": BASE64_STANDARD.encode(bytes.clone())
            }
        });
        assert_eq!(deserialized, expected);
    }
}
