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
        delete_title: "Do you want to delete your NL Wallet?",
        delete_description: "Something happened to your phone? Or do you want to delete your NL Wallet? Use your \
                             deletion code to delete your NL Wallet and all your data. This stops anyone from using \
                             it.",
        delete_warning: "This action cannot be undone.",
        delete_step1_instruction: "Enter your 18-digit deletion code that you wrote down when creating the wallet.",
        delete_code_label: "Your deletion code",
        delete_confirm: "Delete wallet",
        delete_code_invalid_length: "The code must be exactly 18 characters long.",
        delete_code_required: "Please enter your deletion code.",
        cancel: "Cancel",
        help_title: "Do you have questions or need help?",
        need_help: "Need help?",
    },
    nl: Words {
        en: "English",
        nl: "Nederlands",
        index_title: "NL Wallet revocatie portaal",
        delete_title: "Wil je je NL Wallet verwijderen?",
        delete_description: "Is er iets mis met je telefoon? Of wil je je NL Wallet verwijderen? Gebruik je \
                             verwijderingscode om je NL Wallet en alle gegevens te verwijderen. Dit weerhoudt \
                             iedereen van het gebruik ervan.",
        delete_warning: "Deze actie kan niet ongedaan worden gemaakt.",
        delete_step1_instruction: "Voer je 18-cijferige verwijderingscode in die je hebt opgeschreven bij het \
                                   aanmaken van de wallet.",
        delete_code_label: "Je verwijderingscode",
        delete_confirm: "Wallet verwijderen",
        delete_code_invalid_length: "De code moet precies 18 tekens lang zijn.",
        delete_code_required: "Voer uw verwijderingscode in.",
        cancel: "Annuleren",
        help_title: "Heb je vragen of hulp nodig?",
        need_help: "Hulp nodig?",
    },
};

pub struct Words<'a> {
    en: &'a str,
    nl: &'a str,
    pub index_title: &'a str,
    pub delete_title: &'a str,
    pub delete_description: &'a str,
    pub delete_warning: &'a str,
    pub delete_step1_instruction: &'a str,
    pub delete_code_label: &'a str,
    pub delete_confirm: &'a str,
    pub delete_code_invalid_length: &'a str,
    pub delete_code_required: &'a str,
    pub cancel: &'a str,
    pub help_title: &'a str,
    pub need_help: &'a str,
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
