mod mapping;

use std::collections::HashMap;

use indexmap::IndexMap;

type AttributeKey = &'static str;

#[derive(Debug, Clone)]
pub struct Document {
    pub id: Option<String>,
    pub document_type: DocumentType,
    pub attributes: IndexMap<AttributeKey, Attribute>,
}

#[derive(Debug, Clone, Copy)]
pub enum DocumentType {
    Identity,
    ResidenceAddress,
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
}
