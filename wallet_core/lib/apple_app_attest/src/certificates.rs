use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use derive_more::AsRef;
use nutype::nutype;
use p256::ecdsa::VerifyingKey;
use p256::pkcs8::DecodePublicKey;
use rasn::AsnType;
use rasn::Decode;
use rasn::Decoder;
#[cfg(feature = "serialize")]
use rasn::Encoder;
use rasn::types::OctetString;
use rustls_pki_types::CertificateDer;
use rustls_pki_types::TrustAnchor;
use rustls_pki_types::UnixTime;
use webpki::EndEntityCert;
use webpki::KeyUsage;
use webpki::ring::ECDSA_P256_SHA256;
use webpki::ring::ECDSA_P256_SHA384;
use webpki::ring::ECDSA_P384_SHA256;
use webpki::ring::ECDSA_P384_SHA384;
use x509_parser::certificate::X509Certificate;
use x509_parser::der_parser::Oid;
use x509_parser::der_parser::oid;
use x509_parser::error::X509Error;
use x509_parser::prelude::FromDer;

#[rustfmt::skip]
pub const APPLE_ANONYMOUS_ATTESTATION_OID: Oid = oid!(1.2.840.113635.100.8.2);

#[derive(Debug, thiserror::Error)]
pub enum CertificateError {
    #[error("parsing certificate chain failed: {0}")]
    ChainParsing(#[source] webpki::Error),
    #[error("provided time is earlier than the unix epoch")]
    TimeBeforeUnixEpoch,
    #[error("certificate chain verification failed: {0}")]
    ChainVerification(#[source] webpki::Error),
    #[error("parsing credential certificate failed: {0}")]
    CredentialParsing(#[source] X509Error),
    #[error("parsing public key failed: {0}")]
    PublicKeyParsing(#[source] p256::pkcs8::spki::Error),
    #[error("anonymous attestation extension is not present in certificate")]
    ExtensionMissing,
    #[error("extracting anonymous attestation extension from certificate failed: {0}")]
    ExtensionExtraction(#[source] X509Error),
    #[error("parsing anonymous attestation certificate extension failed: {0}")]
    ExtensionParsing(#[source] rasn::error::DecodeError),
}

#[nutype(
    validate(predicate = |certs| !certs.is_empty()),
    derive(Debug, Clone, TryFrom, AsRef)
)]
pub struct DerX509CertificateChain(Vec<Vec<u8>>);

#[cfg(feature = "serialize")]
impl From<DerX509CertificateChain> for Vec<Vec<u8>> {
    fn from(value: DerX509CertificateChain) -> Self {
        value.into_inner()
    }
}

impl DerX509CertificateChain {
    fn credential_certificate_der(&self) -> &[u8] {
        // This is guaranteed to succeed by the type's validation predicate.
        self.as_ref().first().unwrap()
    }

    fn intermediate_certificates_der(&self) -> Vec<&[u8]> {
        self.as_ref().iter().skip(1).map(Vec::as_slice).collect()
    }

    pub fn credential_certificate(&self) -> Result<CredentialCertificate<'_>, CertificateError> {
        let (_, cert) = X509Certificate::from_der(self.credential_certificate_der())
            .map_err(|error| CertificateError::CredentialParsing(error.into()))?;

        let certificate = CredentialCertificate(cert);

        Ok(certificate)
    }

    pub(crate) fn verify(&self, trust_anchors: &[TrustAnchor], time: DateTime<Utc>) -> Result<(), CertificateError> {
        let timestamp = time
            .timestamp()
            .try_into()
            .map_err(|_| CertificateError::TimeBeforeUnixEpoch)?;

        EndEntityCert::try_from(&CertificateDer::from(self.credential_certificate_der()))
            .map_err(CertificateError::ChainParsing)?
            .verify_for_usage(
                &[
                    ECDSA_P256_SHA256,
                    ECDSA_P256_SHA384,
                    ECDSA_P384_SHA256,
                    ECDSA_P384_SHA384,
                ],
                trust_anchors,
                self.intermediate_certificates_der()
                    .into_iter()
                    .map(CertificateDer::from)
                    .collect::<Vec<_>>()
                    .as_slice(),
                UnixTime::since_unix_epoch(Duration::from_secs(timestamp)),
                KeyUsage::client_auth(),
                None,
                None,
            )
            .map_err(CertificateError::ChainVerification)?;

        Ok(())
    }
}

#[derive(Debug, AsnType, Decode)]
#[cfg_attr(feature = "serialize", derive(rasn::Encode))]
pub struct AppleAnonymousAttestationExtension {
    #[rasn(tag(explicit(1)))]
    pub nonce: OctetString,
}

#[derive(Debug, AsRef)]
pub struct CredentialCertificate<'a>(X509Certificate<'a>);

impl CredentialCertificate<'_> {
    pub fn public_key(&self) -> Result<VerifyingKey, CertificateError> {
        let public_key = VerifyingKey::from_public_key_der(self.as_ref().public_key().raw)
            .map_err(CertificateError::PublicKeyParsing)?;

        Ok(public_key)
    }

    pub fn attestation_extension(&self) -> Result<AppleAnonymousAttestationExtension, CertificateError> {
        let extension = self
            .as_ref()
            .get_extension_unique(&APPLE_ANONYMOUS_ATTESTATION_OID)
            .map_err(CertificateError::ExtensionExtraction)?
            .ok_or(CertificateError::ExtensionMissing)?;

        let decoded_extension = rasn::der::decode(extension.value).map_err(CertificateError::ExtensionParsing)?;

        Ok(decoded_extension)
    }
}
