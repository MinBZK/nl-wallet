use chrono::DateTime;
use chrono::Utc;
use url::Url;

use super::CertificateUsage;
use super::SubjectAltNameUri;

#[derive(Debug, Clone, Default)]
pub struct CertificateConfiguration {
    pub not_before: Option<DateTime<Utc>>,
    pub not_after: Option<DateTime<Utc>>,
    pub exclude_aki: bool,
    pub usage: Option<CertificateUsage>,
    pub crl_distribution_points: Vec<Url>,
    pub subject_alt_names: Vec<SubjectAltNameUri>,
}

impl CertificateConfiguration {
    pub fn with_usage(usage: CertificateUsage) -> Self {
        Self {
            usage: Some(usage),
            ..Default::default()
        }
    }
}
