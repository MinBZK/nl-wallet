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
        meta_title: "NL Wallet revocation portal",
        delete_title: "Do you want to delete your NL Wallet?",
        success_title: "Your NL Wallet has been blocked",
        error_title: "Sorry, something went wrong",
        delete_description: "Something happened to your phone? Or do you want to delete your NL Wallet? Use your \
                             deletion code to delete your NL Wallet and all your data. This stops anyone from using \
                             it.",
        delete_warning: "This action cannot be undone.",
        delete_step1_instruction: "Enter your 18-digit deletion code that you wrote down when creating the wallet.",
        delete_code_label: "Your deletion code",
        delete_confirm: "Delete wallet",
        delete_code_invalid_length: "The code must be exactly 18 characters long.",
        delete_code_required: "Please enter your deletion code.",
        delete_code_incorrect: "Deletion code is incorrect. Please try again.",
        success_wb_confirmation: "You used your deletion code to delete your NL Wallet on {} at {}. Your wallet has \
                                  been blocked and no one can use your wallet.",
        success_wallet_confirmation: "When your wallet connects to the internet, all your data in the wallet will be \
                                      deleted.",
        error_description: "This action was unsuccessful. This could have several reasons. Please try again.",
        cancel: "Cancel",
        help_title: "Do you have questions or need help?",
        need_help: "Need help?",
        back_to_support: "Back to support",
        try_again: "Try again",
        download_text: "Write your feedback or download the new NL Wallet in the app store.",
        apple_app_store: "Download in the App Store",
        google_play_store: "Discover on Google Play",
    },
    nl: Words {
        en: "English",
        nl: "Nederlands",
        meta_title: "NL Wallet revocatie portaal",
        delete_title: "Wil je je NL Wallet verwijderen?",
        success_title: "Je NL Wallet is stopgezet",
        error_title: "Sorry, er is iets fout gegaan",
        delete_description: "Is er iets mis met je telefoon gebeurd? Of wil je je NL Wallet verwijderen? Gebruik je \
                             verwijderingscode om je NL Wallet en alle gegevens te verwijderen. Zo kan niemand deze \
                             gebruiken.",
        delete_warning: "Deze actie kan niet ongedaan worden gemaakt.",
        delete_step1_instruction: "Voer je 18-cijferige verwijderingscode in die je hebt opgeschreven bij het \
                                   aanmaken van de wallet.",
        delete_code_label: "Je verwijderingscode",
        delete_confirm: "Wallet verwijderen",
        delete_code_invalid_length: "De code moet precies 18 tekens lang zijn.",
        delete_code_required: "Voer uw verwijderingscode in.",
        delete_code_incorrect: "Verwijderingscode is incorrect. Probeer het opnieuw.",
        success_wb_confirmation: "Je hebt verwijderings-code gebruikt om je NL Wallet te verwijderen op {} om {}. Je \
                                  wallet is stopgezet en niemand kan je wallet gebruiken",
        success_wallet_confirmation: "Als je wallet verbinding maakt met het internet, worden alle gegevens in je \
                                      wallet verwijderd.",
        error_description: "Deze actie is niet gelukt. Dit kan verschillende redenen hebben. Probeer het opnieuw.",
        cancel: "Annuleren",
        help_title: "Heb je vragen of hulp nodig?",
        need_help: "Hulp nodig?",
        back_to_support: "Terug naar support",
        try_again: "Probeer opnieuw",
        download_text: "Geef je feedback of download de nieuwe NL Wallet in de appstore.",
        apple_app_store: "Download in de App Store",
        google_play_store: "Ontdek het op Google Play",
    },
};

pub struct Words<'a> {
    en: &'a str,
    nl: &'a str,
    pub meta_title: &'a str,
    pub delete_title: &'a str,
    pub success_title: &'a str,
    pub error_title: &'a str,
    pub delete_description: &'a str,
    pub delete_warning: &'a str,
    pub delete_step1_instruction: &'a str,
    pub delete_code_label: &'a str,
    pub delete_confirm: &'a str,
    pub delete_code_invalid_length: &'a str,
    pub delete_code_required: &'a str,
    pub delete_code_incorrect: &'a str,
    pub success_wb_confirmation: &'a str,
    pub success_wallet_confirmation: &'a str,
    pub error_description: &'a str,
    pub cancel: &'a str,
    pub help_title: &'a str,
    pub need_help: &'a str,
    pub back_to_support: &'a str,
    pub try_again: &'a str,
    pub download_text: &'a str,
    pub apple_app_store: &'a str,
    pub google_play_store: &'a str,
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
