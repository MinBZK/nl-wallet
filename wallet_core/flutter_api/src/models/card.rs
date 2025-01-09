use wallet;
use wallet::Attribute;
use wallet::AttributeValue;
use wallet::Document;
use wallet::DocumentAttributes;
use wallet::DocumentPersistence;
use wallet::GenderAttributeValue;

use super::disclosure::Organization;

pub struct Card {
    pub issuer: Organization,
    pub persistence: CardPersistence,
    pub doc_type: String,
    pub attributes: Vec<CardAttribute>,
}

pub enum CardPersistence {
    InMemory,
    Stored { id: String },
}

pub struct CardAttribute {
    pub key: String,
    pub labels: Vec<LocalizedString>,
    pub value: CardValue,
}

pub enum CardValue {
    String { value: String },
    Boolean { value: bool },
    Date { value: String },
    Gender { value: GenderCardValue },
}

pub enum GenderCardValue {
    Unknown,
    Male,
    Female,
    NotApplicable,
}

impl From<DocumentPersistence> for CardPersistence {
    fn from(value: DocumentPersistence) -> Self {
        match value {
            DocumentPersistence::InMemory => CardPersistence::InMemory,
            DocumentPersistence::Stored(id) => CardPersistence::Stored { id },
        }
    }
}

impl From<GenderAttributeValue> for GenderCardValue {
    fn from(value: GenderAttributeValue) -> Self {
        match value {
            GenderAttributeValue::Unknown => Self::Unknown,
            GenderAttributeValue::Male => Self::Male,
            GenderAttributeValue::Female => Self::Female,
            GenderAttributeValue::NotApplicable => Self::NotApplicable,
        }
    }
}

#[derive(Clone)]
pub struct LocalizedString {
    pub language: String,
    pub value: String,
}

impl From<Document> for Card {
    fn from(value: Document) -> Self {
        Card {
            persistence: value.persistence.into(),
            doc_type: value.doc_type.to_string(),
            attributes: into_card_attributes(value.attributes),
            issuer: value.issuer_registration.organization.into(),
        }
    }
}

impl From<(String, Attribute)> for CardAttribute {
    fn from((key, attribute): (String, Attribute)) -> Self {
        let labels = attribute
            .key_labels
            .into_iter()
            .map(|(language, value)| LocalizedString {
                language: language.to_string(),
                value: value.to_string(),
            })
            .collect();
        let value = attribute.value.into();
        CardAttribute { key, labels, value }
    }
}

impl From<AttributeValue> for CardValue {
    fn from(value: AttributeValue) -> Self {
        match value {
            AttributeValue::String(s) => Self::String { value: s },
            AttributeValue::Boolean(b) => Self::Boolean { value: b },
            AttributeValue::Date(d) => Self::Date {
                value: d.format("%Y-%m-%d").to_string(),
            },
            AttributeValue::Gender(g) => Self::Gender { value: g.into() },
        }
    }
}

pub(crate) fn into_card_attributes(attributes: DocumentAttributes) -> Vec<CardAttribute> {
    attributes
        .into_iter()
        .map(|(key, attribute)| CardAttribute::from((key.to_string(), attribute)))
        .collect()
}
