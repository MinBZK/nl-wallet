use axum::extract::FromRequestParts;
use axum::extract::Query;
use axum::http::header::ACCEPT_LANGUAGE;
use axum::http::request::Parts;
use axum::http::HeaderMap;
use serde::Deserialize;
use serde::Serialize;
use serde_with::DeserializeFromStr;
use serde_with::SerializeDisplay;

#[derive(
    Debug,
    Default,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    SerializeDisplay,
    DeserializeFromStr,
    strum::EnumString,
    strum::Display,
    strum::EnumIter,
)]
pub enum Language {
    #[default]
    #[strum(to_string = "nl")]
    Nl,
    #[strum(to_string = "en")]
    En,
}

impl Language {
    fn parse(s: &str) -> Option<Self> {
        match s.split('-').next() {
            Some("en") => Some(Language::En),
            Some("nl") => Some(Language::Nl),
            _ => None,
        }
    }

    fn match_accept_language(headers: &HeaderMap) -> Option<Self> {
        let accept_language = headers.get(ACCEPT_LANGUAGE)?;
        let languages = accept_language::parse(accept_language.to_str().ok()?);

        // applies function to the elements of iterator and returns the first non-None result
        languages.into_iter().find_map(|l| Language::parse(&l))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LanguageParam {
    pub lang: Language,
}

impl<S> FromRequestParts<S> for Language
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> std::result::Result<Self, Self::Rejection> {
        let lang = Query::<LanguageParam>::from_request_parts(parts, state)
            .await
            .map(|l| l.lang)
            .unwrap_or(Language::match_accept_language(&parts.headers).unwrap_or_default());
        Ok(lang)
    }
}

#[cfg(test)]
mod test {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("en", Some(Language::En))]
    #[case("nl", Some(Language::Nl))]
    #[case("123", None)]
    #[case("en-GB", Some(Language::En))]
    #[case("nl-NL", Some(Language::Nl))]
    fn test_parse_language(#[case] s: &str, #[case] expected: Option<Language>) {
        assert_eq!(Language::parse(s), expected);
    }
}
