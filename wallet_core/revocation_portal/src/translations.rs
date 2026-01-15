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
        index_title: "NL Wallet revocation portal",
    },
    nl: Words {
        en: "English",
        nl: "Nederlands",
        index_title: "NL Wallet revocatie portaal",
    },
};

pub struct Words<'a> {
    en: &'a str,
    nl: &'a str,
    pub index_title: &'a str,
}

impl<'a> Index<Language> for Words<'a> {
    type Output = &'a str;

    fn index(&self, lang: Language) -> &Self::Output {
        match lang {
            Language::Nl => &self.nl,
            Language::En => &self.en,
        }
    }
}
