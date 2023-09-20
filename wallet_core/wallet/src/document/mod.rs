mod convert;
mod mapping;

use std::collections::HashMap;

use chrono::NaiveDate;
use indexmap::IndexMap;

pub use convert::{AttributeValueType, DocumentMdocError};

pub type DocumentType = &'static str;
pub type AttributeKey = &'static str;

#[derive(Debug, Clone)]
pub struct Document {
    pub id: Option<String>,
    pub doc_type: DocumentType,
    pub attributes: IndexMap<AttributeKey, Attribute>,
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
