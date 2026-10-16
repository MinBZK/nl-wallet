use std::io::Cursor;

use flutter_rust_bridge::frb;
use image::ImageError;
use image::ImageFormat;
use image::ImageReader;
use tracing::warn;

#[frb(non_opaque)]
pub enum Image {
    Jpeg { data: Vec<u8> },
    Png { data: Vec<u8> },
    Bmp { data: Vec<u8> },
    Svg { svg: SanitizedSvg },
    Asset { path: String },
}

impl Image {
    fn new_bmp_from_jpeg2000(input: &[u8]) -> Result<Self, ImageError> {
        // Register the JPEG 2000 decoder with the image crate.
        hayro_jpeg2000::integration::register_decoding_hook();

        let mut output = Cursor::new(Vec::new());
        ImageReader::new(Cursor::new(input))
            .with_guessed_format()?
            .decode()?
            .write_to(&mut output, ImageFormat::Bmp)?;

        let image = Self::Bmp {
            data: output.into_inner(),
        };

        Ok(image)
    }
}

impl TryFrom<wallet::attestation_types::Image> for Image {
    type Error = svg_sanitize::Error;

    fn try_from(value: wallet::attestation_types::Image) -> Result<Self, Self::Error> {
        Ok(match value {
            wallet::attestation_types::Image::Jpeg(data) => Image::Jpeg { data },
            wallet::attestation_types::Image::Png(data) => Image::Png { data },
            wallet::attestation_types::Image::Svg(xml) => Image::Svg {
                svg: svg_sanitize::SanitizedSvg::try_new(&xml)?.into(),
            },
        })
    }
}

impl TryFrom<wallet::ImageAttribute> for Image {
    type Error = Vec<u8>;

    fn try_from(value: wallet::ImageAttribute) -> Result<Self, Self::Error> {
        match value {
            wallet::ImageAttribute::Jpeg(data) => Ok(Self::Jpeg { data }),
            wallet::ImageAttribute::Jpeg2000(data) => Self::new_bmp_from_jpeg2000(&data).map_err(|error| {
                warn!("could not convert JPEG 2000 image to BMP: {error}");

                data
            }),
        }
    }
}

pub struct ImageWithMetadata {
    pub image: Image,
    pub alt_text: Option<String>,
}

#[frb(opaque)]
pub struct SanitizedSvg(svg_sanitize::SanitizedSvg);

impl From<svg_sanitize::SanitizedSvg> for SanitizedSvg {
    fn from(value: svg_sanitize::SanitizedSvg) -> Self {
        Self(value)
    }
}

impl SanitizedSvg {
    #[frb(sync)]
    pub fn xml(&self) -> String {
        self.0.as_ref().to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::Image;

    const JPEG2000_IMAGE: &[u8] = include_bytes!("../../assets/tests/willeke.jp2");

    #[test]
    fn test_image_new_bmp_from_jpeg2000() {
        let image = Image::new_bmp_from_jpeg2000(JPEG2000_IMAGE).expect("JPEG 2000 image should decode");

        // Detect BMP magic bytes.
        assert!(matches!(&image, Image::Bmp { data } if matches!(data.as_slice(), [0x42, 0x4d, ..])));
    }
}
