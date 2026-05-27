use flutter_rust_bridge::frb;

#[frb(non_opaque)]
pub enum Image {
    Svg { svg: SanitizedSvg },
    Png { data: Vec<u8> },
    Jpeg { data: Vec<u8> },
    Asset { path: String },
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
