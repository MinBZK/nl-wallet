use p256::ecdsa::Signature;
use p256::ecdsa::SigningKey;

use wallet_common::keys::EcdsaKey;
use wallet_common::keys::EcdsaKeySend;

use crate::utils::x509::BorrowingCertificate;
use crate::utils::x509::CertificateError;

#[derive(Debug)]
pub struct KeyPair<S = SigningKey> {
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
mod generate {
    use p256::ecdsa::SigningKey;
    use p256::pkcs8::der::asn1::SequenceOf;
    use p256::pkcs8::der::Encode;
    use p256::pkcs8::DecodePrivateKey;
    use p256::pkcs8::EncodePrivateKey;
    use p256::pkcs8::ObjectIdentifier;
    use rcgen::BasicConstraints;
    use rcgen::CertificateParams;
    use rcgen::CustomExtension;
    use rcgen::DnType;
    use rcgen::IsCa;
    use rcgen::SanType;
    use rcgen::PKCS_ECDSA_P256_SHA256;
    use time::OffsetDateTime;

    use crate::server_keys::KeyPair;
    use crate::utils::x509::BorrowingCertificate;
    use crate::utils::x509::CertificateConfiguration;
    use crate::utils::x509::CertificateError;
    use crate::utils::x509::CertificateType;
    use crate::utils::x509::CertificateUsage;
    use crate::utils::x509::MdocCertificateExtension;

    impl KeyPair {
        /// Generate a new self-signed CA key pair.
        pub fn generate_ca(
            common_name: &str,
            configuration: CertificateConfiguration,
        ) -> Result<Self, CertificateError> {
            let mut ca_params = CertificateParams::from(configuration);
            ca_params.is_ca = IsCa::Ca(BasicConstraints::Constrained(0));
            ca_params.distinguished_name.push(DnType::CommonName, common_name);
            let key_pair = rcgen::KeyPair::generate()?;
            let certificate = ca_params.self_signed(&key_pair)?;
            let private_key = Self::rcgen_cert_privkey(&key_pair)?;

            let key_pair_from_signing_key = Self::new_from_signing_key(
                private_key,
                BorrowingCertificate::from_certificate_der(certificate.der().clone())?,
            )?;
            Ok(key_pair_from_signing_key)
        }

        /// Generate a new key pair signed with the specified CA.
        pub fn generate(
            &self,
            common_name: &str,
            certificate_type: &CertificateType,
            configuration: CertificateConfiguration,
        ) -> Result<Self, CertificateError> {
            let mut cert_params = CertificateParams::from(configuration);
            cert_params.is_ca = IsCa::NoCa;
            cert_params.distinguished_name.push(DnType::CommonName, common_name);
            cert_params
                .subject_alt_names
                .push(SanType::DnsName(common_name.try_into()?));
            cert_params.custom_extensions.extend(certificate_type.to_custom_exts()?);

            let ca_keypair = rcgen::KeyPair::from_pkcs8_der_and_sign_algo(
                &self
                    .private_key()
                    .to_pkcs8_der()
                    .map_err(CertificateError::GeneratingPrivateKey)?
                    .as_bytes()
                    .into(),
                &PKCS_ECDSA_P256_SHA256,
            )?;
            let ca = rcgen::CertificateParams::from_ca_cert_der(&self.certificate().as_ref().into())?
                .self_signed(&ca_keypair)?;

            let cert_key_pair = rcgen::KeyPair::generate()?;
            let certificate = cert_params.signed_by(&cert_key_pair, &ca, &ca_keypair)?;
            let private_key = Self::rcgen_cert_privkey(&cert_key_pair)?;

            let key_pair_from_signing_key = Self::new_from_signing_key(
                private_key,
                BorrowingCertificate::from_certificate_der(certificate.der().clone())?,
            )?;
            Ok(key_pair_from_signing_key)
        }

        fn rcgen_cert_privkey(keypair: &rcgen::KeyPair) -> Result<SigningKey, CertificateError> {
            SigningKey::from_pkcs8_der(keypair.serialized_der()).map_err(CertificateError::GeneratingPrivateKey)
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
        pub fn to_custom_exts(&self) -> Result<Vec<CustomExtension>, CertificateError> {
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
    mod mock {
        use crate::server_keys::KeyPair;
        use crate::utils::issuer_auth::IssuerRegistration;
        use crate::utils::reader_auth::ReaderRegistration;

        use super::*;

        const ISSUANCE_CA_CN: &str = "ca.issuer.example.com";
        const ISSUANCE_CERT_CN: &str = "cert.issuer.example.com";

        const RP_CA_CN: &str = "ca.rp.example.com";
        const RP_CERT_CN: &str = "cert.rp.example.com";

        impl KeyPair {
            pub fn generate_issuer_mock_ca() -> Result<Self, CertificateError> {
                KeyPair::generate_ca(ISSUANCE_CA_CN, Default::default())
            }

            pub fn generate_reader_mock_ca() -> Result<Self, CertificateError> {
                KeyPair::generate_ca(RP_CA_CN, Default::default())
            }

            pub fn generate_issuer_mock(
                &self,
                issuer_registration: Option<IssuerRegistration>,
            ) -> Result<Self, CertificateError> {
                self.generate(
                    ISSUANCE_CERT_CN,
                    &CertificateType::Mdl(issuer_registration.map(Box::new)),
                    Default::default(),
                )
            }

            pub fn generate_reader_mock(
                &self,
                reader_registration: Option<ReaderRegistration>,
            ) -> Result<Self, CertificateError> {
                self.generate(
                    RP_CERT_CN,
                    &CertificateType::ReaderAuth(reader_registration.map(Box::new)),
                    Default::default(),
                )
            }
        }
    }
}
