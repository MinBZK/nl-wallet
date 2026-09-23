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
        index_title: "Try NL Wallet yourself",
        index_intro: &[
            "Here you can see examples of how you may use the NL Wallet app in the future.",
            "For example, you can add a driving license to your wallet. You can then show it to a police officer.",
            "These examples show how it may work in the future. The final version may be different.",
        ],
        index_needs_heading: "What do you need?",
        index_needs: "You need the {env} version of the NL Wallet app.",
        index_icon: "You can recognise this version by the word ‘{env}’ on the app icon.",
        index_opened_from: "Did you open this page from NL Wallet? Then you are using the right version.",
        index_info: "You can find more information about this development at ",
        index_info_link: "edi.pleio.nl",
        index_list_heading: "Choose an example below and follow the steps to get started.",
        amsterdam_mdoc_index: "Log in municipality (mdoc)",
        amsterdam_sd_jwt_index: "Log in municipality (SD-JWT)",
        marketplace_index: "Log in webshop",
        xyz_mdoc_index: "Open bank account (mdoc)",
        xyz_sd_jwt_index: "Open bank account (SD-JWT)",
        xyz_sd_jwt_eu_index: "Open bank account (SD-JWT, EU PID)",
        monkeybike_index: "Create account",
        job_index: "Apply for a job",
        university_mdoc_index: "Get diploma cards (mdoc)",
        university_sd_jwt_index: "Get diploma cards (SD-JWT)",
        insurance_index: "Get insurance cards",
        housing_index: "Get housing card",
        loyalty_index: "Get loyalty card",
        museum_maandkaart_index: "Get museum monthly pass",
        mdl_index: "Get mobile driver's license",
        mvc_index: "Get vehicle certificate",
    },
    nl: Words {
        en: "English",
        nl: "Nederlands",
        index_title: "Probeer de NL Wallet zelf uit",
        index_intro: &[
            "Hier vind je voorbeelden van hoe je de NL Wallet app in de toekomst kunt gebruiken.",
            "Je kunt bijvoorbeeld een rijbewijs toevoegen aan je wallet. Daarna kun je dat rijbewijs laten zien aan \
             een agent.",
            "De voorbeelden laten zien hoe dit in de toekomst zou kunnen werken. Maar het kan er ook anders uit gaan \
             zien.",
        ],
        index_needs_heading: "Wat heb je nodig?",
        index_needs: "Je hebt de {env}-versie van de NL Wallet app nodig.",
        index_icon: "Je herkent deze versie aan het woord ‘{env}’ op het app-icoon.",
        index_opened_from: "Heb je deze pagina geopend vanuit NL Wallet? Dan gebruik je de goede versie.",
        index_info: "Meer informatie over deze ontwikkeling vind je op ",
        index_info_link: "edi.pleio.nl",
        index_list_heading: "Kies hieronder een voorbeeld en volg de stappen om te beginnen.",
        amsterdam_mdoc_index: "Inloggen gemeente (mdoc)",
        amsterdam_sd_jwt_index: "Inloggen gemeente (SD-JWT)",
        marketplace_index: "Inloggen webshop",
        xyz_mdoc_index: "Bankrekening openen (mdoc)",
        xyz_sd_jwt_index: "Bankrekening openen (SD-JWT)",
        xyz_sd_jwt_eu_index: "Bankrekening openen (SD-JWT, EU PID)",
        monkeybike_index: "Account aanmaken",
        job_index: "Reageer op vacature",
        university_mdoc_index: "Ontvang diploma kaarten (mdoc)",
        university_sd_jwt_index: "Ontvang diploma kaarten (SD-JWT)",
        insurance_index: "Ontvang verzekeringskaarten",
        housing_index: "Ontvang huurkaart",
        loyalty_index: "Ontvang bonuskaart",
        museum_maandkaart_index: "Ontvang museumkaart",
        mdl_index: "Ontvang digitaal rijbewijs",
        mvc_index: "Ontvang kentekenbewijs",
    },
};

pub struct Words<'a> {
    en: &'a str,
    nl: &'a str,
    pub index_title: &'a str,
    pub index_intro: &'a [&'a str],
    pub index_needs_heading: &'a str,
    pub index_needs: &'a str,
    pub index_icon: &'a str,
    pub index_opened_from: &'a str,
    pub index_info: &'a str,
    pub index_info_link: &'a str,
    pub index_list_heading: &'a str,
    pub amsterdam_mdoc_index: &'a str,
    pub amsterdam_sd_jwt_index: &'a str,
    pub marketplace_index: &'a str,
    pub xyz_mdoc_index: &'a str,
    pub xyz_sd_jwt_index: &'a str,
    pub xyz_sd_jwt_eu_index: &'a str,
    pub monkeybike_index: &'a str,
    pub job_index: &'a str,
    pub university_mdoc_index: &'a str,
    pub university_sd_jwt_index: &'a str,
    pub insurance_index: &'a str,
    pub housing_index: &'a str,
    pub loyalty_index: &'a str,
    pub museum_maandkaart_index: &'a str,
    pub mdl_index: &'a str,
    pub mvc_index: &'a str,
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
