use chrono::DateTime;
use chrono::Utc;

use super::CertificateUsage;

#[derive(Debug, Clone, Default)]
pub struct CertificateConfiguration {
    pub not_before: Option<DateTime<Utc>>,
    pub not_after: Option<DateTime<Utc>>,
    pub exclude_aki: bool,
    pub usage: Option<CertificateUsage>,
    /// TODO: PVW-5885 PVW-5895 Remove when ReaderRegistration and IssuerRegistration are removed
    pub extension: Option<rcgen::CustomExtension>,
}

impl CertificateConfiguration {
    pub fn with_usage(usage: CertificateUsage) -> Self {
        Self {
            usage: Some(usage),
            ..Default::default()
        }
    }

    pub fn with_usage_and_extension(usage: CertificateUsage, extension: rcgen::CustomExtension) -> Self {
        Self {
            usage: Some(usage),
            extension: Some(extension),
            ..Default::default()
        }
    }
}
