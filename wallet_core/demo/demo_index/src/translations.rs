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
    },
    nl: Words {
        en: "English",
        nl: "Nederlands",
        index_title: "NL Wallet demo",
        index_intro: "Deze voorbeelden zijn fictief en dienen alleen ter illustratie. Volg de ontwikkelingen op",
        index_intro_link: "edi.pleio.nl",
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
    },
};

pub struct Words<'a> {
    en: &'a str,
    nl: &'a str,
    pub index_title: &'a str,
    pub index_intro: &'a str,
    pub index_intro_link: &'a str,
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
