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
    pub summary: Option<String>,
    pub rendering: Option<RenderingMetadata>,
}

impl From<wallet::sd_jwt_vc_metadata::DisplayMetadata> for DisplayMetadata {
    fn from(value: wallet::sd_jwt_vc_metadata::DisplayMetadata) -> Self {
        Self {
            lang: value.lang,
            name: value.name,
            description: value.description,
            summary: value.summary,
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

impl From<wallet::sd_jwt_vc_metadata::RenderingMetadata> for RenderingMetadata {
    fn from(value: wallet::sd_jwt_vc_metadata::RenderingMetadata) -> Self {
        match value {
            wallet::sd_jwt_vc_metadata::RenderingMetadata::Simple {
                logo,
                background_color,
                text_color,
            } => RenderingMetadata::Simple {
                logo: logo.map(LogoMetadata::from),
                background_color,
                text_color,
            },
            wallet::sd_jwt_vc_metadata::RenderingMetadata::SvgTemplates => RenderingMetadata::SvgTemplates,
        }
    }
}

pub struct LogoMetadata {
    pub uri: String,
    pub uri_integrity: String,
    pub alt_text: String,
}

impl From<wallet::sd_jwt_vc_metadata::LogoMetadata> for LogoMetadata {
    fn from(value: wallet::sd_jwt_vc_metadata::LogoMetadata) -> Self {
        Self {
            uri: value.uri.to_string(),
            uri_integrity: value.uri_integrity.into_inner().to_string(),
            alt_text: value.alt_text.into_inner(),
        }
    }
}

pub struct ClaimDisplayMetadata {
    pub lang: String,
    pub label: String,
    pub description: Option<String>,
}

impl From<wallet::sd_jwt_vc_metadata::ClaimDisplayMetadata> for ClaimDisplayMetadata {
    fn from(value: wallet::sd_jwt_vc_metadata::ClaimDisplayMetadata) -> Self {
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

pub enum AttributeValue {
    String { value: String },
    Boolean { value: bool },
    Number { value: i64 },
    Date { value: String },
}

impl From<wallet::AttestationAttributeValue> for AttributeValue {
    fn from(value: wallet::AttestationAttributeValue) -> Self {
        match value {
            wallet::AttestationAttributeValue::Basic(value) => value.into(),
            wallet::AttestationAttributeValue::Date(value) => AttributeValue::Date {
                value: value.format("%Y-%m-%d").to_string(),
            },
        }
    }
}

impl From<wallet::openid4vc::AttributeValue> for AttributeValue {
    fn from(value: wallet::openid4vc::AttributeValue) -> Self {
        match value {
            wallet::openid4vc::AttributeValue::Bool(value) => AttributeValue::Boolean { value },
            wallet::openid4vc::AttributeValue::Integer(value) => AttributeValue::Number { value },
            wallet::openid4vc::AttributeValue::Text(value) => AttributeValue::String { value },
        }
    }
}
