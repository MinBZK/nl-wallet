use flutter_rust_bridge::frb;

#[frb(non_opaque)]
pub enum Image {
    Jpeg { data: Vec<u8> },
    Png { data: Vec<u8> },
    Svg { svg: SanitizedSvg },
    Asset { path: String },
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

impl TryFrom<Vec<u8>> for Image {
    type Error = Vec<u8>;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        // classify whether it's JPEG or PNG
        match value.as_slice() {
            // JPEG
            [0xFF, 0xD8, .., 0xFF, 0xD9]
            // JPEG 2000 JP2
            | [
                0x00,
                0x00,
                0x00,
                0x0C,
                0x6A,
                0x50,
                0x20,
                0x20,
                0x0D,
                0x0A,
                0x87,
                0x0A,
                ..,
                0xFF,
                0xD9,
            ]
            // JPEG 2000 codestream
            | [0xFF, 0x4F, 0xFF, 0x51, .., 0xFF, 0xD9] => Ok(Image::Jpeg { data: value }),
            // PNG // TODO is this acceptable, these are not supported in ISO 18013-5
            [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, .., 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82] => Ok(Image::Png { data: value }),
            _ => Err(value),
        }
    }
}

pub struct ImageWithMetadata {
    pub image: Image,
    pub alt_text: String,
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
