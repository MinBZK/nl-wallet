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
    StatusListSigning,
    Wia,
    KeyAttestation,
}

const EXTENDED_KEY_USAGE_MDL: &Oid = &oid!(1.0.18013.5.1.2);
// The .127 is made up, the real child node is TDB
const EXTENDED_KEY_USAGE_TSL: &Oid = &oid!(1.3.6.1.5.5.7.3.127);
// The .128 is made up, the real child node is TBD
const EXTENDED_KEY_USAGE_WIA: &Oid = &oid!(1.3.6.1.5.5.7.3.128);
// The .129 is made up, the real child node is TBD
const EXTENDED_KEY_USAGE_KEY_ATTESTATION: &Oid = &oid!(1.3.6.1.5.5.7.3.129);

/// The DER encoded content of the certificate policies extension (RFC 5280, section 4.2.1.4) of wallet provider
/// certificates, which contains a single policy without qualifiers:
///
/// ```text
/// SEQUENCE {                    -- CertificatePolicies
///   SEQUENCE {                  -- PolicyInformation
///     OBJECT IDENTIFIER 2.25.150580842064059311066423755779195795505
///   }
/// }
/// ```
///
/// The policy identifier is a temporary UUID OID (ITU-T X.667) for UUID `7148d0b6-12d3-4c0e-af93-f5d209a3f831`.
/// We encode it ourselves, because the `der` crate only supports OID arcs of up to 32 bits.
#[cfg(any(test, feature = "generate"))]
const WALLET_PROVIDER_CERTIFICATE_POLICIES: &[u8] = &[
    0x30, 0x18, // SEQUENCE
    0x30, 0x16, // SEQUENCE
    0x06, 0x14, // OBJECT IDENTIFIER
    0x69, 0x81, 0xe2, 0xc8, 0xe8, 0xad, 0xc2, 0xad, 0x9a, 0xb0, 0x9d, 0xaf, 0xc9, 0xfd, 0xba, 0xa0, 0xcd, 0x8f, 0xf0,
    0x31,
];

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
                match key_usage_oid {
                    oid if oid == EXTENDED_KEY_USAGE_MDL => Some(Self::Mdl),
                    oid if oid == EXTENDED_KEY_USAGE_TSL => Some(Self::StatusListSigning),
                    oid if oid == EXTENDED_KEY_USAGE_WIA => Some(Self::Wia),
                    oid if oid == EXTENDED_KEY_USAGE_KEY_ATTESTATION => Some(Self::KeyAttestation),
                    _ => None,
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
            CertificateUsage::StatusListSigning => EXTENDED_KEY_USAGE_TSL,
            CertificateUsage::Wia => EXTENDED_KEY_USAGE_WIA,
            CertificateUsage::KeyAttestation => EXTENDED_KEY_USAGE_KEY_ATTESTATION,
        }
    }

    pub fn as_oid_bytes(&self) -> &'static [u8] {
        self.as_oid().as_bytes()
    }

    #[cfg(any(test, feature = "generate"))]
    pub fn key_usages(&self) -> Vec<rcgen::KeyUsagePurpose> {
        match self {
            // ISO/IEC 18013-5:2021, Table B.3 requires digitalSignature only for mdoc signers.
            // This issuer profile is also used for SD-JWT signing.
            Self::Mdl => vec![rcgen::KeyUsagePurpose::DigitalSignature],
            // ETSI EN 319 412-2, section 4.3.2 requires a key usage; type C is digitalSignature only.
            Self::StatusListSigning | Self::Wia | Self::KeyAttestation => {
                vec![rcgen::KeyUsagePurpose::DigitalSignature]
            }
        }
    }

    /// The DER encoded content of the certificate policies extension, see [`WALLET_PROVIDER_CERTIFICATE_POLICIES`].
    ///
    /// ETSI EN 319 412-2, GEN-4.3.3-2 requires the identifier of at least one certificate policy, which applies to
    /// wallet provider certificates through ETSI TS 119 412-6, WAL-5.1-01 and PID-4.1-01.
    #[cfg(any(test, feature = "generate"))]
    pub fn certificate_policies(&self) -> Option<&'static [u8]> {
        match self {
            Self::Wia | Self::KeyAttestation => Some(WALLET_PROVIDER_CERTIFICATE_POLICIES),
            Self::Mdl | Self::StatusListSigning => None,
        }
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
        #[values(
            CertificateUsage::Mdl,
            CertificateUsage::StatusListSigning,
            CertificateUsage::Wia,
            CertificateUsage::KeyAttestation
        )]
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
            if a == CertificateUsage::Mdl && b == CertificateUsage::StatusListSigning);
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
