use super::localize::LocalizedString;

pub struct ServiceDescription {
    pub translations: Vec<LocalizedString>,
}

impl From<wallet::attestation_data::ServiceDescription> for ServiceDescription {
    fn from(value: wallet::attestation_data::ServiceDescription) -> Self {
        Self {
            translations: value
                .translations
                .into_iter()
                .map(|translation| LocalizedString {
                    language: translation.lang,
                    value: translation.value,
                })
                .collect(),
        }
    }
}

pub struct Organization {
    pub legal_name: String,
    pub display_name: String,
    pub service_description: Vec<ServiceDescription>,
    pub web_url: Option<String>,
    pub privacy_policy_url: Option<String>,
    pub identifier: String,
    pub country_code: String,
}

impl From<wallet::attestation_data::Organization> for Organization {
    fn from(value: wallet::attestation_data::Organization) -> Self {
        Organization {
            legal_name: value.legal_name,
            display_name: value.display_name,
            service_description: value.description.into_iter().map(Into::into).collect(),
            identifier: value.identifier,
            country_code: value.country_code,
            web_url: value.web_url.map(|url| url.to_string()),
            privacy_policy_url: value.privacy_policy_url.map(|url| url.to_string()),
        }
    }
}
