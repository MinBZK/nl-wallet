use indexmap::IndexMap;
use serde::Deserialize;
use serde::Serialize;
use serde_with::base64::Base64;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
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

/// Encapsulates an image.
#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mimeType", content = "imageData")]
pub enum Image {
    #[serde(rename = "image/svg+xml")]
    Svg(String),
    #[serde(rename = "image/png")]
    Png(#[serde_as(as = "Base64")] Vec<u8>),
    #[serde(rename = "image/jpeg")]
    Jpeg(#[serde_as(as = "Base64")] Vec<u8>),
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

#[cfg(test)]
pub mod test {
    use super::*;

    use rstest::rstest;
    use serde_json::json;

    #[rstest]
    #[case("image/svg+xml", "<svg></svg>", Image::Svg("<svg></svg>".to_owned()))]
    #[case("image/png", "yv4=", Image::Png(vec![0xca, 0xfe]))]
    #[case("image/jpeg", "q80=", Image::Jpeg(vec![0xab, 0xcd]))]
    fn image_deserialize(#[case] mime_type: &str, #[case] image_data: &str, #[case] expected: Image) {
        assert_eq!(
            serde_json::from_value::<Image>(json!({"mimeType": mime_type ,"imageData": image_data})).unwrap(),
            expected,
        )
    }
}
