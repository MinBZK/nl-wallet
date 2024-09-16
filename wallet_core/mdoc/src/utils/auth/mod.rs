use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};
use url::Url;

pub mod issuer_auth;
pub mod reader_auth;

type Language = String;

/// Holds multiple translations of the same field
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalizedStrings(pub IndexMap<Language, String>);

/// Allows convenient definitions of [`LocalizedStrings`] in Rust code.
impl From<Vec<(&str, &str)>> for LocalizedStrings {
    fn from(source: Vec<(&str, &str)>) -> Self {
        let map = source
            .into_iter()
            .map(|(language, value)| (language.to_owned(), value.to_owned()))
            .collect();
        LocalizedStrings(map)
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageType {
    #[default]
    #[serde(rename = "image/svg+xml")]
    Svg,
    #[serde(rename = "image/png")]
    Png,
    #[serde(rename = "image/jpeg")]
    Jpeg,
}

/// Encapsulates an image.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    /// Media Type of the image, expected to start with: `image/`.
    pub mime_type: ImageType,
    /// String encoded data of the image, f.e. XML text for `image/xml+svg`, or Base64 encoded binary data for
    /// `image/png`.
    pub image_data: String,
}

#[serde_as]
#[skip_serializing_none]
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Organization {
    pub display_name: LocalizedStrings,
    pub legal_name: LocalizedStrings,
    pub description: LocalizedStrings,
    pub category: LocalizedStrings,
    pub logo: Option<Image>,
    pub web_url: Option<Url>,
    pub kvk: Option<String>,
    pub city: Option<LocalizedStrings>,
    pub department: Option<LocalizedStrings>,
    pub country_code: Option<String>,
    pub privacy_policy_url: Option<Url>,
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use super::*;

    impl Organization {
        pub fn new_mock() -> Self {
            Organization {
                display_name: vec![("nl", "Mijn Organisatienaam"), ("en", "My Organization Name")].into(),
                legal_name: vec![("nl", "Organisatie"), ("en", "Organization")].into(),
                description: vec![
                    ("nl", "Beschrijving van Mijn Organisatie"),
                    ("en", "Description of My Organization"),
                ]
                .into(),
                category: vec![("nl", "Categorie"), ("en", "Category")].into(),
                kvk: Some("some-kvk".to_owned()),
                city: Some(vec![("nl", "Den Haag"), ("en", "The Hague")].into()),
                department: Some(vec![("nl", "Afdeling"), ("en", "Department")].into()),
                country_code: Some("nl".to_owned()),
                web_url: Some(Url::parse("https://organisation.example.com").unwrap()),
                privacy_policy_url: Some(Url::parse("https://organisation.example.com/privacy").unwrap()),
                logo: None,
            }
        }
    }
}
