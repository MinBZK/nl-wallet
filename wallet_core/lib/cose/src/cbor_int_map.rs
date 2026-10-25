use std::any::type_name;

use ciborium::value::Value;
use indexmap::IndexMap;
use serde::Deserialize;
use serde::Serialize;
use serde::de;
use serde::de::Deserializer;
use serde::ser;
use serde::ser::Serializer;

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
                        let key = key
                            .as_text()
                            .ok_or_else(|| ser::Error::custom("CborIntMap::serialize failed: key was not text"))?;
                        let index = field_name_indices
                            .get(key)
                            .ok_or_else(|| ser::Error::custom("CborIntMap::serialize failed: missing field index"))?;
                        Ok((index.clone(), val.clone()))
                    })
                    .collect::<Result<_, S::Error>>()?,
            )
            .serialize(serializer),
            unexpected_value => Err(ser::Error::custom(format!(
                "CborIntMap::serialize failed for {}: expected map, found {}",
                type_name::<T>(),
                cbor_value_type(&unexpected_value)
            ))),
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

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use ciborium::value::Value::Array;
    use ciborium::value::Value::Bytes;
    use ciborium::value::Value::Map;
    use ciborium::value::Value::Null;
    use ciborium::value::Value::Text;
    use hex_literal::hex;
    use rstest::rstest;

    use super::*;
    use crate::CborError;
    use crate::serialization::cbor_serialize;

    #[derive(Serialize, Deserialize)]
    struct IndexedValue(Value);

    impl CborIndexedFields for IndexedValue {
        fn field_indices() -> &'static [(&'static str, u64)] {
            &[("field", 0), ("optional", 1)]
        }
    }

    #[rstest]
    #[case::text(Text("sample value".to_string()), "text")]
    #[case::array(Array(vec![Text("sample value".to_string())]), "array")]
    fn test_cbor_int_map_serialization_rejects_non_map(#[case] value: Value, #[case] expected_cbor_type: &str) {
        let error = cbor_serialize(&CborIntMap(IndexedValue(value))).unwrap_err();
        let expected_message = format!(
            "CborIntMap::serialize failed for {}: expected map, found {expected_cbor_type}",
            type_name::<IndexedValue>()
        );

        assert_matches!(
            error,
            CborError::Serialization(ciborium::ser::Error::Value(message)) if message == expected_message
        );
    }

    #[rstest]
    #[case::non_text_key(
        Map(vec![(Bytes(b"sample key".to_vec()), Text("sample value".to_string()))]),
        "CborIntMap::serialize failed: key was not text"
    )]
    #[case::missing_field_index(
        Map(vec![(Text("unknown field".to_string()), Text("sample value".to_string()))]),
        "CborIntMap::serialize failed: missing field index"
    )]
    fn test_cbor_int_map_serialization_rejects_invalid_value(#[case] value: Value, #[case] expected_message: &str) {
        let error = cbor_serialize(&CborIntMap(IndexedValue(value))).unwrap_err();

        assert_matches!(
            error,
            CborError::Serialization(ciborium::ser::Error::Value(message)) if message == expected_message
        );
    }

    #[test]
    fn test_cbor_int_map_serialization_omits_null_fields() {
        let value = CborIntMap(IndexedValue(Map(vec![
            (Text("field".to_string()), Text("value".to_string())),
            (Text("optional".to_string()), Null),
        ])));

        assert_eq!(cbor_serialize(&value).unwrap(), hex!("A1006576616C7565"));
    }
}
