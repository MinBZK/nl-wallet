mod mapping;
mod mdoc;

use std::collections::HashMap;

use chrono::NaiveDate;
use indexmap::IndexMap;

pub use mdoc::{AttributeValueType, DocumentMdocError};

const PID_DOCTYPE: &str = "com.example.pid";
const ADDRESS_DOCTYPE: &str = "com.example.address";

pub type DocumentType = &'static str;
pub type AttributeKey = &'static str;

#[derive(Debug, Clone)]
pub struct Document {
    pub persistence: DocumentPersistence,
    pub doc_type: DocumentType,
    pub attributes: IndexMap<AttributeKey, Attribute>,
}

#[derive(Debug, Clone)]
pub enum DocumentPersistence {
    InMemory,
    Stored(String),
}

pub type AttributeLabelLanguage = &'static str;
pub type AttributeLabel = &'static str;

#[derive(Debug, Clone)]
pub struct Attribute {
    pub key_labels: HashMap<AttributeLabelLanguage, AttributeLabel>,
    pub value: AttributeValue,
}

#[derive(Debug, Clone)]
pub enum AttributeValue {
    String(String),
    Boolean(bool),
    Date(NaiveDate),
    Gender(GenderAttributeValue),
}

#[derive(Debug, Clone, Copy)]
pub enum GenderAttributeValue {
    Unknown,
    Male,
    Female,
    NotApplicable,
}

impl Document {
    /// A lower priority means that this [`Document`] should be displayed above others.
    pub fn priority(&self) -> usize {
        match self.doc_type {
            PID_DOCTYPE => 0,
            ADDRESS_DOCTYPE => 1,
            _ => usize::MAX,
        }
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
