use crypto::x509::BorrowingCertificateExtension;
use serde::Deserialize;
use serde::Serialize;
use serde_with::skip_serializing_none;
use x509_parser::oid_registry::Oid;
use x509_parser::oid_registry::asn1_rs::oid;

use crate::auth::Organization;
use crate::x509::CertificateType;

#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssuerRegistration {
    pub organization: Box<Organization>,
}

impl IssuerRegistration {
    #[cfg(feature = "mock")]
    pub fn to_certificate_configuration(
        &self,
    ) -> Result<crypto::x509::CertificateConfiguration, crypto::x509::CertificateError> {
        let custom_ext = self.to_custom_ext()?;
        Ok(crypto::x509::CertificateConfiguration::with_usage_and_extension(
            crypto::x509::CertificateUsage::Mdl,
            custom_ext,
        ))
    }
}

impl BorrowingCertificateExtension for IssuerRegistration {
    /// oid: 2.1.123.2
    /// root: {joint-iso-itu-t(2) asn1(1) examples(123)}
    /// suffix: 2, unofficial id for Issuer Authentication
    #[rustfmt::skip]
    const OID: Oid<'static> = oid!(2.1.123.2);
}

impl From<IssuerRegistration> for CertificateType {
    fn from(source: IssuerRegistration) -> Self {
        CertificateType::Mdl(source)
    }
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use super::*;

    impl IssuerRegistration {
        pub fn new_mock() -> Self {
            let organization = Organization::new_mock();

            IssuerRegistration { organization }
        }
    }
}
