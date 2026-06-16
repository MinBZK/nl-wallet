use std::string::FromUtf8Error;

use serde::Deserialize;
use serde::Serialize;
use serde_with::base64::Base64;
use serde_with::serde_as;

use crate::data_uri::DataUri;

/// Encapsulates an image.
#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mimeType", content = "imageData")]
pub enum Image {
    #[serde(rename = "image/jpeg")]
    Jpeg(#[serde_as(as = "Base64")] Vec<u8>),
    #[serde(rename = "image/png")]
    Png(#[serde_as(as = "Base64")] Vec<u8>),
    #[serde(rename = "image/svg+xml")]
    Svg(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ImageError {
    #[error("utf8 decode error: {0}")]
    Utf8Decode(#[from] FromUtf8Error),
    #[error("unsupported mime type: {0}")]
    UnsupportedMimeType(String),
}

impl TryFrom<DataUri> for Image {
    type Error = ImageError;

    fn try_from(value: DataUri) -> Result<Self, Self::Error> {
        match value.mime_type.as_str() {
            "image/jpeg" => Ok(Image::Jpeg(value.data)),
            "image/png" => Ok(Image::Png(value.data)),
            "image/svg+xml" => String::from_utf8(value.data)
                .map(Image::Svg)
                .map_err(ImageError::Utf8Decode),
            _ => Err(ImageError::UnsupportedMimeType(value.mime_type)),
        }
    }
}

impl From<Image> for DataUri {
    fn from(value: Image) -> Self {
        match value {
            Image::Jpeg(data) => DataUri {
                mime_type: String::from("image/jpeg"),
                data,
            },
            Image::Png(data) => DataUri {
                mime_type: String::from("image/png"),
                data,
            },
            Image::Svg(xml) => DataUri {
                mime_type: String::from("image/svg+xml"),
                data: xml.as_bytes().to_vec(),
            },
        }
    }
}
