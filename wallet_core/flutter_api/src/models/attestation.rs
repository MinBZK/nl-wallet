use wallet::attestation_data;
use wallet::sd_jwt_vc_metadata::LogoMetadata;

use crate::models::disclosure::Organization;
use crate::models::image::Image;
use crate::models::image::ImageWithMetadata;

pub struct AttestationPresentation {
    pub identity: AttestationIdentity,
    pub attestation_type: String,
    pub display_metadata: Vec<DisplayMetadata>,
    pub issuer: Organization,
    pub attributes: Vec<AttestationAttribute>,
}

impl From<wallet::AttestationPresentation> for AttestationPresentation {
    fn from(value: wallet::AttestationPresentation) -> Self {
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
            wallet::AttestationIdentity::Fixed { id } => AttestationIdentity::Fixed { id: id.to_string() },
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
        logo: Option<ImageWithMetadata>,
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
                logo: logo.map(ImageWithMetadata::from),
                background_color,
                text_color,
            },
            wallet::sd_jwt_vc_metadata::RenderingMetadata::SvgTemplates => RenderingMetadata::SvgTemplates,
        }
    }
}

impl From<LogoMetadata> for ImageWithMetadata {
    fn from(value: LogoMetadata) -> Self {
        ImageWithMetadata {
            image: match value.image {
                wallet::sd_jwt_vc_metadata::Image::Svg(xml) => Image::Svg { xml },
                wallet::sd_jwt_vc_metadata::Image::Png(data) => Image::Png { data },
                wallet::sd_jwt_vc_metadata::Image::Jpeg(data) => Image::Jpeg { data },
            },
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
    pub svg_id: Option<String>,
}

impl From<wallet::AttestationAttribute> for AttestationAttribute {
    fn from(value: wallet::AttestationAttribute) -> Self {
        Self {
            key: value.key.join("__"),
            labels: value.metadata.into_iter().map(ClaimDisplayMetadata::from).collect(),
            value: value.value.into(),
            svg_id: value.svg_id,
        }
    }
}

pub enum AttributeValue {
    String { value: String },
    Boolean { value: bool },
    Number { value: i64 },
    Date { value: String },
    Null,
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

impl From<attestation_data::AttributeValue> for AttributeValue {
    fn from(value: attestation_data::AttributeValue) -> Self {
        match value {
            attestation_data::AttributeValue::Bool(value) => AttributeValue::Boolean { value },
            attestation_data::AttributeValue::Integer(value) => AttributeValue::Number { value },
            attestation_data::AttributeValue::Text(value) => AttributeValue::String { value },
            attestation_data::AttributeValue::Null => AttributeValue::Null,
            attestation_data::AttributeValue::Array(_) => todo!("implement in PVW-4001"),
        }
    }
}
