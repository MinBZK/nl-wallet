use wallet::Document;
use wallet::DocumentPersistence;

use crate::models::disclosure::Organization;

pub struct Attestation {
    pub identity: AttestationIdentity,
    pub attestation_type: String,
    pub display_metadata: Vec<DisplayMetadata>,
    pub issuer: Organization,
    pub attributes: Vec<AttestationAttribute>,
}

impl From<wallet::Attestation> for Attestation {
    fn from(value: wallet::Attestation) -> Self {
        Self {
            identity: value.identity.into(),
            attestation_type: value.attestation_type,
            display_metadata: value.display_metadata.into_iter().map(DisplayMetadata::from).collect(),
            issuer: value.issuer.into(),
            attributes: value.attributes.into_iter().map(AttestationAttribute::from).collect(),
        }
    }
}

impl From<Document> for Attestation {
    fn from(value: Document) -> Self {
        Self {
            identity: match value.persistence {
                DocumentPersistence::Stored(id) => AttestationIdentity::Fixed { id },
                DocumentPersistence::InMemory => AttestationIdentity::Ephemeral,
            },
            attestation_type: value.doc_type.to_string(),
            display_metadata: vec![],
            issuer: value.issuer_registration.organization.into(),
            attributes: value.attributes.into_iter().map(AttestationAttribute::from).collect(),
        }
    }
}

pub enum AttestationIdentity {
    Ephemeral,
    Fixed { id: String },
}

impl From<wallet::AttestationIdentity> for AttestationIdentity {
    fn from(value: wallet::AttestationIdentity) -> Self {
        match value {
            wallet::AttestationIdentity::Ephemeral => AttestationIdentity::Ephemeral,
            wallet::AttestationIdentity::Fixed { id } => AttestationIdentity::Fixed { id },
        }
    }
}

pub struct DisplayMetadata {
    pub lang: String,
    pub name: String,
    pub description: Option<String>,
    pub rendering: Option<RenderingMetadata>,
}

impl From<wallet::sd_jwt::DisplayMetadata> for DisplayMetadata {
    fn from(value: wallet::sd_jwt::DisplayMetadata) -> Self {
        Self {
            lang: value.lang,
            name: value.name,
            description: value.description,
            rendering: value.rendering.map(RenderingMetadata::from),
        }
    }
}

pub enum RenderingMetadata {
    Simple {
        logo: Option<LogoMetadata>,
        background_color: Option<String>,
        text_color: Option<String>,
    },
    SvgTemplates,
}

impl From<wallet::sd_jwt::RenderingMetadata> for RenderingMetadata {
    fn from(value: wallet::sd_jwt::RenderingMetadata) -> Self {
        match value {
            wallet::sd_jwt::RenderingMetadata::Simple {
                logo,
                background_color,
                text_color,
            } => RenderingMetadata::Simple {
                logo: logo.map(LogoMetadata::from),
                background_color,
                text_color,
            },
            wallet::sd_jwt::RenderingMetadata::SvgTemplates => RenderingMetadata::SvgTemplates,
        }
    }
}

pub struct LogoMetadata {
    pub uri: String,
    pub uri_integrity: String,
    pub alt_text: String,
}

impl From<wallet::sd_jwt::LogoMetadata> for LogoMetadata {
    fn from(value: wallet::sd_jwt::LogoMetadata) -> Self {
        Self {
            uri: value.uri.to_string(),
            uri_integrity: value.uri_integrity.0,
            alt_text: value.alt_text.0,
        }
    }
}

pub struct ClaimDisplayMetadata {
    pub lang: String,
    pub label: String,
    pub description: Option<String>,
}

impl From<wallet::sd_jwt::ClaimDisplayMetadata> for ClaimDisplayMetadata {
    fn from(value: wallet::sd_jwt::ClaimDisplayMetadata) -> Self {
        Self {
            lang: value.lang,
            label: value.label,
            description: value.description,
        }
    }
}

pub struct AttestationAttribute {
    pub key: String,
    pub labels: Vec<ClaimDisplayMetadata>,
    pub value: AttributeValue,
}

impl From<wallet::AttestationAttribute> for AttestationAttribute {
    fn from(value: wallet::AttestationAttribute) -> Self {
        Self {
            key: value.key.join("__"),
            labels: value.metadata.into_iter().map(ClaimDisplayMetadata::from).collect(),
            value: value.value.into(),
        }
    }
}

impl From<(wallet::AttributeKey, wallet::Attribute)> for AttestationAttribute {
    fn from((key, value): (wallet::AttributeKey, wallet::Attribute)) -> Self {
        Self {
            key: key.to_string(),
            labels: value
                .key_labels
                .into_iter()
                .map(|(lang, label)| ClaimDisplayMetadata {
                    lang: lang.to_string(),
                    label: label.to_string(),
                    description: None,
                })
                .collect(),
            value: match value.value {
                wallet::AttributeValue::String(value) => AttributeValue::String { value },
                wallet::AttributeValue::Boolean(value) => AttributeValue::Boolean { value },
                wallet::AttributeValue::Date(value) => AttributeValue::String {
                    value: value.to_string(),
                },
                _ => unimplemented!(),
            },
        }
    }
}

pub enum AttributeValue {
    String { value: String },
    Boolean { value: bool },
    Number { value: i64 },
}

impl From<wallet::openid4vc::AttributeValue> for AttributeValue {
    fn from(value: wallet::openid4vc::AttributeValue) -> Self {
        match value {
            wallet::openid4vc::AttributeValue::Text(value) => AttributeValue::String { value },
            wallet::openid4vc::AttributeValue::Bool(value) => AttributeValue::Boolean { value },
            wallet::openid4vc::AttributeValue::Number(value) => AttributeValue::Number { value },
        }
    }
}
