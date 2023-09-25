mod mapping;
mod mdoc;

use std::{cmp::Ordering, collections::HashMap};

use chrono::NaiveDate;
use indexmap::IndexMap;

pub use mdoc::{AttributeValueType, DocumentMdocError};

use self::mapping::MDOC_DOC_TYPE_PRIORITY;

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
    pub fn compare_inverse_priority(left: &Self, right: &Self) -> Ordering {
        right.priority().cmp(&left.priority())
    }

    /// A higher priority means that this [`Document`] should be displayed above
    /// others with a lower priority.
    fn priority(&self) -> usize {
        MDOC_DOC_TYPE_PRIORITY.get(&self.doc_type).copied().unwrap_or_default()
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
            empty_document("com.example.address"),
            empty_document("bar"),
            empty_document("com.example.pid"),
            empty_document("baz"),
        ];

        documents.sort_by(Document::compare_inverse_priority);
        let doc_types = documents
            .into_iter()
            .map(|document| document.doc_type)
            .collect::<Vec<_>>();

        assert_eq!(
            doc_types,
            vec!["com.example.pid", "com.example.address", "foo", "bar", "baz"]
        );
    }
}
