use chrono::DateTime;
use chrono::Utc;
use url::Url;

use super::CertificateProfile;
use super::SubjectAltNameUri;

#[derive(Debug, Clone, Default)]
pub struct CertificateConfiguration {
    pub not_before: Option<DateTime<Utc>>,
    pub not_after: Option<DateTime<Utc>>,
    pub exclude_aki: bool,
    pub profile: Option<CertificateProfile>,
    pub crl_distribution_points: Vec<Url>,
    /// Locations of the certificate of the issuing CA, which are included in the authority information access
    /// extension of end-entity certificates.
    pub ca_issuers: Vec<Url>,
    pub subject_alt_names: Vec<SubjectAltNameUri>,
}

impl CertificateConfiguration {
    pub fn with_profile(profile: CertificateProfile) -> Self {
        Self {
            profile: Some(profile),
            ..Default::default()
        }
    }
}
