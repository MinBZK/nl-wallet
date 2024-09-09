use std::fmt::{Debug, Formatter};

use p256::ecdsa::{Signature, SigningKey};

use error_category::ErrorCategory;
use wallet_common::keys::EcdsaKey;

use crate::utils::x509::Certificate;

pub struct KeyPair {
    private_key: SigningKey,
    certificate: Certificate,
}

#[derive(thiserror::Error, Debug, ErrorCategory)]
#[category(pd)]
pub enum KeysError {
    #[error("key generation error: {0}")]
    KeyGeneration(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl KeyPair {
    pub fn new(private_key: SigningKey, certificate: Certificate) -> KeyPair {
        KeyPair {
            private_key,
            certificate,
        }
    }

    pub fn private_key(&self) -> &SigningKey {
        &self.private_key
    }

    pub fn certificate(&self) -> &Certificate {
        &self.certificate
    }
}

impl Debug for KeyPair {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyPair")
            .field("certificate", &self.certificate)
            .finish_non_exhaustive()
    }
}

impl From<KeyPair> for Certificate {
    fn from(source: KeyPair) -> Certificate {
        source.certificate
    }
}

impl EcdsaKey for KeyPair {
    type Error = p256::ecdsa::Error;

    async fn verifying_key(&self) -> std::result::Result<p256::ecdsa::VerifyingKey, Self::Error> {
        Ok(*self.private_key.verifying_key())
    }

    async fn try_sign(&self, msg: &[u8]) -> std::result::Result<Signature, Self::Error> {
        p256::ecdsa::signature::Signer::try_sign(&self.private_key, msg)
    }
}

pub trait KeyRing {
    fn key_pair(&self, id: &str) -> Option<&KeyPair>;
}

#[cfg(any(test, feature = "test"))]
pub mod test {
    use super::{KeyPair, KeyRing};

    /// An implementation of [`KeyRing`] containing a single key.
    pub struct SingleKeyRing(pub KeyPair);

    impl KeyRing for SingleKeyRing {
        fn key_pair(&self, _: &str) -> Option<&KeyPair> {
            Some(&self.0)
        }
    }
}

#[cfg(any(test, feature = "generate"))]
mod generate {
    use p256::{
        ecdsa::SigningKey,
        pkcs8::{
            der::{asn1::SequenceOf, Encode},
            DecodePrivateKey, EncodePrivateKey, ObjectIdentifier,
        },
    };
    use rcgen::{
        BasicConstraints, Certificate as RcgenCertificate, CertificateParams, CustomExtension, DnType, IsCa, SanType,
    };
    use time::OffsetDateTime;

    use crate::{
        server_keys::KeyPair,
        utils::x509::{
            CertificateConfiguration, CertificateError, CertificateType, CertificateUsage, MdocCertificateExtension,
            OID_EXT_KEY_USAGE,
        },
    };

    impl KeyPair {
        /// Generate a new self-signed CA key pair.
        pub fn generate_ca(
            common_name: &str,
            configuration: CertificateConfiguration,
        ) -> Result<Self, CertificateError> {
            let mut ca_params = CertificateParams::from(configuration);
            ca_params.is_ca = IsCa::Ca(BasicConstraints::Constrained(0));
            ca_params.distinguished_name.push(DnType::CommonName, common_name);

            let certificate = RcgenCertificate::from_params(ca_params)?;
            let privkey = Self::rcgen_cert_privkey(&certificate)?;

            Ok(KeyPair::new(privkey, certificate.serialize_der()?.into()))
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
                .push(SanType::DnsName(common_name.to_string()));
            cert_params.custom_extensions.extend(certificate_type.to_custom_exts()?);
            let cert_unsigned =
                RcgenCertificate::from_params(cert_params).map_err(CertificateError::GeneratingFailed)?;

            let ca_keypair = rcgen::KeyPair::from_der(
                &self
                    .private_key()
                    .to_pkcs8_der()
                    .map_err(CertificateError::GeneratingPrivateKey)?
                    .to_bytes(),
            )?;
            let ca = RcgenCertificate::from_params(rcgen::CertificateParams::from_ca_cert_der(
                self.certificate().as_bytes(),
                ca_keypair,
            )?)?;

            let certificate = cert_unsigned.serialize_der_with_signer(&ca)?;
            let private_key = Self::rcgen_cert_privkey(&cert_unsigned)?;

            Ok(KeyPair::new(private_key, certificate.into()))
        }

        fn rcgen_cert_privkey(cert: &RcgenCertificate) -> Result<SigningKey, CertificateError> {
            SigningKey::from_pkcs8_der(cert.get_key_pair().serialized_der())
                .map_err(CertificateError::GeneratingPrivateKey)
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
            // The spec requires that we add mdoc-specific OIDs to the extended key usage extension, but
            // [`CertificateParams`] only supports a whitelist of key usages that it is aware of. So we
            // DER-serialize it manually and add it to the custom extensions.
            // We unwrap in these functions because they have fixed input for which they always succeed.
            let mut seq = SequenceOf::<ObjectIdentifier, 1>::new();
            seq.add(ObjectIdentifier::from_bytes(self.to_eku()).unwrap()).unwrap();
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
        use crate::{
            server_keys::KeyPair,
            utils::{issuer_auth::IssuerRegistration, reader_auth::ReaderRegistration},
        };

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
