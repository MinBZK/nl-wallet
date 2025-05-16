use indexmap::IndexMap;
use nutype::nutype;
use serde::Deserialize;
use serde::Serialize;

use http_utils::urls::HttpsUri;

use crate::utils::serialization::TaggedBytes;
use crate::Attributes;
use crate::DataElementIdentifier;
use crate::DataElementValue;
use crate::DocType;
use crate::NameSpace;
use crate::Tdate;

use super::AttestationQualification;

#[nutype(
    derive(Debug, Clone, PartialEq, AsRef, TryFrom, Into, Serialize, Deserialize),
    validate(predicate = |attributes|
        !attributes.is_empty() && !attributes.values().any(|entries| entries.is_empty())
    ),
)]
pub struct UnsignedAttributes(IndexMap<NameSpace, Vec<Entry>>);

/// A collection of data representing a not-yet-signed mdoc.
#[derive(Debug, Clone)]
pub struct UnsignedMdoc {
    pub doc_type: DocType,
    pub valid_from: Tdate,
    pub valid_until: Tdate,
    pub attributes: UnsignedAttributes,

    /// The SAN DNS name or URI of the issuer, as it appears in the issuer's certificate.
    pub issuer_uri: HttpsUri,
    pub attestation_qualification: AttestationQualification,
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

impl From<Attributes> for Vec<Entry> {
    fn from(attributes: Attributes) -> Self {
        attributes
            .into_iter()
            .map(|TaggedBytes(item)| Entry {
                name: item.element_identifier,
                value: item.element_value,
            })
            .collect()
    }
}
