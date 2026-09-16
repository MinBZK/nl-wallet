use std::str::FromStr;

use attestation_types::claim_path::ClaimPath;
use attestation_types::data_uri::DataUri;
use attestation_types::data_uri::DataUriError;
use attestation_types::image::Image;
use attestation_types::image::ImageError;
use openid4vc::metadata::issuer_metadata::BackgroundImage as CredentialBackgroundImage;
use openid4vc::metadata::issuer_metadata::CredentialClaim;
use openid4vc::metadata::issuer_metadata::CredentialDisplay;
use openid4vc::metadata::issuer_metadata::CredentialMetadata;
use openid4vc::metadata::issuer_metadata::Logo as CredentialLogo;
use openid4vc::metadata::issuer_metadata::NameLocale;
use openid4vc::wallet_issuance::issuance_session::OfferedCredentialMetadata;
use sd_jwt_vc_metadata::BackgroundImageMetadata;
use sd_jwt_vc_metadata::ClaimDisplayMetadata;
use sd_jwt_vc_metadata::ClaimMetadata;
use sd_jwt_vc_metadata::DisplayMetadata;
use sd_jwt_vc_metadata::LogoMetadata;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;
use sd_jwt_vc_metadata::RenderingMetadata;
use serde::Deserialize;
use serde::Serialize;
use serde_with::TryFromInto;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use url::Url;
use utils::vec_at_least::VecNonEmpty;

use crate::storage::StoredAttestationMetadata;

#[derive(Debug, thiserror::Error)]
pub enum AttestationMetadataError {
    #[error("display information is missing a name for locale {}", .0.as_deref().unwrap_or("<none>"))]
    NoDisplayName(Option<String>),

    #[error("display information is missing a locale")]
    NoDisplayLocale,

    #[error("could not read image as a data URI: {0}")]
    ImageDataUri(#[source] DataUriError),

    #[error("could not convert image: {0}")]
    Image(#[source] ImageError),
}

pub trait AttestationDisplay {
    fn into_presentation_components(
        self,
    ) -> Result<(Vec<AttestationDisplayMetadata>, Vec<ClaimDescription>), AttestationMetadataError>;
}

impl AttestationDisplay for CredentialMetadata {
    fn into_presentation_components(
        self,
    ) -> Result<(Vec<AttestationDisplayMetadata>, Vec<ClaimDescription>), AttestationMetadataError> {
        // Note that metadata without any display properties or claims is deliberately not an error here, but rather a
        // UI concern.
        let display = self
            .display
            .map(|display| {
                display
                    .into_inner()
                    .into_iter()
                    .map(AttestationDisplayMetadata::try_from)
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?
            .unwrap_or_default();

        let claims = self
            .claims
            .map(|claims| claims.into_inner().into_iter().map(ClaimDescription::from).collect())
            .unwrap_or_default();

        Ok((display, claims))
    }
}

// Note that this conversion is infallible. It also always yields display metadata, as a type metadata chain is
// validated to contain it when it is normalized.
impl AttestationDisplay for NormalizedTypeMetadata {
    fn into_presentation_components(
        self,
    ) -> Result<(Vec<AttestationDisplayMetadata>, Vec<ClaimDescription>), AttestationMetadataError> {
        let (display, claims) = self.into_display_and_claims();

        let display = display
            .into_inner()
            .into_iter()
            .map(AttestationDisplayMetadata::from)
            .collect();
        let claims = claims.into_iter().map(ClaimDescription::from).collect();

        Ok((display, claims))
    }
}

impl AttestationDisplay for StoredAttestationMetadata {
    fn into_presentation_components(
        self,
    ) -> Result<(Vec<AttestationDisplayMetadata>, Vec<ClaimDescription>), AttestationMetadataError> {
        match self {
            StoredAttestationMetadata::TypeMetadata(type_metadata) => type_metadata.into_presentation_components(),
            StoredAttestationMetadata::CredentialMetadata(credential_metadata) => {
                credential_metadata.into_presentation_components()
            }
        }
    }
}

impl AttestationDisplay for OfferedCredentialMetadata {
    fn into_presentation_components(
        self,
    ) -> Result<(Vec<AttestationDisplayMetadata>, Vec<ClaimDescription>), AttestationMetadataError> {
        match self {
            OfferedCredentialMetadata::TypeMetadata { normalized, .. } => normalized.into_presentation_components(),
            OfferedCredentialMetadata::CredentialMetadata(credential_metadata) => {
                credential_metadata.into_presentation_components()
            }
        }
    }
}

// Note that these types are stored as part of the JSON serialized `AttestationPresentation` of a history event, so
// their serialization has to remain compatible with that of the SD-JWT VC Type Metadata types they replace.

/// How an attestation is displayed to the user for a single locale, independent of the kind of metadata it was derived
/// from.
#[skip_serializing_none]
#[derive(derive_more::Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttestationDisplayMetadata {
    /// A language tag as defined in Section 2 of [RFC5646](https://www.rfc-editor.org/info/rfc5646).
    pub locale: String,

    /// A human-readable name for the attestation, intended for end users.
    pub name: String,

    /// A human-readable description for the attestation, intended for end users.
    pub description: Option<String>,

    /// A templated summary for the attestation, intended to be rendered to the end user. Only SD-JWT VC Type Metadata
    /// provides this.
    pub summary: Option<String>,

    /// How the attestation itself is to be rendered, if the metadata describes this.
    #[debug(skip)]
    pub rendering: Option<Rendering>,
}

/// How an attestation is to be rendered to the user.
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Rendering {
    Simple {
        /// The logo to be displayed for the attestation.
        logo: Option<Logo>,

        /// The background image to be displayed for the attestation.
        background_image: Option<BackgroundImage>,

        /// An RGB color value for the background of the attestation.
        background_color: Option<String>,

        /// An RGB color value for the text of the attestation.
        text_color: Option<String>,
    },
    SvgTemplates,
}

/// The logo of an attestation.
#[serde_as]
#[derive(derive_more::Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Logo {
    /// Explicitly reject non-embedded images and unsupported mime types
    #[debug(skip)]
    #[serde(rename = "uri")]
    #[serde_as(as = "TryFromInto<DataUri>")]
    pub image: Image,

    /// Alternative text for the image. Although this is optional in both specifications, it is mandatory within the
    /// context of the wallet app because of accessibility requirements, and is empty if the issuer did not provide it.
    pub alt_text: String,
}

/// The background image of an attestation.
#[serde_as]
#[derive(derive_more::Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackgroundImage {
    /// Explicitly reject non-embedded images and unsupported mime types
    #[debug(skip)]
    #[serde(rename = "uri")]
    #[serde_as(as = "TryFromInto<DataUri>")]
    pub image: Image,
}

/// The description of a single claim of an attestation, independent of the kind of metadata it was derived from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimDescription {
    /// The path to the claim within the attestation.
    pub path: VecNonEmpty<ClaimPath>,

    /// How the claim is displayed to the user, per locale.
    pub display: Vec<ClaimDisplay>,

    /// The identifier of the claim for reference in an SVG template, if any.
    pub svg_id: Option<String>,
}

/// How a single claim of an attestation is displayed to the user for a single locale.
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimDisplay {
    /// A language tag as defined in Section 2 of [RFC5646](https://www.rfc-editor.org/info/rfc5646).
    pub locale: String,

    /// A human-readable label for the claim, intended for end users.
    pub label: String,

    /// A human-readable description for the claim, intended for end users. Only SD-JWT VC Type Metadata provides this.
    pub description: Option<String>,
}

// Conversions from SD-JWT VC Type Metadata, all of which are infallible.

impl From<DisplayMetadata> for AttestationDisplayMetadata {
    fn from(value: DisplayMetadata) -> Self {
        Self {
            locale: value.locale,
            name: value.name,
            description: value.description,
            summary: value.summary,
            rendering: value.rendering.map(Rendering::from),
        }
    }
}

impl From<RenderingMetadata> for Rendering {
    fn from(value: RenderingMetadata) -> Self {
        match value {
            RenderingMetadata::Simple {
                logo,
                background_image,
                background_color,
                text_color,
            } => Self::Simple {
                logo: logo.map(Logo::from),
                background_image: background_image.map(BackgroundImage::from),
                background_color,
                text_color,
            },
            RenderingMetadata::SvgTemplates => Self::SvgTemplates,
        }
    }
}

impl From<LogoMetadata> for Logo {
    fn from(value: LogoMetadata) -> Self {
        Self {
            image: value.image,
            alt_text: value.alt_text.into_inner(),
        }
    }
}

impl From<BackgroundImageMetadata> for BackgroundImage {
    fn from(value: BackgroundImageMetadata) -> Self {
        Self { image: value.image }
    }
}

impl From<ClaimMetadata> for ClaimDescription {
    fn from(value: ClaimMetadata) -> Self {
        Self {
            path: value.path,
            display: value.display.into_iter().map(ClaimDisplay::from).collect(),
            svg_id: value.svg_id.map(String::from),
        }
    }
}

impl From<ClaimDisplayMetadata> for ClaimDisplay {
    fn from(value: ClaimDisplayMetadata) -> Self {
        Self {
            locale: value.locale,
            label: value.label,
            description: value.description,
        }
    }
}

// Conversions from Credential Issuer metadata, which are fallible because images are hosted externally and both the
// name and the locale of a display entry are optional in the specification.

impl TryFrom<CredentialDisplay> for AttestationDisplayMetadata {
    type Error = AttestationMetadataError;

    fn try_from(value: CredentialDisplay) -> Result<Self, Self::Error> {
        let CredentialDisplay {
            name_locale: NameLocale { name, locale },
            logo,
            description,
            background_color,
            background_image,
            text_color,
        } = value;

        let locale = locale.ok_or(AttestationMetadataError::NoDisplayLocale)?;
        let name = name.ok_or_else(|| AttestationMetadataError::NoDisplayName(Some(locale.clone())))?;

        let logo = logo.map(Logo::try_from).transpose()?;
        let background_image = background_image.map(BackgroundImage::try_from).transpose()?;

        // Only include rendering information if any of its properties is actually present.
        let rendering =
            (logo.is_some() || background_image.is_some() || background_color.is_some() || text_color.is_some())
                .then_some(Rendering::Simple {
                    logo,
                    background_image,
                    background_color,
                    text_color,
                });

        let display = Self {
            locale,
            name,
            description,
            // The Credential Issuer metadata has no equivalent of a templated summary.
            summary: None,
            rendering,
        };

        Ok(display)
    }
}

impl TryFrom<CredentialLogo> for Logo {
    type Error = AttestationMetadataError;

    fn try_from(value: CredentialLogo) -> Result<Self, Self::Error> {
        let logo = Self {
            image: image_from_uri(&value.uri)?,
            alt_text: value.alt_text.unwrap_or_default(),
        };

        Ok(logo)
    }
}

impl TryFrom<CredentialBackgroundImage> for BackgroundImage {
    type Error = AttestationMetadataError;

    fn try_from(value: CredentialBackgroundImage) -> Result<Self, Self::Error> {
        let background_image = Self {
            image: image_from_uri(&value.uri)?,
        };

        Ok(background_image)
    }
}

impl From<CredentialClaim> for ClaimDescription {
    fn from(value: CredentialClaim) -> Self {
        let display = value
            .display
            .map(|display| {
                display
                    .into_inner()
                    .into_iter()
                    // Both fields are optional in the specification, while the wallet requires them for display.
                    .filter_map(|NameLocale { name, locale }| {
                        Some(ClaimDisplay {
                            locale: locale?,
                            label: name?,
                            description: None,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Self {
            path: value.path,
            display,
            // Only SD-JWT VC Type Metadata provides SVG template identifiers.
            svg_id: None,
        }
    }
}

/// Decode an image that is embedded in a URI. Images hosted externally are rejected.
fn image_from_uri(uri: &Url) -> Result<Image, AttestationMetadataError> {
    let data_uri = DataUri::from_str(uri.as_str()).map_err(AttestationMetadataError::ImageDataUri)?;

    Image::try_from(data_uri).map_err(AttestationMetadataError::Image)
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use attestation_types::claim_path::ClaimPath;
    use attestation_types::image::Image;
    use openid4vc::metadata::issuer_metadata::CredentialClaim;
    use openid4vc::metadata::issuer_metadata::CredentialDisplay;
    use openid4vc::metadata::issuer_metadata::Logo as CredentialLogo;
    use openid4vc::metadata::issuer_metadata::NameLocale;
    use utils::vec_at_least::VecNonEmpty;
    use utils::vec_nonempty;

    use super::*;

    fn key_path(keys: &[&str]) -> VecNonEmpty<ClaimPath> {
        keys.iter()
            .map(|key| ClaimPath::SelectByKey(String::from(*key)))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }

    #[test]
    fn test_credential_metadata_presentation_components() {
        let (display, claims) = CredentialMetadata::new_full_example()
            .into_presentation_components()
            .expect("credential metadata should convert to presentation components");

        let display = display.into_iter().next().expect("display should contain one entry");
        assert_eq!(display.locale, "en");
        assert_eq!(display.name, "Example credential");
        assert_eq!(display.description.as_deref(), Some("An example"));
        // The Credential Issuer metadata has no equivalent of a templated summary.
        assert!(display.summary.is_none());

        let rendering = display.rendering.expect("display should contain rendering metadata");
        assert_matches!(
            rendering,
            Rendering::Simple {
                logo: Some(logo),
                background_image: Some(background_image),
                background_color: Some(background_color),
                text_color: Some(text_color),
            } if matches!(logo.image, Image::Png(_))
                && logo.alt_text == "a single pixel"
                && matches!(background_image.image, Image::Png(_))
                && background_color == "#FFFFFF"
                && text_color == "#000000"
        );

        assert_eq!(
            claims.iter().map(|claim| claim.path.clone()).collect::<Vec<_>>(),
            vec![key_path(&["birth_date"]), key_path(&["place_of_birth", "locality"])]
        );
        assert_eq!(
            claims.first().unwrap().display,
            vec![ClaimDisplay {
                locale: String::from("en"),
                label: String::from("label for birth_date"),
                description: None,
            }]
        );
        // Only SD-JWT VC Type Metadata provides SVG template identifiers.
        assert!(claims.iter().all(|claim| claim.svg_id.is_none()));
    }

    /// Type metadata always yields display properties, as a chain is validated to contain them when normalized.
    #[test]
    fn test_type_metadata_presentation_components() {
        let (display, claims) = NormalizedTypeMetadata::nl_pid_example()
            .into_presentation_components()
            .expect("type metadata should convert to presentation components");

        assert!(!display.is_empty());
        assert!(!claims.is_empty());
    }

    /// Metadata that describes no display properties is not an error, as it is up to the UI to decide what to render.
    #[test]
    fn test_credential_metadata_without_display() {
        let metadata = CredentialMetadata {
            display: None,
            claims: Some(vec_nonempty![CredentialClaim {
                path: key_path(&["birth_date"]),
                mandatory: false,
                display: None,
            }]),
        };

        let (display, claims) = metadata
            .into_presentation_components()
            .expect("credential metadata without display should convert");

        assert!(display.is_empty());
        assert_eq!(claims.len(), 1);
    }

    #[test]
    fn test_credential_metadata_presentation_components_error_external_logo() {
        let metadata = CredentialMetadata {
            display: Some(vec_nonempty![CredentialDisplay {
                name_locale: NameLocale {
                    name: Some(String::from("Example credential")),
                    locale: Some(String::from("en")),
                },
                // Only images that are embedded in the URI are accepted.
                logo: Some(CredentialLogo {
                    uri: "https://example.com/logo.png".parse().unwrap(),
                    alt_text: None,
                }),
                description: None,
                background_color: None,
                background_image: None,
                text_color: None,
            }]),
            claims: None,
        };

        let error = metadata
            .into_presentation_components()
            .expect_err("credential metadata with an externally hosted logo should not convert");

        assert_matches!(error, AttestationMetadataError::ImageDataUri(_));
    }

    /// Both the name and the locale of a display entry are optional in the specification, while the wallet requires
    /// them in order to present the attestation.
    #[test]
    fn test_credential_metadata_presentation_components_error_incomplete_display() {
        let display = |name_locale| CredentialDisplay {
            name_locale,
            logo: None,
            description: None,
            background_color: None,
            background_image: None,
            text_color: None,
        };

        let error = CredentialMetadata {
            display: Some(vec_nonempty![display(NameLocale {
                name: Some(String::from("Example credential")),
                locale: None,
            })]),
            claims: None,
        }
        .into_presentation_components()
        .expect_err("credential metadata without a display locale should not convert");

        assert_matches!(error, AttestationMetadataError::NoDisplayLocale);

        let error = CredentialMetadata {
            display: Some(vec_nonempty![display(NameLocale {
                name: None,
                locale: Some(String::from("en")),
            })]),
            claims: None,
        }
        .into_presentation_components()
        .expect_err("credential metadata without a display name should not convert");

        assert_matches!(error, AttestationMetadataError::NoDisplayName(Some(locale)) if locale == "en");
    }

    #[test]
    fn test_credential_metadata_without_claims() {
        let metadata = CredentialMetadata {
            display: Some(vec_nonempty![CredentialDisplay {
                name_locale: NameLocale {
                    name: Some(String::from("Example credential")),
                    locale: Some(String::from("en")),
                },
                logo: None,
                description: None,
                background_color: None,
                background_image: None,
                text_color: None,
            }]),
            claims: None,
        };

        let (display, claims) = metadata
            .into_presentation_components()
            .expect("credential metadata without claims should convert");

        // Rendering metadata is only present if any of its properties is.
        assert!(
            display
                .into_iter()
                .next()
                .expect("display should contain one entry")
                .rendering
                .is_none()
        );
        assert!(claims.is_empty());
    }
}
