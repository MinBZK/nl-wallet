//! CBOR serialization: wrapper types that modify serialization and specialized (de)serialization implementations.
use core::fmt::Debug;
use std::any::type_name;
use std::borrow::Cow;

use base64::prelude::*;
use ciborium::tag;
use ciborium::value::Value;
use error_category::ErrorCategory;
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

use crate::iso::*;

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

fn cbor_value_type(value: &Value) -> &'static str {
    match value {
        Value::Integer(_) => "integer",
        Value::Bytes(_) => "byte string",
        Value::Float(_) => "float",
        Value::Text(_) => "text",
        Value::Bool(_) => "boolean",
        Value::Null => "null",
        Value::Tag(_, _) => "tag",
        Value::Array(_) => "array",
        Value::Map(_) => "map",
        _ => "unknown",
    }
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
            unexpected_value => Err(ser::Error::custom(format!(
                "CborSeq::serialize failed for {}: expected map, found {}",
                type_name::<T>(),
                cbor_value_type(&unexpected_value)
            ))),
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

/// Wrapper around `T`, representing a fixed constant. `T` which must implement `RequiredValueTrait`.
/// Implements serde (de)serialization as follows:
/// * During serialization, always serializes to `T::required_value()`.
/// * During deserialization, accepts only `T::required_value()`.
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    use std::assert_matches;

    use ciborium::value::Value::Array;
    use ciborium::value::Value::Bytes;
    use ciborium::value::Value::Null;
    use ciborium::value::Value::Text;
    use hex_literal::hex;
    use rstest::rstest;

    use super::*;
    use crate::examples::Example;

    #[rstest]
    #[case::text(Text("sample value".to_string()), "text")]
    #[case::array(Array(vec![Text("sample value".to_string())]), "array")]
    fn test_cbor_serialization_rejects_non_map(#[case] value: Value, #[case] expected_cbor_type: &str) {
        let error = cbor_serialize(&CborSeq(value)).unwrap_err();
        let expected_message = format!(
            "CborSeq::serialize failed for {}: expected map, found {expected_cbor_type}",
            type_name::<Value>()
        );

        assert_matches!(
            error,
            CborError::Serialization(ciborium::ser::Error::Value(message)) if message == expected_message
        );
    }

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
