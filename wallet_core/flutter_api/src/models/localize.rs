pub struct LocalizedString {
    pub language: String,
    pub value: String,
}

pub struct LocalizedStrings(pub wallet::attestation_data::LocalizedStrings);

impl From<LocalizedStrings> for Vec<LocalizedString> {
    fn from(value: LocalizedStrings) -> Self {
        let LocalizedStrings(wallet::attestation_data::LocalizedStrings(localized_strings)) = value;
        localized_strings
            .iter()
            .map(|(language, value)| LocalizedString {
                language: language.to_owned(),
                value: value.to_owned(),
            })
            .collect()
    }
}
