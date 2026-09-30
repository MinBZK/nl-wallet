use std::str::FromStr;
use std::string::FromUtf8Error;

use serde::Deserialize;
use serde::Serialize;
use serde_with::base64::Base64;
use serde_with::serde_as;
use url::Url;

use crate::data_uri::DataUri;
use crate::data_uri::DataUriError;

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

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum ImageUriError {
    DataUri(DataUriError),
    Image(ImageError),
}

// Fetching external images is not supported (PVW-6115), so only `data:` URIs are supported
impl TryFrom<&Url> for Image {
    type Error = ImageUriError;

    fn try_from(value: &Url) -> Result<Self, Self::Error> {
        let data_uri = DataUri::from_str(value.as_str()).map_err(ImageUriError::DataUri)?;
        let image = Image::try_from(data_uri).map_err(ImageUriError::Image)?;

        Ok(image)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("data:image/png;base64,q80=")]
    #[case("data:image/jpeg;base64,yv4=")]
    #[case("data:image/svg+xml;utf8,<svg></svg>")]
    fn test_try_from_into_image(#[case] uri: &str) {
        let uri = DataUri::from_str(uri).unwrap();
        let image: Image = Image::try_from(uri.clone()).unwrap();
        assert_eq!(uri, image.into());
    }

    #[test]
    fn test_image_uri_unsupported_mime_type() {
        let uri = DataUri::from_str("data:image/webp;base64,q7o=").unwrap();
        let error = Image::try_from(uri).expect_err("should return error");
        assert!(matches!(error, ImageError::UnsupportedMimeType(mime_type) if mime_type == "image/webp"));
    }

    #[test]
    fn image_try_from_url_rejects_external_uri() {
        let url: Url = "https://example.com/logo.png".parse().unwrap();
        assert!(matches!(Image::try_from(&url), Err(ImageUriError::DataUri(_))));
    }

    #[test]
    fn image_try_from_url_decodes_embedded_uri() {
        let url: Url = "data:image/png;base64,q80=".parse().unwrap();
        assert_eq!(Image::try_from(&url).unwrap(), Image::Png(vec![0xab, 0xcd]));
    }
}
