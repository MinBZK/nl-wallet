use std::ops::Index;

use web_utils::language::Language;

pub struct Translations<'a> {
    en: Words<'a>,
    nl: Words<'a>,
}

impl<'a> Index<Language> for Translations<'a> {
    type Output = Words<'a>;

    fn index(&self, lang: Language) -> &Self::Output {
        match lang {
            Language::Nl => &self.nl,
            Language::En => &self.en,
        }
    }
}

pub const TRANSLATIONS: Translations = Translations {
    en: Words {
        en: "English",
        nl: "Nederlands",
        insurance_title: "InsurAnce",
        insurance_consent_body: "InsurAnce wants to add the following information to your NL Wallet",
        add_to_nl_wallet: "Give permission",
    },
    nl: Words {
        en: "English",
        nl: "Nederlands",
        insurance_title: "VerzekerAar",
        insurance_consent_body: "VerzekerAar wil de volgende gegevens toevoegen aan je NL Wallet",
        add_to_nl_wallet: "Geef toestemming",
    },
};

pub struct Words<'a> {
    /// Display name of the English language, used by the language selector.
    en: &'a str,
    /// Display name of the Dutch language, used by the language selector.
    nl: &'a str,
    pub insurance_title: &'a str,
    pub insurance_consent_body: &'a str,
    pub add_to_nl_wallet: &'a str,
}

/// Indexing a [`Words`] by [`Language`] yields that language's own display name, so the language
/// selector can label each option in its native form regardless of the currently selected language.
impl<'a> Index<Language> for Words<'a> {
    type Output = &'a str;

    fn index(&self, lang: Language) -> &Self::Output {
        match lang {
            Language::Nl => &self.nl,
            Language::En => &self.en,
        }
    }
}
