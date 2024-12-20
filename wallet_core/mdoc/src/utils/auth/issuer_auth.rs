use serde::Deserialize;
use serde::Serialize;
use serde_with::skip_serializing_none;
use x509_parser::oid_registry::asn1_rs::oid;
use x509_parser::oid_registry::Oid;

use crate::utils::x509::CertificateType;
use crate::utils::x509::MdocCertificateExtension;

use super::Organization;

#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssuerRegistration {
    pub organization: Organization,
}

impl MdocCertificateExtension for IssuerRegistration {
    /// oid: 2.1.123.2
    /// root: {joint-iso-itu-t(2) asn1(1) examples(123)}
    /// suffix: 2, unofficial id for Issuer Authentication
    const OID: Oid<'static> = oid!(2.1.123 .2);
}

impl From<IssuerRegistration> for CertificateType {
    fn from(source: IssuerRegistration) -> Self {
        CertificateType::Mdl(Box::new(source).into())
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
