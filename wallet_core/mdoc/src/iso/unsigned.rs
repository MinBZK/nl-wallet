use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{Attributes, DataElementIdentifier, DataElementValue, DocType, NameSpace, Tdate};

/// A not-yet-signed mdoc, presented by the issuer to the holder during issuance, so that the holder can agree
/// or disagree to receive the signed mdoc in the rest of the protocol.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct UnsignedMdoc {
    pub doctype: DocType,
    pub valid_from: Tdate,
    pub valid_until: Tdate,
    pub attributes: IndexMap<NameSpace, Vec<Entry>>,

    /// The amount of copies of this mdoc that the holder will receive.
    pub copy_count: u64,
}

/// An attribute name and value.
///
/// See also [`IssuerSignedItem`](super::IssuerSignedItem), which additionally contains the attribute's `random` and
/// `digestID`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Entry {
    pub name: DataElementIdentifier,
    pub value: DataElementValue,
}

impl From<&Attributes> for Vec<Entry> {
    fn from(attrs: &Attributes) -> Self {
        attrs
            .0
            .iter()
            .map(|issuer_signed| Entry {
                name: issuer_signed.0.element_identifier.clone(),
                value: issuer_signed.0.element_value.clone(),
            })
            .collect()
    }
}
