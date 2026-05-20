pub enum Image {
    Svg { xml: SanitizedSvg },
    Png { data: Vec<u8> },
    Jpeg { data: Vec<u8> },
    Asset { path: String },
}

pub struct ImageWithMetadata {
    pub image: Image,
    pub alt_text: String,
}

// pub(crate) is required for direct field access in frb_generated.rs
pub struct SanitizedSvg(pub(crate) String);

impl From<svg_sanitize::SanitizedSvg> for SanitizedSvg {
    fn from(value: svg_sanitize::SanitizedSvg) -> Self {
        Self(value.into())
    }
}
