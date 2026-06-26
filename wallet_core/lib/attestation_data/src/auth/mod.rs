pub mod issuer_auth;
pub mod reader_auth;

use attestation_types::image::Image;
use derive_more::Debug;
use indexmap::IndexMap;
use serde::Deserialize;
use serde::Serialize;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use url::Url;

type Language = String;

/// Holds multiple translations of the same field
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalizedStrings(pub IndexMap<Language, String>);

#[serde_as]
#[skip_serializing_none]
// TODO: Check if serde is still necessary when Issuer and Reader registrations are removed (PVW-5895,PVW-5870)
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Organization {
    pub display_name: String,
    pub legal_name: String,
    pub description: LocalizedStrings,
    pub category: LocalizedStrings,
    #[debug(skip)]
    pub logo: Option<Image>,
    pub web_url: Option<Url>,
    // TODO: Remove rename when Issuer and Reader registrations are removed (PVW-5895,PVW-5870)
    #[serde(rename = "kvk")]
    pub identifier: Option<String>,
    pub city: Option<LocalizedStrings>,
    pub department: Option<LocalizedStrings>,
    pub country_code: String,
    pub privacy_policy_url: Option<Url>,
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use super::*;

    /// Allows convenient definitions of [`LocalizedStrings`] in Rust code.
    impl<'a, I: IntoIterator<Item = (&'a str, &'a str)>> From<I> for LocalizedStrings {
        fn from(source: I) -> Self {
            let map = source
                .into_iter()
                .map(|(language, value)| (language.to_owned(), value.to_owned()))
                .collect();
            LocalizedStrings(map)
        }
    }

    impl Organization {
        pub fn new_mock() -> Box<Self> {
            Organization {
                display_name: "Mijn Organisatienaam".to_owned(),
                legal_name: "Organisatie".to_owned(),
                description: [
                    ("nl", "Beschrijving van Mijn Organisatie"),
                    ("en", "Description of My Organization"),
                ]
                .into(),
                category: [("nl", "Categorie"), ("en", "Category")].into(),
                identifier: Some("some-identifier".to_owned()),
                city: Some([("nl", "Den Haag"), ("en", "The Hague")].into()),
                department: Some([("nl", "Afdeling"), ("en", "Department")].into()),
                country_code: "NL".to_owned(),
                web_url: Some(Url::parse("https://organisation.example.com").unwrap()),
                privacy_policy_url: Some(Url::parse("https://organisation.example.com/privacy").unwrap()),
                logo: None,
            }
            .into()
        }
    }
}

#[cfg(test)]
pub mod test {
    use rstest::rstest;
    use serde_json::json;

    use super::*;

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
