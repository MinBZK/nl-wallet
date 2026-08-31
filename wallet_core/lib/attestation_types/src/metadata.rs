use serde::Deserialize;
use serde::Serialize;
use serde_with::TryFromInto;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use utils::spec::SpecOptional;

use crate::data_uri::DataUri;
use crate::image::Image;

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
