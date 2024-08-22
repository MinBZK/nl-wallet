use std::num::NonZeroU8;

use indexmap::IndexMap;
use nutype::nutype;
use serde::{Deserialize, Serialize};

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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub name: DataElementIdentifier,
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

#[cfg(test)]
mod tests {
    use regex::Regex;

    use crate::test::data;

    use super::UnsignedMdoc;

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
}
