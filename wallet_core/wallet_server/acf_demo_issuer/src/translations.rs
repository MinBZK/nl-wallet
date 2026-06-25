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
        insurance_title: "InsurAnce",
        insurance_consent_body: "InsurAnce wants to add the following information to your NL Wallet",
        add_to_nl_wallet: "Give permission",
    },
    nl: Words {
        insurance_title: "VerzekerAar",
        insurance_consent_body: "VerzekerAar wil de volgende gegevens toevoegen aan je NL Wallet",
        add_to_nl_wallet: "Geef toestemming",
    },
};

pub struct Words<'a> {
    pub insurance_title: &'a str,
    pub insurance_consent_body: &'a str,
    pub add_to_nl_wallet: &'a str,
}
