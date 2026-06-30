use std::sync::LazyLock;

use axum::extract::FromRequestParts;
use axum::extract::Query;
use axum::http::HeaderMap;
use axum::http::header::ACCEPT_LANGUAGE;
use axum::http::request::Parts;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use crypto::utils::sha256;
use serde::Deserialize;
use serde::Serialize;
use serde_with::DeserializeFromStr;
use serde_with::SerializeDisplay;

pub static LANGUAGE_JS_SHA256: LazyLock<String> =
    LazyLock::new(|| BASE64_STANDARD.encode(sha256(include_bytes!("../static/language.js"))));

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
    pub fn chrono_locale(&self) -> chrono::prelude::Locale {
        match self {
            Language::Nl => chrono::prelude::Locale::nl_NL,
            Language::En => chrono::prelude::Locale::en_GB,
        }
    }

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
    use axum::extract::FromRequestParts;
    use axum::http::HeaderValue;
    use axum::http::Request;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("en", Some(Language::En))]
    #[case("nl", Some(Language::Nl))]
    #[case("123", None)]
    #[case("en-gb", Some(Language::En))]
    #[case("nl-nl", Some(Language::Nl))]
    fn test_parse_language(#[case] s: &str, #[case] expected: Option<Language>) {
        assert_eq!(Language::parse(s), expected);
    }

    #[rstest]
    #[case("da, en-gb;q=0.8, en;q=0.7", Some(Language::En))]
    #[case("da, en;q=0.8, nl;q=0.7", Some(Language::En))]
    #[case("da, nl;q=0.8, en;q=0.7", Some(Language::Nl))]
    #[case("da, nl;q=0.7, en;q=0.8", Some(Language::En))]
    #[case("da, en-gb;q=0.8", Some(Language::En))]
    #[case("da, en;q=0.7", Some(Language::En))]
    #[case("nl, en-gb;q=0.8, en;q=0.7", Some(Language::Nl))]
    #[case("en, nl-nl;q=0.8, nl;q=0.7", Some(Language::En))]
    #[case("en, nl-be;q=0.8", Some(Language::En))]
    #[case("nl, en", Some(Language::Nl))]
    #[case("nl", Some(Language::Nl))]
    #[case("en", Some(Language::En))]
    #[case("da", None)]
    fn test_match_accept_language(#[case] accept_language: HeaderValue, #[case] expected: Option<Language>) {
        let mut headers = HeaderMap::new();
        headers.append(ACCEPT_LANGUAGE, accept_language);

        assert_eq!(Language::match_accept_language(&headers), expected);
    }

    /// Resolve a [`Language`] through the [`FromRequestParts`] extractor from an optional `?lang=`
    /// query value and an optional `Accept-Language` header value, exercising their precedence.
    async fn extract_language(lang_query: Option<&str>, accept_language: Option<&str>) -> Language {
        let uri = match lang_query {
            Some(lang) => format!("/?lang={lang}"),
            None => "/".to_string(),
        };
        let mut builder = Request::builder().uri(uri);
        if let Some(accept_language) = accept_language {
            builder = builder.header(ACCEPT_LANGUAGE, accept_language);
        }
        let (mut parts, ()) = builder.body(()).unwrap().into_parts();

        Language::from_request_parts(&mut parts, &()).await.unwrap()
    }

    #[rstest]
    // The `?lang=` query parameter takes precedence over the `Accept-Language` header.
    #[case(Some("en"), Some("nl"), Language::En)]
    #[case(Some("nl"), Some("en"), Language::Nl)]
    // Without a (valid) query parameter, the `Accept-Language` header is used.
    #[case(None, Some("nl, en;q=0.8"), Language::Nl)]
    #[case(None, Some("en, nl;q=0.8"), Language::En)]
    // An unsupported query language is ignored, falling back to the header.
    #[case(Some("fr"), Some("en"), Language::En)]
    // With neither a usable query parameter nor header, the default language is used.
    #[case(None, None, Language::Nl)]
    #[case(None, Some("fr"), Language::Nl)]
    #[tokio::test]
    async fn test_extractor_precedence(
        #[case] lang_query: Option<&str>,
        #[case] accept_language: Option<&str>,
        #[case] expected: Language,
    ) {
        assert_eq!(extract_language(lang_query, accept_language).await, expected);
    }
}
