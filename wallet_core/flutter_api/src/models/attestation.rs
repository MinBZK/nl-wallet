use anyhow::anyhow;

use crate::models::disclosure::Organization;
use crate::models::image::Image;
use crate::models::image::ImageWithMetadata;

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
                logo: logo.map(ImageWithMetadata::try_from).and_then(Result::ok),
                background_color,
                text_color,
            },
            wallet::sd_jwt_vc_metadata::RenderingMetadata::SvgTemplates => RenderingMetadata::SvgTemplates,
        }
    }
}

impl TryFrom<wallet::sd_jwt_vc_metadata::LogoMetadata> for ImageWithMetadata {
    type Error = anyhow::Error;

    fn try_from(value: wallet::sd_jwt_vc_metadata::LogoMetadata) -> Result<Self, Self::Error> {
        // For simplicity, we let the embedded and remote distinction bubble up to here
        // and only convert supported embedded images.
        // If remote images were to be supported, there should be some logic that fetches the url,
        // most likely that concern should not be here.
        let alt_text = value.alt_text.into_inner();
        match value.uri_metadata {
            wallet::sd_jwt_vc_metadata::UriMetadata::Embedded { uri } => match uri.mime_type.as_str() {
                "image/jpeg" => Ok(Image::Jpeg { data: uri.data }),
                "image/png" => Ok(Image::Png { data: uri.data }),
                "image/svg+xml" => String::from_utf8(uri.data)
                    .map(|xml| Image::Svg { xml })
                    .map_err(anyhow::Error::from),
                _ => Err(anyhow!("Unsupported mime type: {}", uri.mime_type)),
            }
            .map(|image| ImageWithMetadata { image, alt_text }),
            wallet::sd_jwt_vc_metadata::UriMetadata::Remote { .. } => Err(anyhow!("Remote images are not supported")),
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
            wallet::openid4vc::AttributeValue::Array(_) => todo!("implement in PVW-4001"),
        }
    }
}
