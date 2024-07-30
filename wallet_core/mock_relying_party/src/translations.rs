use std::{collections::HashMap, ops::Index, sync::LazyLock};

use serde::Serialize;

use crate::app::Language;

pub type Translations<'a> = HashMap<Language, Words<'a>>;

pub static TRANSLATIONS: LazyLock<Translations> = LazyLock::new(|| {
    let nl = Words {
        en: "English",
        nl: "Nederlands",
    };

    let en = Words {
        en: "English",
        nl: "Nederlands",
    };

    let mut translations = HashMap::new();
    translations.insert(Language::Nl, nl);
    translations.insert(Language::En, en);
    translations
});

#[derive(Serialize)]
pub struct Words<'a> {
    en: &'a str,
    nl: &'a str,
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
