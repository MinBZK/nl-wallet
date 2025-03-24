#[cfg(any(test, feature = "generate"))]
pub mod generate {
    use p256::pkcs8::der::asn1::SequenceOf;
    use p256::pkcs8::der::Encode;
    use p256::pkcs8::ObjectIdentifier;
    use rcgen::CustomExtension;

    use crypto::x509::BorrowingCertificateExtension;
    use crypto::x509::CertificateError;

    use crate::utils::x509::CertificateType;
    use crate::utils::x509::CertificateUsage;

    impl From<CertificateUsage> for CustomExtension {
        fn from(value: CertificateUsage) -> Self {
            const OID_EXT_KEY_USAGE: &[u64] = &[2, 5, 29, 37];

            // The spec requires that we add mdoc-specific OIDs to the extended key usage extension, but
            // [`CertificateParams`] only supports a whitelist of key usages that it is aware of. So we
            // DER-serialize it manually and add it to the custom extensions.
            // We unwrap in these functions because they have fixed input for which they always succeed.
            let mut seq = SequenceOf::<ObjectIdentifier, 1>::new();
            seq.add(ObjectIdentifier::from_bytes(value.eku()).unwrap()).unwrap();
            let mut ext = CustomExtension::from_oid_content(OID_EXT_KEY_USAGE, seq.to_der().unwrap());
            ext.set_criticality(true);
            ext
        }
    }

    impl TryFrom<CertificateType> for Vec<CustomExtension> {
        type Error = CertificateError;

        fn try_from(source: CertificateType) -> Result<Vec<CustomExtension>, CertificateError> {
            let usage = CertificateUsage::from(&source);
            let mut extensions = vec![usage.into()];

            match source {
                CertificateType::ReaderAuth(Some(reader_registration)) => {
                    let ext_reader_auth = reader_registration.to_custom_ext()?;
                    extensions.push(ext_reader_auth);
                }
                CertificateType::Mdl(Some(issuer_registration)) => {
                    let ext_issuer_auth = issuer_registration.to_custom_ext()?;
                    extensions.push(ext_issuer_auth);
                }
                _ => {}
            };
            Ok(extensions)
        }
    }

    #[cfg(any(test, feature = "mock"))]
    pub mod mock {
        use crypto::server_keys::generate::mock::ISSUANCE_CERT_CN;
        use crypto::server_keys::generate::mock::RP_CERT_CN;
        use crypto::server_keys::generate::Ca;
        use crypto::server_keys::KeyPair;

        use crate::utils::issuer_auth::IssuerRegistration;
        use crate::utils::reader_auth::ReaderRegistration;

        use super::*;

        pub fn generate_issuer_mock(
            ca: &Ca,
            issuer_registration: Option<IssuerRegistration>,
        ) -> Result<KeyPair, CertificateError> {
            ca.generate_key_pair(
                ISSUANCE_CERT_CN,
                CertificateType::Mdl(issuer_registration.map(Box::new)),
                Default::default(),
            )
        }

        pub fn generate_reader_mock(
            ca: &Ca,
            reader_registration: Option<ReaderRegistration>,
        ) -> Result<KeyPair, CertificateError> {
            ca.generate_key_pair(
                RP_CERT_CN,
                CertificateType::ReaderAuth(reader_registration.map(Box::new)),
                Default::default(),
            )
        }
    }
}
