//! CBOR serialization: wrapper types that modify serialization and specialized (de)serialization implementations.
use std::borrow::Cow;

use base64::prelude::*;
use ciborium::tag;
use ciborium::value::Value;
use core::fmt::Debug;
use coset::AsCborValue;
use indexmap::IndexMap;
use serde::Deserialize;
use serde::Serialize;
use serde::de;
use serde::de::DeserializeOwned;
use serde::de::Deserializer;
use serde::ser;
use serde::ser::Serializer;
use serde_aux::serde_introspection::serde_introspect;
use serde_bytes::ByteBuf;
use serde_with::DeserializeAs;
use serde_with::SerializeAs;

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

/// Wrapper that serializes a named-field struct to/from a CBOR array (sequence), dropping all
/// field names from the wire format.
///
/// Normally `serde` serializes struct fields as a map of `{ field_name: value, ... }`. `CborSeq`
/// instead emits only the values in field declaration order, producing a compact CBOR array as
/// required by ISO 18013-5 for some of the structs.
///
/// The idiomatic pattern in this codebase is to define a `...Keyed` struct with named fields for
/// use in Rust code, then expose a type alias wrapping it in `CborSeq` as the canonical
/// (wire-format) type. This keeps field names available for readability while ensuring the
/// correct on-wire encoding.
///
/// - **Serialization**: the struct is encoded as a CBOR array of values in declaration order.
/// - **Deserialization**: the array elements are zipped with the field names (via `serde_introspect`) and deserialized
///   back into `T`.
///
/// # Example
///
/// `SessionTranscript` is a type alias for `CborSeq<SessionTranscriptKeyed>`:
///
/// ```ignore
/// pub struct SessionTranscriptKeyed {       // <-- named fields, used in Rust code
///     pub device_engagement_bytes: Option<DeviceEngagementBytes>,  // position 0
///     pub ereader_key_bytes: Option<ESenderKeyBytes>,              // position 1
///     pub handover: Handover,                                      // position 2
/// }
///
/// pub type SessionTranscript = CborSeq<SessionTranscriptKeyed>;    // <-- wire type
/// ```
///
/// `SessionTranscript` serializes as a CBOR array `[<device_engagement_bytes>, <ereader_key_bytes>, <handover>]`
/// rather than a map `{"device_engagement_bytes": ..., "ereader_key_bytes": ..., "handover": ...}`.
///
/// ```ignore
/// let transcript: SessionTranscript = SessionTranscriptKeyed { .. }.into();
/// let bytes = cbor_serialize(&transcript)?; // encodes as [<val0>, <val1>, <val2>]
/// ```
#[derive(Debug, Clone)]
pub struct CborSeq<T>(pub T);

impl<T> From<T> for CborSeq<T> {
    fn from(val: T) -> Self {
        CborSeq(val)
    }
}

/// Trait that provides a mapping from struct field names to their CBOR integer map keys.
///
/// Implement via `#[derive(CborIndexedFields)]` from `mdoc_derive`. Fields are numbered
/// sequentially starting at 0 by default; `#[cbor_index = N]` on a field resets the counter
/// to `N` and subsequent fields continue from `N + 1`.
pub trait CborIndexedFields {
    fn field_indices() -> &'static [(&'static str, u64)];
}

/// Wrapper that serializes a named-field struct to/from a CBOR map with integer keys.
///
/// Normally `serde` serializes struct fields using their string names as map keys.
/// `CborIntMap` replaces those string keys with each field's zero-based positional index,
/// producing the compact integer-keyed maps required by ISO 18013-5 for some of the structs.
///
/// - **Serialization**: field names are replaced by their declaration-order index (0, 1, 2, ...); fields whose value
///   serializes as CBOR null are omitted.
/// - **Deserialization**: integer keys are mapped back to field names by position before delegating to `T`'s own
///   deserializer.
///
/// # Example
///
/// `DeviceEngagement` is a type alias for `CborIntMap<Engagement>`, where `Engagement` is:
///
/// ```ignore
/// pub struct Engagement {
///     pub version: EngagementVersion,  // serialized as key 0
///     pub security: Option<Security>,  // serialized as key 1
/// }
/// ```
///
/// Wrapping it in `CborIntMap` encodes it as `{0: "1.0", 1: <security>}` on the wire
/// instead of `{"version": "1.0", "security": <security>}`.
///
/// ```ignore
/// let engagement = DeviceEngagement(Engagement { version: EngagementVersion::V1_0, security: None });
/// let bytes = cbor_serialize(&engagement)?; // encodes as {0: "1.0"}
/// ```
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
            e => panic!("struct serialization failed: {e:?}"),
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
    T: Serialize + Deserialize<'de> + CborIndexedFields,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let field_name_indices: IndexMap<String, Value> = T::field_indices()
            .iter()
            .map(|(field_name, index)| (field_name.to_string(), Value::Integer((*index).into())))
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
            e => panic!("struct serialization failed: {e:?}"),
        }
    }
}

impl<'de, T> Deserialize<'de> for CborIntMap<T>
where
    T: Deserialize<'de> + CborIndexedFields,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let ordered_map = match Value::deserialize(deserializer)? {
            Value::Map(v) => Ok(v),
            _ => Err(de::Error::custom("CborIntMap::deserialize failed: not a map")),
        }?;

        let index_to_field: IndexMap<u64, &str> = T::field_indices()
            .iter()
            .map(|(field_name, index)| (*index, *field_name))
            .collect();

        let with_fieldnames: Vec<(Value, Value)> = ordered_map
            .into_iter()
            .map(|(index, val)| {
                let index: u64 = index
                    .as_integer()
                    .ok_or(de::Error::custom(
                        "CborIntMap::deserialize failed: key was not an integer",
                    ))?
                    .try_into()
                    .map_err(|e| de::Error::custom(format!("CborIntMap::deserialize failed: {e}")))?;
                let fieldname = index_to_field.get(&index).ok_or(de::Error::custom(format!(
                    "CborIntMap::deserialize failed: unknown index {index}"
                )))?;
                Ok((Value::Text(fieldname.to_string()), val))
            })
            .collect::<Result<_, _>>()?;

        Value::Map(with_fieldnames)
            .deserialized()
            .map(|v| CborIntMap(v))
            .map_err(|e| de::Error::custom(format!("CborIntMap::deserialize failed: {e}")))
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

#[derive(Debug, Clone)]
pub struct OpenID4VPHandoverString;

impl RequiredValueTrait for OpenID4VPHandoverString {
    type Type = Cow<'static, str>;
    const REQUIRED_VALUE: Cow<'static, str> = Cow::Borrowed("OpenID4VPHandover");
}

// Don't (de)serialize the CBOR tag when we serialize to JSON
impl Serialize for Tdate {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            self.0.0.serialize(serializer)
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

/// Helper type for (de)serializing types to/from a URL-safe-no-pad Base64 string
/// containing the CBOR-serialized value using `serde_with`.
pub struct CborBase64;

impl<T> SerializeAs<T> for CborBase64
where
    T: Serialize,
{
    fn serialize_as<S>(source: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = cbor_serialize(source).map_err(serde::ser::Error::custom)?;
        let base64 = BASE64_URL_SAFE_NO_PAD.encode(bytes).serialize(serializer)?;

        Ok(base64)
    }
}

impl<'de, T> DeserializeAs<'de, T> for CborBase64
where
    T: DeserializeOwned,
{
    fn deserialize_as<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        let base64 = String::deserialize(deserializer)?;
        let bytes = BASE64_URL_SAFE_NO_PAD
            .decode(base64)
            .map_err(serde::de::Error::custom)?;
        let value = cbor_deserialize(bytes.as_slice()).map_err(serde::de::Error::custom)?;

        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use ciborium::value::Value::Array;
    use ciborium::value::Value::Bytes;
    use ciborium::value::Value::Null;
    use ciborium::value::Value::Text;
    use hex_literal::hex;

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
    // variant and (2) serializing it back yields identical CBOR. This tests not only that the derived Deserialize
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
        let oid4vp_handover_cbor = Array(vec![Text("OpenID4VPHandover".to_string()), Bytes(b"bytes".to_vec())]);

        let oid4vp_handover: Handover = oid4vp_handover_cbor.deserialized().unwrap();
        assert_matches!(
            &oid4vp_handover,
            Handover::Oid4vpHandover(CborSeq(OID4VPHandover { info_hash, ..})) if info_hash == b"bytes"
        );

        assert_eq!(Value::serialized(&oid4vp_handover).unwrap(), oid4vp_handover_cbor);
    }
}
