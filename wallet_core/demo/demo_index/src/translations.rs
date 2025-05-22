use std::ops::Index;

use demo_utils::language::Language;

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
        index_title: "NL Wallet demo",
        index_intro: "These examples are fictional and for illustration purposes only. Follow the developments at",
        index_intro_link: "edi.pleio.nl",
        amsterdam_index: "Log in municipality",
        marketplace_index: "Log in webshop",
        xyz_index: "Open bank account",
        monkeybike_index: "Create account",
        job_index: "Apply for a job",
        university_index: "Get diploma cards",
        insurance_index: "Get insurance cards",
    },
    nl: Words {
        en: "English",
        nl: "Nederlands",
        index_title: "NL Wallet demo",
        index_intro: "Deze voorbeelden zijn fictief en dienen alleen ter illustratie. Volg de ontwikkelingen op",
        index_intro_link: "edi.pleio.nl",
        amsterdam_index: "Inloggen gemeente",
        marketplace_index: "Inloggen webshop",
        xyz_index: "Bankrekening openen",
        monkeybike_index: "Account aanmaken",
        job_index: "Reageer op vacature",
        university_index: "Ontvang diploma kaarten",
        insurance_index: "Ontvang verzekeringskaarten",
    },
};

pub struct Words<'a> {
    en: &'a str,
    nl: &'a str,
    pub index_title: &'a str,
    pub index_intro: &'a str,
    pub index_intro_link: &'a str,
    pub amsterdam_index: &'a str,
    pub marketplace_index: &'a str,
    pub xyz_index: &'a str,
    pub monkeybike_index: &'a str,
    pub job_index: &'a str,
    pub university_index: &'a str,
    pub insurance_index: &'a str,
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
