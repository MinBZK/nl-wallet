use derive_more::Constructor;
use derive_more::Display;

/// A distinguished name encoded in a canonical, OID-registry-independent format.
/// This type is specifically designed for database persistence and comparison.
/// Format: "OID1=base64(DER1),OID2=base64(DER2),..."
#[derive(derive_more::Debug, Clone, Eq, PartialEq, Hash, Display, Constructor)]
#[cfg_attr(feature = "persistence", derive(sea_orm::DeriveValueType))]
pub struct DistinguishedName(String);

impl DistinguishedName {
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl AsRef<str> for DistinguishedName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod test {
    use x509_parser::certificate::X509Certificate;

    use crate::server_keys::generate::Ca;
    use crate::x509::BorrowingCertificate;

    #[test]
    fn test_dn_from_certificate() {
        let ca = Ca::generate("myca", Default::default()).unwrap();
        let certificate = BorrowingCertificate::from_certificate_der(ca.certificate().clone())
            .expect("self signed CA should contain a valid X.509 certificate");

        assert_eq!("CN=myca", certificate.distinguished_name().unwrap());
        assert_eq!(
            "2.5.4.3=DARteWNh",
            certificate.distinguished_name_canonical().unwrap().as_ref()
        );

        let x509_cert = certificate.x509_certificate();
        assert_certificate_common_name(x509_cert, &["myca"]);
    }

    fn assert_certificate_common_name(certificate: &X509Certificate, expected_common_name: &[&str]) {
        let actual_common_name = certificate
            .subject
            .iter_common_name()
            .map(|cn| cn.as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(actual_common_name, expected_common_name);
    }
}
