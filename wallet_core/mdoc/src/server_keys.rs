use derive_more::Debug;
use p256::ecdsa::Signature;
use p256::ecdsa::SigningKey;

use wallet_common::keys::EcdsaKey;
use wallet_common::keys::EcdsaKeySend;

use crate::utils::x509::BorrowingCertificate;
use crate::utils::x509::CertificateError;

#[derive(Debug)]
pub struct KeyPair<S = SigningKey> {
    #[debug(skip)]
    private_key: S,
    certificate: BorrowingCertificate,
}

impl KeyPair {
    pub fn new_from_signing_key(
        private_key: SigningKey,
        certificate: BorrowingCertificate,
    ) -> Result<KeyPair, CertificateError> {
        if certificate.public_key() != private_key.verifying_key() {
            return Err(CertificateError::KeyMismatch);
        }

        Ok(KeyPair {
            private_key,
            certificate,
        })
    }
}

impl<S: EcdsaKey> KeyPair<S> {
    pub async fn new(private_key: S, certificate: BorrowingCertificate) -> Result<KeyPair<S>, CertificateError> {
        if certificate.public_key()
            != &private_key
                .verifying_key()
                .await
                .map_err(|e| CertificateError::PublicKeyFromPrivate(Box::new(e)))?
        {
            return Err(CertificateError::KeyMismatch);
        }

        Ok(KeyPair {
            private_key,
            certificate,
        })
    }
}

impl<S> KeyPair<S> {
    pub fn private_key(&self) -> &S {
        &self.private_key
    }

    pub fn certificate(&self) -> &BorrowingCertificate {
        &self.certificate
    }
}

impl<S> From<KeyPair<S>> for BorrowingCertificate {
    fn from(source: KeyPair<S>) -> BorrowingCertificate {
        source.certificate
    }
}

impl<S: EcdsaKey> EcdsaKey for KeyPair<S> {
    type Error = S::Error;

    async fn verifying_key(&self) -> std::result::Result<p256::ecdsa::VerifyingKey, Self::Error> {
        self.private_key.verifying_key().await
    }

    async fn try_sign(&self, msg: &[u8]) -> std::result::Result<Signature, Self::Error> {
        self.private_key.try_sign(msg).await
    }
}

pub trait KeyRing {
    type Key: EcdsaKeySend;

    fn key_pair(&self, id: &str) -> Option<&KeyPair<Self::Key>>;
}

#[cfg(any(test, feature = "test"))]
pub mod test {
    use p256::ecdsa::SigningKey;

    use super::KeyPair;
    use super::KeyRing;

    /// An implementation of [`KeyRing`] containing a single key.
    pub struct SingleKeyRing(pub KeyPair<SigningKey>);

    impl KeyRing for SingleKeyRing {
        type Key = SigningKey;

        fn key_pair(&self, _: &str) -> Option<&KeyPair<SigningKey>> {
            Some(&self.0)
        }
    }
}

#[cfg(any(test, feature = "generate"))]
pub mod generate {
    use p256::ecdsa::SigningKey;
    use p256::pkcs8::der::asn1::SequenceOf;
    use p256::pkcs8::der::Encode;
    use p256::pkcs8::DecodePrivateKey;
    use p256::pkcs8::ObjectIdentifier;
    use rcgen::BasicConstraints;
    use rcgen::CertificateParams;
    use rcgen::CustomExtension;
    use rcgen::DnType;
    use rcgen::IsCa;
    use rcgen::SanType;
    use rcgen::PKCS_ECDSA_P256_SHA256;
    use rustls_pki_types::CertificateDer;
    use rustls_pki_types::TrustAnchor;
    use time::OffsetDateTime;

    use wallet_common::trust_anchor::BorrowingTrustAnchor;
    use x509_parser::prelude::FromDer;
    use x509_parser::prelude::X509Certificate;

    use crate::server_keys::KeyPair;
    use crate::utils::x509::BorrowingCertificate;
    use crate::utils::x509::CertificateConfiguration;
    use crate::utils::x509::CertificateError;
    use crate::utils::x509::CertificateType;
    use crate::utils::x509::CertificateUsage;
    use crate::utils::x509::MdocCertificateExtension;

    fn rcgen_cert_privkey(keypair: &rcgen::KeyPair) -> Result<SigningKey, CertificateError> {
        SigningKey::from_pkcs8_der(keypair.serialized_der()).map_err(CertificateError::GeneratingPrivateKey)
    }

    pub struct Ca {
        certificate: rcgen::Certificate,
        key_pair: rcgen::KeyPair,
        borrowing_trust_anchor: BorrowingTrustAnchor,
    }

    impl Ca {
        fn new(certificate: rcgen::Certificate, key_pair: rcgen::KeyPair) -> Result<Self, CertificateError> {
            let borrowing_trust_anchor = BorrowingTrustAnchor::from_der(certificate.der().as_ref())?;

            let key_pair_ca = Self {
                certificate,
                key_pair,
                borrowing_trust_anchor,
            };

            Ok(key_pair_ca)
        }

        /// Generate a new self-signed CA key pair, constrained to the specified number of intermediates CAs.
        pub fn generate_with_intermediate_count(
            common_name: &str,
            configuration: CertificateConfiguration,
            intermediate_count: u8,
        ) -> Result<Self, CertificateError> {
            let mut params = CertificateParams::from(configuration);
            params.is_ca = IsCa::Ca(BasicConstraints::Constrained(intermediate_count));
            params.distinguished_name.push(DnType::CommonName, common_name);

            let key_pair = rcgen::KeyPair::generate()?;
            let certificate = params.self_signed(&key_pair)?;

            Self::new(certificate, key_pair)
        }

        /// Generate a new self-signed CA key pair, constrained to having no intermediate CAs.
        pub fn generate(common_name: &str, configuration: CertificateConfiguration) -> Result<Self, CertificateError> {
            Self::generate_with_intermediate_count(common_name, configuration, 0)
        }

        pub fn from_der(
            certificate_der: impl AsRef<[u8]>,
            signing_key_der: impl AsRef<[u8]>,
        ) -> Result<Self, CertificateError> {
            let (_, x509_certificate) = X509Certificate::from_der(certificate_der.as_ref())?;

            // Check if the parsed certificate is actually a root CA.
            if !x509_certificate.is_ca() || x509_certificate.issuer() != x509_certificate.subject() {
                return Err(CertificateError::NotRootCa);
            }

            let params = CertificateParams::from_ca_cert_der(&certificate_der.as_ref().into())?;
            let key_pair = rcgen::KeyPair::from_pkcs8_der_and_sign_algo(
                &signing_key_der.as_ref().into(),
                &PKCS_ECDSA_P256_SHA256,
            )?;
            let certificate = params.self_signed(&key_pair)?;

            Self::new(certificate, key_pair)
        }

        pub fn as_certificate_der(&self) -> &CertificateDer<'static> {
            self.certificate.der()
        }

        pub fn as_borrowing_trust_anchor(&self) -> &BorrowingTrustAnchor {
            &self.borrowing_trust_anchor
        }

        pub fn to_signing_key(&self) -> Result<SigningKey, CertificateError> {
            rcgen_cert_privkey(&self.key_pair)
        }

        pub fn to_trust_anchor(&self) -> TrustAnchor {
            self.borrowing_trust_anchor.as_trust_anchor().clone()
        }

        /// Generate a new intermediate CA key pair, with any constraint
        /// on the amount of intermediates it can have decremented by one.
        pub fn generate_intermediate(
            &self,
            common_name: &str,
            certificate_usage: CertificateUsage,
            configuration: CertificateConfiguration,
        ) -> Result<Self, CertificateError> {
            let constraint = match self.certificate.params().is_ca {
                IsCa::Ca(BasicConstraints::Unconstrained) => BasicConstraints::Unconstrained,
                IsCa::Ca(BasicConstraints::Constrained(count)) if count > 0 => BasicConstraints::Constrained(count - 1),
                _ => return Err(CertificateError::BasicConstraintViolation),
            };

            let mut params = CertificateParams::from(configuration);
            params.is_ca = IsCa::Ca(constraint);
            params.distinguished_name.push(DnType::CommonName, common_name);
            params.custom_extensions.push(certificate_usage.to_custom_ext());

            let key_pair = rcgen::KeyPair::generate()?;
            let certificate = params.signed_by(&key_pair, &self.certificate, &self.key_pair)?;

            Self::new(certificate, key_pair)
        }

        /// Generate a new key pair signed with the specified CA.
        pub fn generate_key_pair(
            &self,
            common_name: &str,
            certificate_type: &CertificateType,
            configuration: CertificateConfiguration,
        ) -> Result<KeyPair, CertificateError> {
            let mut params = CertificateParams::from(configuration);
            params.is_ca = IsCa::NoCa;
            params.distinguished_name.push(DnType::CommonName, common_name);
            params.subject_alt_names.push(SanType::DnsName(common_name.try_into()?));
            params.custom_extensions.extend(certificate_type.to_custom_exts()?);

            let key_pair = rcgen::KeyPair::generate()?;
            let certificate = params.signed_by(&key_pair, &self.certificate, &self.key_pair)?;

            let private_key = rcgen_cert_privkey(&key_pair)?;
            let certificate = BorrowingCertificate::from_certificate_der(certificate.into())?;

            let key_pair = KeyPair {
                private_key,
                certificate,
            };

            Ok(key_pair)
        }
    }

    impl From<CertificateConfiguration> for CertificateParams {
        fn from(source: CertificateConfiguration) -> Self {
            let mut result = CertificateParams::default();
            if let Some(not_before) = source.not_before.and_then(|ts| ts.timestamp_nanos_opt()) {
                result.not_before = OffsetDateTime::from_unix_timestamp_nanos(not_before as i128).unwrap();
            }
            if let Some(not_after) = source.not_after.and_then(|ts| ts.timestamp_nanos_opt()) {
                result.not_after = OffsetDateTime::from_unix_timestamp_nanos(not_after as i128).unwrap();
            }
            result
        }
    }

    impl CertificateUsage {
        fn to_custom_ext(self) -> CustomExtension {
            const OID_EXT_KEY_USAGE: &[u64] = &[2, 5, 29, 37];

            // The spec requires that we add mdoc-specific OIDs to the extended key usage extension, but
            // [`CertificateParams`] only supports a whitelist of key usages that it is aware of. So we
            // DER-serialize it manually and add it to the custom extensions.
            // We unwrap in these functions because they have fixed input for which they always succeed.
            let mut seq = SequenceOf::<ObjectIdentifier, 1>::new();
            seq.add(ObjectIdentifier::from_bytes(self.eku()).unwrap()).unwrap();
            let mut ext = CustomExtension::from_oid_content(OID_EXT_KEY_USAGE, seq.to_der().unwrap());
            ext.set_criticality(true);
            ext
        }
    }

    impl CertificateType {
        fn to_custom_exts(&self) -> Result<Vec<CustomExtension>, CertificateError> {
            let usage: CertificateUsage = self.into();
            let mut extensions = vec![usage.to_custom_ext()];

            match self {
                Self::ReaderAuth(Some(reader_registration)) => {
                    let ext_reader_auth = reader_registration.to_custom_ext()?;
                    extensions.push(ext_reader_auth);
                }
                Self::Mdl(Some(issuer_registration)) => {
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
        use crate::server_keys::KeyPair;
        use crate::utils::issuer_auth::IssuerRegistration;
        use crate::utils::reader_auth::ReaderRegistration;

        use super::*;

        const ISSUANCE_CA_CN: &str = "ca.issuer.example.com";
        pub const ISSUANCE_CERT_CN: &str = "cert.issuer.example.com";

        const RP_CA_CN: &str = "ca.rp.example.com";
        const RP_CERT_CN: &str = "cert.rp.example.com";

        impl Ca {
            pub fn generate_issuer_mock_ca() -> Result<Self, CertificateError> {
                Self::generate(ISSUANCE_CA_CN, Default::default())
            }

            pub fn generate_reader_mock_ca() -> Result<Self, CertificateError> {
                Self::generate(RP_CA_CN, Default::default())
            }

            pub fn generate_issuer_mock(
                &self,
                issuer_registration: Option<IssuerRegistration>,
            ) -> Result<KeyPair, CertificateError> {
                self.generate_key_pair(
                    ISSUANCE_CERT_CN,
                    &CertificateType::Mdl(issuer_registration.map(Box::new)),
                    Default::default(),
                )
            }

            pub fn generate_reader_mock(
                &self,
                reader_registration: Option<ReaderRegistration>,
            ) -> Result<KeyPair, CertificateError> {
                self.generate_key_pair(
                    RP_CERT_CN,
                    &CertificateType::ReaderAuth(reader_registration.map(Box::new)),
                    Default::default(),
                )
            }
        }
    }
}
