use serde::Deserialize;
use serde::Serialize;
use serde_with::TryFromInto;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use utils::spec::SpecOptional;
use utils::vec_at_least::VecNonEmpty;

use crate::claim_path::ClaimPath;
use crate::data_uri::DataUri;
use crate::data_uri::DataUriError;
use crate::image::Image;
use crate::image::ImageError;

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

/// Implemented by every kind of metadata that allows both validating a received attestation against its metadata and
/// converting it to a representation that can be shown to the user.
pub trait AttestationClaims {
    /// The paths of all claims that consist solely of `SelectByKey` components, as a path of `&str`. Claims with any
    /// other kind of path component are skipped, as those are not supported.
    fn claim_key_paths(&self) -> impl Iterator<Item = VecNonEmpty<&str>>;

    /// The paths of all claims the issuer is required to include in the attestation.
    fn mandatory_claims(&self) -> impl Iterator<Item = &VecNonEmpty<ClaimPath>>;
}

/// The description of a single claim of an attestation, independent of the kind of metadata it was derived from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimDescription {
    /// The path to the claim within the attestation.
    pub path: VecNonEmpty<ClaimPath>,

    /// How the claim is displayed to the user, per locale.
    pub display: Vec<ClaimDisplayMetadata>,

    /// The identifier of the claim for reference in an SVG template, if any.
    pub svg_id: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimDisplayMetadata {
    /// A language tag as defined in Section 2 of [RFC5646](https://www.rfc-editor.org/info/rfc5646).
    pub locale: String,

    /// A human-readable label for the claim, intended for end users.
    pub label: String,

    /// A human-readable description for the claim, intended for end users.
    pub description: Option<String>,
}

#[skip_serializing_none]
#[derive(derive_more::Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisplayMetadata {
    ///  A language tag as defined in Section 2 of [RFC5646](https://www.rfc-editor.org/info/rfc5646).
    pub locale: String,

    /// A human-readable name for the type, intended for end users.
    pub name: String,

    /// A human-readable description for the type, intended for end users.
    pub description: Option<String>,

    /// A templated summary for the type, intended to be rendered to the end user.
    pub summary: Option<String>,

    /// An object containing rendering information for the type
    #[debug(skip)]
    pub rendering: Option<RenderingMetadata>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RenderingMetadata {
    Simple {
        /// An object containing information about the logo to be displayed for the type.
        logo: Option<LogoMetadata>,

        /// An object containing information about the background image to be displayed for the type.
        background_image: Option<BackgroundImageMetadata>,

        /// An RGB color value for the background of the credential.
        background_color: Option<String>,

        /// An RGB color value for the text of the credential.
        text_color: Option<String>,
    },
    SvgTemplates,
}

#[serde_as]
#[derive(derive_more::Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogoMetadata {
    /// Explicitly reject non-embedded images and unsupported mime types
    #[debug(skip)]
    #[serde(rename = "uri")]
    #[serde_as(as = "TryFromInto<DataUri>")]
    pub image: Image,

    /// Note that although this is optional in the specification, it is mandatory within the context of the wallet app
    /// because of accessibility requirements.
    pub alt_text: SpecOptional<String>,
}

#[serde_as]
#[derive(derive_more::Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackgroundImageMetadata {
    /// Explicitly reject non-embedded images and unsupported mime types
    #[debug(skip)]
    #[serde(rename = "uri")]
    #[serde_as(as = "TryFromInto<DataUri>")]
    pub image: Image,
}
