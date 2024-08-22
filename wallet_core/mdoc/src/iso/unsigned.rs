use std::num::NonZeroU8;

use base64::prelude::*;
use ciborium::Value;
use indexmap::IndexMap;
use nutype::nutype;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, FromInto, IfIsHumanReadable};

use crate::{
    utils::serialization::TaggedBytes, Attributes, DataElementIdentifier, DataElementValue, DocType, NameSpace, Tdate,
};

#[nutype(
    derive(Debug, Clone, PartialEq, AsRef, TryFrom, Into, Serialize, Deserialize),
    validate(predicate = |attributes|
        !attributes.is_empty() && !attributes.values().any(|entries| entries.is_empty())
    ),
)]
pub struct UnsignedAttributes(IndexMap<NameSpace, Vec<Entry>>);

/// A not-yet-signed mdoc, presented by the issuer to the holder during issuance, so that the holder can agree
/// or disagree to receive the signed mdoc in the rest of the protocol.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct UnsignedMdoc {
    // ISO 18013-5 calls this `docType` (which in Rust would be `doc_type`), and OpenID4VCI calls this `doctype`.
    // We rename it during serialization to cater to OpenID4VCI, but call it `doc_type` here for consistency
    // with the other structs that have a doc_type field.
    #[serde(rename = "doctype")]
    pub doc_type: DocType,
    pub valid_from: Tdate,
    pub valid_until: Tdate,
    pub attributes: UnsignedAttributes,

    /// The amount of copies of this mdoc that the holder will receive.
    pub copy_count: NonZeroU8,
}

/// An attribute name and value.
///
/// See also [`IssuerSignedItem`](super::IssuerSignedItem), which additionally contains the attribute's `random` and
/// `digestID`.
#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub name: DataElementIdentifier,

    #[serde_as(as = "IfIsHumanReadable<FromInto<JsonCborValue>>")]
    pub value: DataElementValue,
}

impl From<&Attributes> for Vec<Entry> {
    fn from(attrs: &Attributes) -> Self {
        attrs
            .as_ref()
            .iter()
            .map(|TaggedBytes(item)| Entry {
                name: item.element_identifier.clone(),
                value: item.element_value.clone(),
            })
            .collect()
    }
}

fn json_serializable_value(value: Value) -> Value {
    match value {
        Value::Integer(int) => Value::Integer(int),
        Value::Float(float) => Value::Float(float),
        Value::Bool(bool) => Value::Bool(bool),
        Value::Text(text) => Value::Text(text),
        Value::Null => Value::Null,

        Value::Bytes(bytes) => Value::Text(BASE64_STANDARD_NO_PAD.encode(bytes)),
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
    use base64::prelude::*;
    use ciborium::cbor;
    use regex::Regex;
    use serde::Serialize;
    use serde_json::json;
    use serde_with::{serde_as, FromInto};

    use crate::test::data;

    use super::{Engine, JsonCborValue, UnsignedMdoc};

    #[test]
    fn test_unsigned_mdoc_disclosure_count() {
        let unsigned = UnsignedMdoc::from(data::pid_full_name().into_iter().next().unwrap());
        let unsigned_json = serde_json::to_string(&unsigned).unwrap();

        // Replace the `copy_count` in the JSON with invalid values, which should not deserialize.
        let unsigned_json_cc_0 = Regex::new(r#""copy_count":\s*\d+"#)
            .unwrap()
            .replace(&unsigned_json, "\"copy_count\": 0");
        let unsigned_json_cc_256 = Regex::new(r#""copy_count":\s*\d+"#)
            .unwrap()
            .replace(&unsigned_json, "\"copy_count\": 256");

        serde_json::from_str::<UnsignedMdoc>(&unsigned_json_cc_0)
            .expect_err("should not be valid JSON of UnsignedMdoc");
        serde_json::from_str::<UnsignedMdoc>(&unsigned_json_cc_256)
            .expect_err("should not be valid JSON of UnsignedMdoc");

        // As a sanity check, replace the `copy_count` again with a valid value.
        let unsigned_json_cc_100 = Regex::new(r#""copy_count":\s*\d+"#)
            .unwrap()
            .replace(&unsigned_json_cc_0, "\"copy_count\": 100");

        serde_json::from_str::<UnsignedMdoc>(&unsigned_json_cc_100).expect("should be valid JSON of UnsignedMdoc");
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

        let serialized = serde_json::to_string_pretty(&Test { value: testvalue }).unwrap();
        let deserialized: serde_json::Value = serde_json::from_str(&serialized).unwrap();

        let expected = json!({
            "value": {
                "int": 42,
                "float": 2.818281828,
                "text": "Hello, world!",
                "bool": true,
                "null": null,
                "array": ["foo"],
                "map": {"recursive": BASE64_STANDARD_NO_PAD.encode(bytes.clone())},
                "tagged_date": "2020-01-01",
                "bytes": BASE64_STANDARD_NO_PAD.encode(bytes.clone())
            }
        });
        assert_eq!(deserialized, expected);
    }
}
