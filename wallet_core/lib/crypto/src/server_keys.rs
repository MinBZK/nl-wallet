use derive_more::Debug;
use p256::ecdsa::Signature;
use p256::ecdsa::SigningKey;

use crate::keys::EcdsaKey;
use crate::x509::BorrowingCertificate;
use crate::x509::CertificateError;

#[derive(Debug, Clone)]
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

    async fn verifying_key(&self) -> Result<p256::ecdsa::VerifyingKey, Self::Error> {
        self.private_key.verifying_key().await
    }

    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error> {
        self.private_key.try_sign(msg).await
    }
}

#[cfg(any(test, feature = "generate"))]
pub mod generate {
    use p256::ecdsa::SigningKey;
    use p256::pkcs8::DecodePrivateKey;
    use rcgen::BasicConstraints;
    use rcgen::CertificateParams;
    use rcgen::CertificateRevocationList;
    use rcgen::CertificateRevocationListParams;
    use rcgen::CrlDistributionPoint;
    use rcgen::DnType;
    use rcgen::IsCa;
    use rcgen::Issuer;
    use rcgen::KeyIdMethod;
    use rcgen::PKCS_ECDSA_P256_SHA256;
    use rcgen::PublicKeyData;
    use rcgen::RevokedCertParams;
    use rcgen::SanType;
    use rcgen::SerialNumber;
    use rcgen::SubjectPublicKeyInfo;
    use rustls_pki_types::CertificateDer;
    use rustls_pki_types::TrustAnchor;
    use time::Duration;
    use time::OffsetDateTime;
    use x509_parser::prelude::FromDer;
    use x509_parser::prelude::X509Certificate;

    use crate::server_keys::KeyPair;
    use crate::trust_anchor::BorrowingTrustAnchor;
    use crate::x509::BorrowingCertificate;
    use crate::x509::CertificateConfiguration;
    use crate::x509::CertificateError;
    use crate::x509::CertificateUsage;
    use crate::x509::DistinguishedName;

    fn rcgen_cert_privkey(keypair: &rcgen::KeyPair) -> Result<SigningKey, CertificateError> {
        SigningKey::from_pkcs8_der(keypair.serialized_der())
            .map_err(|error| CertificateError::GeneratingPrivateKey(Box::new(error)))
    }

    pub struct Ca {
        issuer: Issuer<'static, rcgen::KeyPair>,
        certificate: CertificateDer<'static>,
        borrowing_trust_anchor: BorrowingTrustAnchor,
        intermediate_count: u8,
    }

    impl Ca {
        fn new(
            issuer: Issuer<'static, rcgen::KeyPair>,
            certificate: CertificateDer<'static>,
            intermediate_count: u8,
        ) -> Result<Self, CertificateError> {
            let borrowing_trust_anchor = BorrowingTrustAnchor::from_der(certificate.as_ref())
                .map_err(|error| CertificateError::Verification(Box::new(error)))?;

            let ca = Self {
                issuer,
                certificate,
                borrowing_trust_anchor,
                intermediate_count,
            };

            Ok(ca)
        }

        /// Generate a new self-signed CA key pair, constrained to the specified number of intermediates CAs.
        pub fn generate_with_intermediate_count(
            distinguished_name: DistinguishedName,
            configuration: CertificateConfiguration,
            intermediate_count: u8,
        ) -> Result<Self, CertificateError> {
            let mut params = CertificateParams::from(configuration);
            params.is_ca = IsCa::Ca(BasicConstraints::Constrained(intermediate_count));
            params.distinguished_name = distinguished_name.into();

            let key_pair = rcgen::KeyPair::generate()?;
            let certificate = params.self_signed(&key_pair)?;
            let issuer = Issuer::new(params, key_pair);

            Self::new(issuer, certificate.into(), intermediate_count)
        }

        /// Generate a new self-signed CA key pair, constrained to having no intermediate CAs.
        pub fn generate(
            distinguished_name: DistinguishedName,
            configuration: CertificateConfiguration,
        ) -> Result<Self, CertificateError> {
            Self::generate_with_intermediate_count(distinguished_name, configuration, 0)
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

            let key_pair = rcgen::KeyPair::from_pkcs8_der_and_sign_algo(
                &signing_key_der.as_ref().into(),
                &PKCS_ECDSA_P256_SHA256,
            )?;
            let certificate = CertificateDer::from(certificate_der.as_ref()).into_owned();
            let issuer = Issuer::from_ca_cert_der(&certificate, key_pair)?;

            // Unfortunately `x509_parser` does not parse the intermediate count from
            // the basic constraint, so we should assume the worst, which is 0.
            Self::new(issuer, certificate, 0)
        }

        pub fn certificate(&self) -> &CertificateDer<'static> {
            &self.certificate
        }

        pub fn as_borrowing_certificate(&self) -> Result<BorrowingCertificate, CertificateError> {
            BorrowingCertificate::from_der(self.certificate().as_ref())
        }

        pub fn borrowing_trust_anchor(&self) -> &BorrowingTrustAnchor {
            &self.borrowing_trust_anchor
        }

        pub fn to_borrowing_trust_anchor(&self) -> BorrowingTrustAnchor {
            self.borrowing_trust_anchor.clone()
        }

        pub fn to_signing_key(&self) -> Result<SigningKey, CertificateError> {
            rcgen_cert_privkey(self.issuer.key())
        }

        pub fn to_trust_anchor(&self) -> TrustAnchor<'_> {
            self.borrowing_trust_anchor.as_trust_anchor().clone()
        }

        /// Generate a signed CRL from this CA, revoking the given certificates.
        pub fn generate_crl(
            &self,
            revoked_certs: Vec<RevokedCertParams>,
        ) -> Result<CertificateRevocationList, CertificateError> {
            let now = OffsetDateTime::now_utc();
            self.generate_crl_with_validity(revoked_certs, now, now + Duration::days(7))
        }

        /// Generate a signed CRL from this CA, revoking the given certificates, with explicit
        /// `thisUpdate`/`nextUpdate` fields. Used to test CRL expiry handling.
        pub fn generate_crl_with_validity(
            &self,
            revoked_certs: Vec<RevokedCertParams>,
            this_update: OffsetDateTime,
            next_update: OffsetDateTime,
        ) -> Result<CertificateRevocationList, CertificateError> {
            let params = CertificateRevocationListParams {
                this_update,
                next_update,
                crl_number: SerialNumber::from(1u64),
                issuing_distribution_point: None,
                revoked_certs,
                key_identifier_method: KeyIdMethod::Sha256,
            };
            params
                .signed_by(&self.issuer)
                .map_err(CertificateError::GeneratingFailed)
        }

        /// Generate a new intermediate CA key pair, with any constraint
        /// on the amount of intermediates it can have decremented by one.
        pub fn generate_intermediate(
            &self,
            distinguished_name: DistinguishedName,
            configuration: CertificateConfiguration,
        ) -> Result<Self, CertificateError> {
            if self.intermediate_count < 1 {
                return Err(CertificateError::BasicConstraintViolation);
            }

            let intermediate_count = self.intermediate_count - 1;
            let constraint = BasicConstraints::Constrained(intermediate_count);

            let mut params = CertificateParams::from(configuration);
            params.is_ca = IsCa::Ca(constraint);
            params.distinguished_name = distinguished_name.into();

            let key_pair = rcgen::KeyPair::generate()?;
            let certificate = params.signed_by(&key_pair, &self.issuer)?;
            let issuer = Issuer::new(params, key_pair);

            Self::new(issuer, certificate.into(), intermediate_count)
        }

        fn certificate_for(
            &self,
            pk: &impl PublicKeyData,
            distinguished_name: DistinguishedName,
            configuration: CertificateConfiguration,
            subject_alt_names: impl IntoIterator<Item = impl Into<SanType>>,
        ) -> Result<BorrowingCertificate, CertificateError> {
            let mut params = CertificateParams::from(configuration);
            params.is_ca = IsCa::NoCa;
            params.distinguished_name = distinguished_name.into();
            params.subject_alt_names = subject_alt_names.into_iter().map(Into::into).collect();

            let certificate = params.signed_by(pk, &self.issuer)?;
            let certificate = BorrowingCertificate::from_certificate_der(certificate.into())?;
            Ok(certificate)
        }

        /// Generate a new key pair signed with the specified CA.
        pub fn generate_key_pair(
            &self,
            distinguished_name: DistinguishedName,
            configuration: CertificateConfiguration,
            subject_alt_names: impl IntoIterator<Item = impl Into<SanType>>,
        ) -> Result<KeyPair, CertificateError> {
            let key_pair = rcgen::KeyPair::generate()?;
            let private_key = rcgen_cert_privkey(&key_pair)?;
            let certificate = self.certificate_for(&key_pair, distinguished_name, configuration, subject_alt_names)?;

            let key_pair = KeyPair {
                private_key,
                certificate,
            };

            Ok(key_pair)
        }

        /// Generate a new key pair signed with the specified CA.
        pub fn generate_certificate(
            &self,
            public_key: &[u8],
            distinguished_name: DistinguishedName,
            configuration: CertificateConfiguration,
            subject_alt_names: impl IntoIterator<Item = impl Into<SanType>>,
        ) -> Result<BorrowingCertificate, CertificateError> {
            let public_key = SubjectPublicKeyInfo::from_der(public_key)?;
            self.certificate_for(&public_key, distinguished_name, configuration, subject_alt_names)
        }

        /// Generate a new key pair and return both a self-signed root `Ca` for it and a
        /// cross-signed certificate DER where the same key pair is signed by `self`.
        /// The cross-cert carries the given `usage` EKU so that webpki can validate chains
        /// that use it as an intermediate.
        pub fn generate_root_and_cross_cert(
            &self,
            distinguished_name: DistinguishedName,
            configuration: CertificateConfiguration,
        ) -> Result<(Self, CertificateDer<'static>), CertificateError> {
            let key_pair = rcgen::KeyPair::generate()?;

            let mut self_signed_params = CertificateParams::from(configuration.clone());
            self_signed_params.is_ca = IsCa::Ca(BasicConstraints::Constrained(0));
            self_signed_params.distinguished_name = distinguished_name.clone().into();
            let self_signed_cert = self_signed_params.self_signed(&key_pair)?;

            let mut cross_params = CertificateParams::from(configuration);
            cross_params.is_ca = IsCa::Ca(BasicConstraints::Constrained(0));
            cross_params.distinguished_name = distinguished_name.into();
            let cross_cert = cross_params.signed_by(&key_pair, &self.issuer)?;

            let issuer = Issuer::new(self_signed_params, key_pair);
            let root_ca = Self::new(issuer, self_signed_cert.into(), 0)?;

            Ok((root_ca, cross_cert.into()))
        }
    }

    impl From<CertificateConfiguration> for CertificateParams {
        fn from(source: CertificateConfiguration) -> Self {
            let mut result = CertificateParams::default();
            if let Some(not_before) = source.not_before.and_then(|ts| ts.timestamp_nanos_opt()) {
                result.not_before = OffsetDateTime::from_unix_timestamp_nanos(i128::from(not_before)).unwrap();
            }
            if let Some(not_after) = source.not_after.and_then(|ts| ts.timestamp_nanos_opt()) {
                result.not_after = OffsetDateTime::from_unix_timestamp_nanos(i128::from(not_after)).unwrap();
            }
            result.use_authority_key_identifier_extension = !source.exclude_aki;
            if let Some(usage) = source.usage {
                result.extended_key_usages.push(usage.to_key_usage_purpose());
            }
            if let Some(extension) = source.extension {
                result.custom_extensions.push(extension);
            }
            result.crl_distribution_points = source
                .crl_distribution_points
                .into_iter()
                .map(|uri| CrlDistributionPoint {
                    uris: vec![uri.to_string()],
                })
                .collect();
            result
        }
    }

    #[cfg(any(test, feature = "mock"))]
    pub mod mock {

        use std::sync::LazyLock;

        use super::*;
        use crate::x509::NO_SAN;
        use crate::x509::SubjectAltNameUri;

        pub static WRPAC_CA_DN: LazyLock<DistinguishedName> =
            LazyLock::new(|| DistinguishedName::create_mock("CA wrpac"));

        pub static ISSUANCE_CA_DN: LazyLock<DistinguishedName> =
            LazyLock::new(|| DistinguishedName::create_mock("CA issuer"));
        pub static ISSUANCE_CERT_DN: LazyLock<DistinguishedName> =
            LazyLock::new(|| DistinguishedName::create_legal_person_mock("Cert issuer"));
        pub static ISSUANCE_CERT_SAN_URI: LazyLock<SubjectAltNameUri> =
            LazyLock::new(|| "https://issuer.example.com".parse().unwrap());
        pub static PID_ISSUER_CERT_DN: LazyLock<DistinguishedName> =
            LazyLock::new(|| DistinguishedName::create_legal_person_mock("PID"));
        pub static PID_ISSUER_CERT_SAN_URI: LazyLock<SubjectAltNameUri> =
            LazyLock::new(|| "https://pid.example.com".parse().unwrap());
        pub static WIA_CERT_DN: LazyLock<DistinguishedName> = LazyLock::new(|| DistinguishedName::create_mock("WIA"));

        pub static RP_CA_DN: LazyLock<DistinguishedName> =
            LazyLock::new(|| DistinguishedName::create_mock("CA relying party"));
        pub static RP_CERT_DN: LazyLock<DistinguishedName> =
            LazyLock::new(|| DistinguishedName::create_legal_person_mock("Cert relying party"));
        pub static RP_CERT_SAN_URI: LazyLock<SubjectAltNameUri> =
            LazyLock::new(|| "https://cert.rp.example.com".parse().unwrap());

        impl Ca {
            pub fn generate_mock() -> Self {
                Self::generate(
                    DistinguishedName::create_mock("myca"),
                    CertificateConfiguration::default(),
                )
                .unwrap()
            }

            pub fn generate_wrpac_mock_ca() -> Result<Self, CertificateError> {
                Self::generate(WRPAC_CA_DN.clone(), Default::default())
            }

            pub fn generate_issuer_mock_ca() -> Result<Self, CertificateError> {
                Self::generate(ISSUANCE_CA_DN.clone(), Default::default())
            }

            pub fn generate_issuer_mock_ca_without_aki() -> Result<Self, CertificateError> {
                Self::generate(
                    ISSUANCE_CA_DN.clone(),
                    CertificateConfiguration {
                        exclude_aki: true,
                        ..Default::default()
                    },
                )
            }

            pub fn generate_wrpac_issuer_mock(&self) -> Result<KeyPair, CertificateError> {
                self.generate_key_pair(ISSUANCE_CERT_DN.clone(), Default::default(), NO_SAN)
            }

            pub fn generate_wrpac_verifier_mock(&self) -> Result<KeyPair, CertificateError> {
                self.generate_key_pair(RP_CERT_DN.clone(), Default::default(), NO_SAN)
            }

            pub fn generate_pid_issuer_mock(&self) -> Result<KeyPair, CertificateError> {
                self.generate_key_pair(
                    PID_ISSUER_CERT_DN.clone(),
                    CertificateConfiguration::with_usage(CertificateUsage::Mdl),
                    [PID_ISSUER_CERT_SAN_URI.clone()],
                )
            }

            pub fn generate_issuer_mock(&self) -> Result<KeyPair, CertificateError> {
                self.generate_key_pair(
                    ISSUANCE_CERT_DN.clone(),
                    CertificateConfiguration::with_usage(CertificateUsage::Mdl),
                    [ISSUANCE_CERT_SAN_URI.clone()],
                )
            }

            pub fn generate_wia_mock(&self) -> Result<KeyPair, CertificateError> {
                self.generate_key_pair(
                    WIA_CERT_DN.clone(),
                    CertificateConfiguration::with_usage(CertificateUsage::Wia),
                    NO_SAN,
                )
            }

            pub fn generate_issuer_status_list_mock(&self) -> Result<KeyPair, CertificateError> {
                self.generate_key_pair(
                    ISSUANCE_CERT_DN.clone(),
                    CertificateConfiguration::with_usage(CertificateUsage::OAuthStatusSigning),
                    [ISSUANCE_CERT_SAN_URI.clone()],
                )
            }

            pub fn generate_pid_issuer_status_list_mock(&self) -> Result<KeyPair, CertificateError> {
                self.generate_key_pair(
                    PID_ISSUER_CERT_DN.clone(),
                    CertificateConfiguration::with_usage(CertificateUsage::OAuthStatusSigning),
                    [PID_ISSUER_CERT_SAN_URI.clone()],
                )
            }

            /// Generate a TLS server key pair with the given hostname as the DNS SAN.
            /// No custom extended key usage extensions are added, which allows webpki to accept
            /// the certificate for TLS server authentication.
            pub fn generate_tls_mock(&self, hostname: &str) -> Result<KeyPair, CertificateError> {
                let key_pair = rcgen::KeyPair::generate()?;
                let private_key = rcgen_cert_privkey(&key_pair)?;

                let mut params = CertificateParams::default();
                params.is_ca = IsCa::NoCa;
                params.distinguished_name.push(DnType::CommonName, hostname);
                params.subject_alt_names.push(SanType::DnsName(hostname.try_into()?));

                let certificate = params.signed_by(&key_pair, &self.issuer)?;
                let certificate = BorrowingCertificate::from_certificate_der(certificate.into())?;

                Ok(KeyPair {
                    private_key,
                    certificate,
                })
            }
        }
    }
}
