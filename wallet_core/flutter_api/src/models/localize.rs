#[derive(Clone)]
pub struct LocalizedString {
    pub language: String,
    pub value: String,
}

impl From<wallet::LocalizedString> for LocalizedString {
    fn from(value: wallet::LocalizedString) -> Self {
        Self {
            language: value.language,
            value: value.value,
        }
    }
}
