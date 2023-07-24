use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use p256::{
    ecdsa::{SigningKey, VerifyingKey},
    elliptic_curve::pkcs8::DecodePublicKey,
    pkcs8::{
        der::{asn1::SequenceOf, Encode},
        DecodePrivateKey, EncodePrivateKey, ObjectIdentifier,
    },
};
use rcgen::{
    BasicConstraints, Certificate as RcgenCertificate, CertificateParams, CustomExtension, DnType, IsCa, RcgenError,
};
use serde_bytes::ByteBuf;
use webpki::{EndEntityCert, KeyPurposeId, NonTlsTrustAnchors, Time, TrustAnchor, ECDSA_P256_SHA256};
use x509_parser::{
    nom::AsBytes,
    prelude::{FromDer, X509Certificate, X509Error},
};

use super::Generator;

#[derive(thiserror::Error, Debug)]
pub enum CertificateError {
    #[error("certificate verification failed: {0}")]
    Verification(#[source] webpki::Error),
    #[error("certificate parsing for validation failed: {0}")]
    ValidationParsing(#[from] webpki::Error),
    #[error("certificate content parsing failed: {0}")]
    ContentParsing(#[from] x509_parser::nom::Err<X509Error>),
    #[error("certificate private key generation failed: {0}")]
    GeneratingPrivateKey(p256::pkcs8::Error),
    #[error("certificate creation failed: {0}")]
    GeneratingFailed(#[from] RcgenError),
    #[error("failed to parse certificate public key: {0}")]
    KeyParsingFailed(p256::pkcs8::spki::Error),
    #[error("EKU count incorrect ({0})")]
    IncorrectEkuCount(usize),
    #[error("EKU incorrect")]
    IncorrectEku(String),
}

const OID_EXT_KEY_USAGE: &[u64] = &[2, 5, 29, 37];

/// Trust anchors against which certificate (chains) must verify, for use in [`Certificate::verify()`].
/// Commonly encoded as self-signed CA X509 certificates.
pub struct TrustAnchors<'a>(&'a [TrustAnchor<'a>]);

impl<'a> From<&'a [TrustAnchor<'a>]> for TrustAnchors<'a> {
    fn from(value: &'a [TrustAnchor<'a>]) -> Self {
        Self(value)
    }
}

/// An x509 certificate, unifying functionality from the following crates:
///
/// - parsing data: `x509_parser`
/// - verification of certificate chains: `webpki`
/// - signing and generating: `rcgen`
pub struct Certificate(ByteBuf);

impl<'a> TryInto<TrustAnchor<'a>> for &'a Certificate {
    type Error = CertificateError;
    fn try_into(self) -> Result<TrustAnchor<'a>, Self::Error> {
        Ok(TrustAnchor::try_from_cert_der(self.as_bytes())?)
    }
}

impl<'a> TryInto<EndEntityCert<'a>> for &'a Certificate {
    type Error = CertificateError;
    fn try_into(self) -> Result<EndEntityCert<'a>, Self::Error> {
        Ok(self.as_bytes().try_into()?)
    }
}

impl<'a> TryInto<X509Certificate<'a>> for &'a Certificate {
    type Error = CertificateError;
    fn try_into(self) -> Result<X509Certificate<'a>, Self::Error> {
        let (_, parsed) = X509Certificate::from_der(self.as_bytes())?;
        Ok(parsed)
    }
}

impl<T: AsRef<[u8]>> From<T> for Certificate {
    fn from(value: T) -> Self {
        Certificate(ByteBuf::from(value.as_ref()))
    }
}

impl Certificate {
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// Verify the certificate against the specified trust anchors.
    pub fn verify(
        &self,
        usage: CertificateUsage,
        intermediate_certs: &[&[u8]],
        time: &impl Generator<DateTime<Utc>>,
        TrustAnchors(trust_anchors): &TrustAnchors,
    ) -> Result<(), CertificateError> {
        self.to_webpki()?
            .verify_is_valid_cert_with_eku(
                &[&ECDSA_P256_SHA256],
                &NonTlsTrustAnchors(trust_anchors),
                intermediate_certs,
                Time::from_seconds_since_unix_epoch(time.generate().timestamp() as u64),
                webpki::ExtendedKeyUsage::Required(KeyPurposeId::new(usage.to_eku())),
                &[],
            )
            .map_err(CertificateError::Verification)
    }

    pub fn public_key(&self) -> Result<VerifyingKey, CertificateError> {
        ecdsa::VerifyingKey::from_public_key_der(self.to_x509()?.public_key().raw)
            .map_err(CertificateError::KeyParsingFailed)
    }

    /// Convert the certificate to a [`X509Certificate`] from the `x509_parser` crate, to read its contents.
    pub fn to_x509(&self) -> Result<X509Certificate, CertificateError> {
        self.try_into()
    }

    /// Convert the certificate to a [`EndEntityCert`] from the `webpki` crate, to verify it (possibly along with a
    /// certificate chain) against a set of trust roots.
    pub fn to_webpki(&self) -> Result<EndEntityCert, CertificateError> {
        self.try_into()
    }

    /// Generate a new self-signed CA certificate.
    pub fn new_ca(common_name: &str) -> Result<(Certificate, SigningKey), CertificateError> {
        let mut ca_params = CertificateParams::new(vec![]);
        ca_params.is_ca = IsCa::Ca(BasicConstraints::Constrained(0));
        ca_params.distinguished_name.push(DnType::CommonName, common_name);
        let cert = RcgenCertificate::from_params(ca_params)?;

        let privkey = Self::rcgen_cert_privkey(&cert)?;

        Ok((cert.serialize_der()?.into(), privkey))
    }

    /// Generate a new certificate signed with the specified CA certificate.
    pub fn new(
        ca: &Certificate,
        ca_privkey: &SigningKey,
        common_name: &str,
        usage: CertificateUsage,
    ) -> Result<(Certificate, SigningKey), CertificateError> {
        let mut cert_params = CertificateParams::new(vec![]);
        cert_params.is_ca = IsCa::NoCa;
        cert_params.distinguished_name.push(DnType::CommonName, common_name);
        cert_params.custom_extensions.push(usage.to_custom_ext());
        let cert_unsigned = RcgenCertificate::from_params(cert_params).map_err(CertificateError::GeneratingFailed)?;

        let ca_keypair = rcgen::KeyPair::from_der(
            &ca_privkey
                .to_pkcs8_der()
                .map_err(CertificateError::GeneratingPrivateKey)?
                .to_bytes(),
        )?;
        let ca = RcgenCertificate::from_params(rcgen::CertificateParams::from_ca_cert_der(&ca.0, ca_keypair)?)?;

        let cert_bts = cert_unsigned.serialize_der_with_signer(&ca)?;
        let cert_privkey = Self::rcgen_cert_privkey(&cert_unsigned)?;

        Ok((cert_bts.into(), cert_privkey))
    }

    pub fn subject(&self) -> Result<IndexMap<String, String>, CertificateError> {
        let subject = self
            .to_x509()?
            .subject
            .iter_attributes()
            .map(|attr| {
                (
                    x509_parser::objects::oid2abbrev(attr.attr_type(), x509_parser::objects::oid_registry())
                        .map_or(attr.attr_type().to_id_string(), |v| v.to_string()),
                    attr.as_str().unwrap().to_string(), // TODO handle non-stringable values?
                )
            })
            .collect();

        Ok(subject)
    }

    fn rcgen_cert_privkey(cert: &RcgenCertificate) -> Result<SigningKey, CertificateError> {
        ecdsa::SigningKey::from_pkcs8_der(cert.get_key_pair().serialized_der())
            .map_err(CertificateError::GeneratingPrivateKey)
    }
}

/// Usage of a [`Certificate`], representing its Extended Key Usage (EKU).
/// [`Certificate::verify()`] receives this as parameter and enforces that it is present in the certificate
/// being verified.
pub enum CertificateUsage {
    Mdl,
    ReaderAuth,
}

impl CertificateUsage {
    fn to_eku(&self) -> &'static [u8] {
        match self {
            CertificateUsage::Mdl => &[40, 129, 140, 93, 5, 1, 2], // OID 1.0.18013.5.1.2
            CertificateUsage::ReaderAuth => &[40, 129, 140, 93, 5, 1, 6], // OID 1.0.18013.5.1.6
        }
    }

    fn to_custom_ext(&self) -> CustomExtension {
        // The spec requires that we add mdoc-specific OIDs to the extended key usage extension, but [`CertificateParams`]
        // only supports a whitelist of key usages that it is aware of. So we DER-serialize it manually and add it to
        // the custom extensions.
        // We unwrap in these functions because they have fixed input for which they always succeed.
        let mut seq = SequenceOf::<ObjectIdentifier, 1>::new();
        seq.add(ObjectIdentifier::from_bytes(self.to_eku()).unwrap()).unwrap();
        let mut ext = CustomExtension::from_oid_content(OID_EXT_KEY_USAGE, seq.to_vec().unwrap());
        ext.set_criticality(true);
        ext
    }
}

#[cfg(test)]
mod test {
    use chrono::{DateTime, Utc};
    use p256::pkcs8::ObjectIdentifier;
    use webpki::TrustAnchor;

    use crate::utils::Generator;

    use super::{Certificate, CertificateUsage};

    #[test]
    fn mdoc_eku_encoding_works() {
        CertificateUsage::Mdl.to_eku();
        CertificateUsage::ReaderAuth.to_eku();
    }

    struct TimeGenerator;
    impl Generator<DateTime<Utc>> for TimeGenerator {
        fn generate(&self) -> DateTime<Utc> {
            Utc::now()
        }
    }

    #[test]
    fn generate_and_verify_cert() {
        let (ca, ca_privkey) = Certificate::new_ca("myca").unwrap();
        let ca_trustanchor: TrustAnchor = (&ca).try_into().unwrap();

        let (cert, _) = Certificate::new(&ca, &ca_privkey, "mycert", CertificateUsage::Mdl).unwrap();

        cert.verify(
            CertificateUsage::Mdl,
            &[],
            &TimeGenerator,
            &[ca_trustanchor].as_slice().into(),
        )
        .unwrap();
    }

    #[test]
    fn parse_oid() {
        let mdl_kp: ObjectIdentifier = "1.0.18013.5.1.2".parse().unwrap();
        let mdl_kp: &'static [u8] = Box::leak(mdl_kp.into()).as_bytes();
        assert_eq!(mdl_kp, CertificateUsage::Mdl.to_eku());

        webpki::KeyPurposeId::new(mdl_kp);
    }
}
