use std::fmt;
use std::str;

use chrono::DateTime;
use chrono::Utc;
use cose::wrprc_cwt::UnverifiedWrprcCwt;
use cose::wrprc_cwt::WrprcCwtError;
use crypto::trust_anchor::TrustAnchors;
use crypto::x509::CanonicalDistinguishedName;
use crypto::x509::CertificateError;
use jwt::DEFAULT_VALIDATION;
use jwt::UnverifiedJwt;
use jwt::error::JwtParseError;
use jwt::error::JwtX5cVerifyError;
use jwt::jades_b_b::JadesbbHeader;
use utils::generator::Generator;

use super::UncheckedRegistrationCertificate;

#[derive(Debug, thiserror::Error)]
pub enum RegistrationCertificateEnvelopeParseError {
    #[error("could not parse registration certificate JWT: {0}")]
    Jwt(#[source] JwtParseError),
    #[error("could not parse registration certificate CWT: {0}")]
    Cwt(#[source] WrprcCwtError),
}

#[derive(Debug, thiserror::Error)]
pub enum RegistrationCertificateEnvelopeError {
    #[error("could not verify registration certificate JWT: {0}")]
    Jwt(#[source] JwtX5cVerifyError),
    #[error("could not verify registration certificate CWT: {0}")]
    Cwt(#[source] WrprcCwtError),
    #[error("could not parse registration-certificate signing-certificate subject: {0}")]
    SigningCertificateSubject(#[source] CertificateError),
}

/// A parsed, but not yet authenticated, registration-certificate envelope.
#[derive(Clone)]
pub enum RegistrationCertificateEnvelope {
    Jwt(UnverifiedJwt<UncheckedRegistrationCertificate, JadesbbHeader>),
    Cwt(Box<UnverifiedWrprcCwt<UncheckedRegistrationCertificate>>),
}

impl fmt::Debug for RegistrationCertificateEnvelope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Jwt(_) => "RegistrationCertificateEnvelope::Jwt",
            Self::Cwt(_) => "RegistrationCertificateEnvelope::Cwt",
        })
    }
}

impl TryFrom<&[u8]> for RegistrationCertificateEnvelope {
    type Error = RegistrationCertificateEnvelopeParseError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        match str::from_utf8(bytes) {
            Ok(compact_jwt) if compact_jwt.split('.').count() == 3 => compact_jwt
                .parse()
                .map(Self::Jwt)
                .map_err(RegistrationCertificateEnvelopeParseError::Jwt),
            _ => UnverifiedWrprcCwt::from_slice(bytes)
                .map(Box::new)
                .map(Self::Cwt)
                .map_err(RegistrationCertificateEnvelopeParseError::Cwt),
        }
    }
}

impl RegistrationCertificateEnvelope {
    pub fn to_vec(&self) -> Result<Vec<u8>, WrprcCwtError> {
        match self {
            Self::Jwt(jwt) => Ok(jwt.serialization().as_bytes().to_vec()),
            Self::Cwt(cwt) => cwt.to_vec(),
        }
    }
}

/// A registration-certificate envelope whose signature and certificate chain have been verified.
pub struct VerifiedRegistrationCertificateEnvelope {
    payload: UncheckedRegistrationCertificate,
    signing_certificate_dn: CanonicalDistinguishedName,
}

impl VerifiedRegistrationCertificateEnvelope {
    pub fn into_payload(self) -> UncheckedRegistrationCertificate {
        self.payload
    }

    pub fn into_parts(self) -> (UncheckedRegistrationCertificate, CanonicalDistinguishedName) {
        (self.payload, self.signing_certificate_dn)
    }
}

/// Verify a parsed JAdES-B/B JWT or WRPRC CWT registration-certificate envelope.
pub fn verify_registration_certificate_envelope(
    registration_certificate: &RegistrationCertificateEnvelope,
    trust_anchors: &TrustAnchors,
    time: &impl Generator<DateTime<Utc>>,
) -> Result<VerifiedRegistrationCertificateEnvelope, RegistrationCertificateEnvelopeError> {
    let (payload, signing_certificate_dn) = match registration_certificate.clone() {
        RegistrationCertificateEnvelope::Jwt(unverified) => {
            let (header, payload) = unverified
                .parse_and_verify_against_trust_anchors(trust_anchors, time, None, DEFAULT_VALIDATION.to_owned())
                .map_err(RegistrationCertificateEnvelopeError::Jwt)?;
            let signing_certificate_dn = header
                .x5c
                .first()
                .to_canonical_distinguished_name()
                .map_err(RegistrationCertificateEnvelopeError::SigningCertificateSubject)?;
            (payload, signing_certificate_dn)
        }
        RegistrationCertificateEnvelope::Cwt(unverified) => {
            let verified = (*unverified)
                .into_verified_against_trust_anchors(trust_anchors, time, None)
                .map_err(RegistrationCertificateEnvelopeError::Cwt)?;
            let signing_certificate_dn = verified
                .header()
                .x5chain
                .first()
                .to_canonical_distinguished_name()
                .map_err(RegistrationCertificateEnvelopeError::SigningCertificateSubject)?;

            (verified.into_payload(), signing_certificate_dn)
        }
    };

    Ok(VerifiedRegistrationCertificateEnvelope {
        payload,
        signing_certificate_dn,
    })
}
