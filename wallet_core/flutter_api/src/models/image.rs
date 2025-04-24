pub enum Image {
    Svg { xml: String },
    Png { data: Vec<u8> },
    Jpeg { data: Vec<u8> },
    Asset { path: String },
}

pub struct ImageWithMetadata {
    pub image: Image,
    pub alt_text: String,
}
