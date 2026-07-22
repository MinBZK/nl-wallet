use tracing::warn;

use super::image::Image;
use super::localize::LocalizedString;
use super::localize::LocalizedStrings;

pub struct Organization {
    pub legal_name: String,
    pub display_name: String,
    pub description: Vec<LocalizedString>,
    pub image: Option<Image>,
    pub web_url: Option<String>,
    pub privacy_policy_url: Option<String>,
    pub identifier: String,
    pub city: Option<Vec<LocalizedString>>,
    pub category: Vec<LocalizedString>,
    pub department: Option<Vec<LocalizedString>>,
    pub country_code: String,
}

impl From<wallet::attestation_data::Organization> for Organization {
    fn from(value: wallet::attestation_data::Organization) -> Self {
        Organization {
            legal_name: value.legal_name,
            display_name: value.display_name,
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
