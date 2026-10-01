use chrono::DateTime;
use chrono::Utc;
use url::Url;

use super::CertificateUsage;

#[derive(Debug, Clone, Default)]
pub struct CertificateConfiguration {
    pub not_before: Option<DateTime<Utc>>,
    pub not_after: Option<DateTime<Utc>>,
    pub exclude_aki: bool,
    pub usage: Option<CertificateUsage>,
    pub crl_distribution_points: Vec<Url>,
}

impl CertificateConfiguration {
    pub fn with_usage(usage: CertificateUsage) -> Self {
        Self {
            usage: Some(usage),
            ..Default::default()
        }
    }
}
