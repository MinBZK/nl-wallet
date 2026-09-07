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

impl Image {
    pub fn try_jpeg_from_bytes(value: Vec<u8>) -> Result<Self, Vec<u8>> {
        // classify whether it's JPEG
        // TODO add support for JPEG 2000 (PVW-6230)
        if matches!(value.as_slice(), &[0xFF, 0xD8, .., 0xFF, 0xD9]) {
            Ok(Image::Jpeg { data: value })
        } else {
            Err(value)
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
