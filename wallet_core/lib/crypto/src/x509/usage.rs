use derive_more::Display;
use x509_parser::certificate::X509Certificate;
use x509_parser::der_parser::Oid;
use x509_parser::der_parser::oid;
use x509_parser::error::X509Error;
use x509_parser::extensions::ExtendedKeyUsage;

/// Usage of a [`Certificate`], representing its Extended Key Usage (EKU).
/// [`Certificate::verify()`] receives this as parameter and enforces that it is present in the certificate
/// being verified.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum CertificateUsage {
    Mdl,
    OAuthStatusSigning,
    Wia,
}

const EXTENDED_KEY_USAGE_MDL: &Oid = &oid!(1.0.18013.5.1.2);
// The .127 is made up, the real child node is TDB
const EXTENDED_KEY_USAGE_TSL: &Oid = &oid!(1.3.6.1.5.5.7.3.127);
// The .128 is made up, the real child node is TBD
const EXTENDED_KEY_USAGE_WIA: &Oid = &oid!(1.3.6.1.5.5.7.3.128);

#[derive(thiserror::Error, Debug)]
pub enum CertificateUsageError {
    #[error("X509 coding error: {0}")]
    X509Error(#[from] X509Error),

    #[error("no extended key usage section in certificate")]
    NoExtendedKeyUsage,

    #[error("no known usage found")]
    NoKnownUsageFound,

    #[error("multiple usages found for same certificate: {0} and {1}")]
    MultipleUsages(CertificateUsage, CertificateUsage),
}

impl CertificateUsage {
    pub fn from_certificate(cert: &X509Certificate) -> Result<Self, CertificateUsageError> {
        let eku_extension = cert
            .extended_key_usage()?
            .ok_or(CertificateUsageError::NoExtendedKeyUsage)?;
        Self::from_key_usage(eku_extension.value)
    }

    fn from_key_usage(ext_key_usage: &ExtendedKeyUsage) -> Result<Self, CertificateUsageError> {
        let mut result = None;
        for key_usage_oid in &ext_key_usage.other {
            let cert_usage = {
                // Unfortunately we cannot use a match statement here.
                if key_usage_oid == EXTENDED_KEY_USAGE_MDL {
                    Some(Self::Mdl)
                } else if key_usage_oid == EXTENDED_KEY_USAGE_TSL {
                    Some(Self::OAuthStatusSigning)
                } else if key_usage_oid == EXTENDED_KEY_USAGE_WIA {
                    Some(Self::Wia)
                } else {
                    None
                }
            };
            match (result, cert_usage) {
                (Some(previous), Some(current)) => {
                    return Err(CertificateUsageError::MultipleUsages(previous, current));
                }
                (None, current @ Some(_)) => result = current,
                _ => {}
            }
        }
        result.ok_or(CertificateUsageError::NoKnownUsageFound)
    }

    fn as_oid(self) -> &'static Oid<'static> {
        match self {
            CertificateUsage::Mdl => EXTENDED_KEY_USAGE_MDL,
            CertificateUsage::OAuthStatusSigning => EXTENDED_KEY_USAGE_TSL,
            CertificateUsage::Wia => EXTENDED_KEY_USAGE_WIA,
        }
    }

    pub fn as_oid_bytes(&self) -> &'static [u8] {
        self.as_oid().as_bytes()
    }

    #[cfg(any(test, feature = "generate"))]
    pub fn to_key_usage_purpose(&self) -> rcgen::ExtendedKeyUsagePurpose {
        rcgen::ExtendedKeyUsagePurpose::Other(
            self.as_oid()
                .iter()
                .expect("oid sub identifier does not fit in u64")
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use rstest::rstest;
    use x509_parser::asn1_rs::FromDer;

    use super::*;

    fn create_der_seq_of_oid_bytes(oid_bytes: &[&[u8]]) -> Vec<u8> {
        let mut der_bytes = Vec::with_capacity(128);
        // Write DER sequence of OIDs (assuming length fits in single byte)
        der_bytes.extend_from_slice(&[0x30, 0]);
        for bytes in oid_bytes {
            der_bytes.extend_from_slice(&[0x06, bytes.len() as u8]);
            der_bytes.extend_from_slice(bytes);
        }
        assert!(der_bytes.len() < 0x80, "arguments are too large in bytes");
        der_bytes[1] = der_bytes.len() as u8 - 2;
        der_bytes
    }

    #[rstest]
    fn certificate_usage_to_oid_from_extension(
        #[values(CertificateUsage::Mdl, CertificateUsage::OAuthStatusSigning, CertificateUsage::Wia)]
        cert_usage: CertificateUsage,
    ) {
        let oid_bytes = cert_usage.as_oid_bytes();

        let ext_bytes = create_der_seq_of_oid_bytes(&[oid_bytes]);
        let (_, extended_key_usage) = ExtendedKeyUsage::from_der(&ext_bytes).unwrap();

        let parsed = CertificateUsage::from_key_usage(&extended_key_usage).unwrap();
        assert_eq!(cert_usage, parsed);
    }

    #[test]
    fn check_for_multiple_key_usages() {
        let ext_bytes =
            create_der_seq_of_oid_bytes(&[EXTENDED_KEY_USAGE_MDL.as_bytes(), EXTENDED_KEY_USAGE_TSL.as_bytes()]);
        let (_, extended_key_usage) = ExtendedKeyUsage::from_der(&ext_bytes).unwrap();

        let result = CertificateUsage::from_key_usage(&extended_key_usage);
        assert!(result.is_err());
        assert_matches!(result, Err(CertificateUsageError::MultipleUsages(a, b))
            if a == CertificateUsage::Mdl && b == CertificateUsage::OAuthStatusSigning);
    }

    #[test]
    fn check_for_no_known_usage() {
        let ext_bytes = create_der_seq_of_oid_bytes(&[Oid::from(&[1, 2, 3]).unwrap().as_bytes()]);
        let (_, extended_key_usage) = ExtendedKeyUsage::from_der(&ext_bytes).unwrap();

        let result = CertificateUsage::from_key_usage(&extended_key_usage);
        assert!(result.is_err());
        assert_matches!(result, Err(CertificateUsageError::NoKnownUsageFound))
    }
}
