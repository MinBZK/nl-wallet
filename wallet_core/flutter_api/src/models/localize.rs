use wallet::sd_jwt::ClaimDisplayMetadata;

#[derive(Clone)]
pub struct LocalizedString {
    pub language: String,
    pub value: String,
}

impl From<ClaimDisplayMetadata> for LocalizedString {
    fn from(value: ClaimDisplayMetadata) -> Self {
        Self {
            language: value.lang,
            value: value.label,
        }
    }
}
