use std::ops::Index;

use demo_utils::language::Language;

// TODO extract duplicate code by using generics
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
        demo_bar_text: "NL Wallet demo",
        demo_see_other: "View other",
        demo_see_examples: "examples",
        demo_follow_development: "Follow the developments at",
        university_title: "Holland University",
        university_get_card: "Get your digital diploma card",
        insurance_title: "InsurAnce",
        insurance_get_cards: "Get your digital insurance cards",
        add_to_nl_wallet: "Add to NL Wallet",
        choose_another_wallet: "Choose another wallet",
        failed_try_again: "Try again",
    },
    nl: Words {
        en: "English",
        nl: "Nederlands",
        demo_bar_text: "NL Wallet demo",
        demo_see_other: "Bekijk andere",
        demo_see_examples: "voorbeelden",
        demo_follow_development: "Volg de ontwikkelingen op",
        university_title: "Holland Universiteit",
        university_get_card: "Ontvang je digitale diplomakaart",
        insurance_title: "VerzekerAar",
        insurance_get_cards: "Ontvang je digitale verzekeringskaart",
        add_to_nl_wallet: "Toevoegen aan NL Wallet",
        choose_another_wallet: "Kies een andere wallet",
        failed_try_again: "Probeer opnieuw",
    },
};

pub struct Words<'a> {
    en: &'a str,
    nl: &'a str,
    pub demo_bar_text: &'a str,
    pub demo_see_other: &'a str,
    pub demo_see_examples: &'a str,
    pub demo_follow_development: &'a str,
    pub university_title: &'a str,
    pub university_get_card: &'a str,
    pub insurance_title: &'a str,
    pub insurance_get_cards: &'a str,
    pub add_to_nl_wallet: &'a str,
    pub choose_another_wallet: &'a str,
    pub failed_try_again: &'a str,
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
