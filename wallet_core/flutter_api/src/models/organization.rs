use tracing::warn;

use super::image::Image;
use super::localize::LocalizedString;
use super::localize::LocalizedStrings;

pub struct Organization {
    pub legal_name: Vec<LocalizedString>,
    pub display_name: Vec<LocalizedString>,
    pub description: Vec<LocalizedString>,
    pub image: Option<Image>,
    pub web_url: Option<String>,
    pub privacy_policy_url: Option<String>,
    pub identifier: Option<String>,
    pub city: Option<Vec<LocalizedString>>,
    pub category: Vec<LocalizedString>,
    pub department: Option<Vec<LocalizedString>>,
    pub country_code: Option<String>,
}

impl From<Box<wallet::attestation_data::Organization>> for Organization {
    fn from(value: Box<wallet::attestation_data::Organization>) -> Self {
        Organization {
            legal_name: LocalizedStrings(value.legal_name).into(),
            display_name: LocalizedStrings(value.display_name).into(),
            description: LocalizedStrings(value.description).into(),
            image: value.logo.and_then(|l| {
                Image::try_from(l)
                    .inspect_err(|e| warn!("error converting logo, not showing: {e}"))
                    .ok()
            }),
            identifier: value.identifier,
            city: value.city.map(|city| LocalizedStrings(city).into()),
            category: LocalizedStrings(value.category).into(),
            department: value.department.map(|department| LocalizedStrings(department).into()),
            country_code: value.country_code,
            web_url: value.web_url.map(|url| url.to_string()),
            privacy_policy_url: value.privacy_policy_url.map(|url| url.to_string()),
        }
    }
}
