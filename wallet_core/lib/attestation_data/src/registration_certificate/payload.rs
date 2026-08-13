use chrono::DateTime;
use chrono::Utc;
use dcql::ClaimsQuery;
use dcql::CredentialQueryFormat;
use serde::Deserialize;
use serde::Deserializer;
use url::Url;
use utils::vec_at_least::VecNonEmpty;

use super::status::RegistrationCertificateStatus;

#[derive(Deserialize)]
pub struct MultiLanguageString {
    pub lang: String,
    #[serde(alias = "content")]
    pub value: String,
}

pub type MultiLanguageStringSet = VecNonEmpty<MultiLanguageString>;

#[derive(Deserialize)]
#[serde(untagged)]
enum ServiceDescriptionsWireFormat {
    Nested(VecNonEmpty<MultiLanguageStringSet>),
    Flat(MultiLanguageStringSet),
}

/// Localized descriptions of one or more services.
///
/// ETSI TS 119 475 Annex B models this as an array of arrays, while the validation table in PVW-5867 describes a
/// single array of objects. Both forms are accepted and normalized to the nested ETSI representation.
pub struct ServiceDescriptions(pub VecNonEmpty<MultiLanguageStringSet>);

impl<'de> Deserialize<'de> for ServiceDescriptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let descriptions = ServiceDescriptionsWireFormat::deserialize(deserializer)?;
        Ok(Self(match descriptions {
            ServiceDescriptionsWireFormat::Nested(descriptions) => descriptions,
            ServiceDescriptionsWireFormat::Flat(description) => utils::vec_nonempty![description],
        }))
    }
}

#[derive(Deserialize)]
pub struct Credential {
    #[serde(flatten)]
    pub format: CredentialQueryFormat,
    pub claim: Option<Vec<ClaimsQuery>>,
}

#[derive(Deserialize)]
pub struct SupervisoryAuthority {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub uri: Option<Url>,
}

/// Intermediary data is parsed so it is not discarded, but its WRPAC binding is deliberately left to PVW-6062.
#[derive(Deserialize)]
pub struct Intermediary {
    pub sub: String,
    #[serde(alias = "name")]
    pub sname: Option<String>,
}

#[derive(Deserialize)]
pub struct UncheckedRegistrationCertificate {
    /// Optional at parse time because the informative Annex C example omits it. Validation always requires it.
    pub id: Option<String>,
    pub name: Option<String>,
    pub sub: String,
    pub sub_ln: Option<String>,
    pub sub_gn: Option<String>,
    pub sub_fn: Option<String>,
    pub country: String,
    pub registry_uri: Url,
    pub support_uri: String,
    pub info_uri: Option<Url>,
    pub privacy_policy: Option<Url>,
    pub srv_description: ServiceDescriptions,
    pub supervisory_authority: SupervisoryAuthority,
    pub entitlements: VecNonEmpty<Url>,
    pub credentials: Option<Vec<Credential>>,
    pub purpose: Option<MultiLanguageStringSet>,
    pub intended_use_id: Option<String>,
    pub provides_attestations: Option<Vec<Credential>>,
    pub public_body: Option<bool>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub iat: DateTime<Utc>,
    #[serde(default, with = "chrono::serde::ts_seconds_option")]
    pub exp: Option<DateTime<Utc>>,
    pub status: RegistrationCertificateStatus,
    pub policy_id: Vec<String>,
    pub certificate_policy: Url,
    pub intermediary: Option<Intermediary>,
}

impl jwt::JwtTyp for UncheckedRegistrationCertificate {
    const TYP: &'static str = jwt::jades_b_b::JADES_B_B_JWT_TYP;
}
