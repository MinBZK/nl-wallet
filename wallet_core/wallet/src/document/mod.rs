mod mapping;
mod mdoc;

use std::collections::HashMap;

use chrono::NaiveDate;
use indexmap::IndexMap;

use nl_wallet_mdoc::utils::issuer_auth::IssuerRegistration;

#[cfg(feature = "snapshot_test")]
use serde::Serialize;

pub use mdoc::AttributeValueType;
pub use mdoc::DisclosureType;
pub use mdoc::DocumentMdocError;

#[cfg(test)]
pub use mdoc::tests::create_full_unsigned_address_mdoc;
#[cfg(test)]
pub use mdoc::tests::create_full_unsigned_pid_mdoc;
#[cfg(test)]
pub use mdoc::tests::create_minimal_unsigned_address_mdoc;
#[cfg(test)]
pub use mdoc::tests::create_minimal_unsigned_pid_mdoc;

pub const PID_DOCTYPE: &str = "com.example.pid";
const ADDRESS_DOCTYPE: &str = "com.example.address";

pub type DocumentType = &'static str;
pub type AttributeKey = &'static str;
pub type DocumentAttributes = IndexMap<AttributeKey, Attribute>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub persistence: DocumentPersistence,
    pub doc_type: DocumentType,
    pub attributes: DocumentAttributes,
    pub issuer_registration: IssuerRegistration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentPersistence {
    InMemory,
    Stored(String),
}

pub type AttributeLabelLanguage = &'static str;
pub type AttributeLabel = &'static str;
pub type AttributeLabels = HashMap<AttributeLabelLanguage, AttributeLabel>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    pub key_labels: AttributeLabels,
    pub value: AttributeValue,
}

#[cfg_attr(feature = "snapshot_test", derive(Serialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttributeValue {
    String(String),
    Boolean(bool),
    Date(NaiveDate),
    Gender(GenderAttributeValue),
}

#[cfg_attr(feature = "snapshot_test", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenderAttributeValue {
    Unknown,
    Male,
    Female,
    NotApplicable,
}

/// A lower priority means that this `doc_type` should be displayed above others.
fn doc_type_priority(doc_type: &str) -> usize {
    match doc_type {
        PID_DOCTYPE => 0,
        ADDRESS_DOCTYPE => 1,
        _ => usize::MAX,
    }
}

impl Document {
    /// A lower priority means that this [`Document`] should be displayed above others.
    pub fn priority(&self) -> usize {
        doc_type_priority(self.doc_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_document(doc_type: &'static str) -> Document {
        Document {
            persistence: DocumentPersistence::InMemory,
            doc_type,
            attributes: Default::default(),
            issuer_registration: IssuerRegistration::new_mock(),
        }
    }

    #[test]
    fn test_document_compare_inverse_priority() {
        let mut documents = vec![
            empty_document("foo"),
            empty_document(ADDRESS_DOCTYPE),
            empty_document("bar"),
            empty_document(PID_DOCTYPE),
            empty_document("baz"),
        ];

        documents.sort_by_key(Document::priority);
        let doc_types = documents
            .into_iter()
            .map(|document| document.doc_type)
            .collect::<Vec<_>>();

        assert_eq!(doc_types, vec![PID_DOCTYPE, ADDRESS_DOCTYPE, "foo", "bar", "baz"]);
    }
}
