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
