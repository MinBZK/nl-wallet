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
pub enum RegistrationCertificateEnvelopeError {
    #[error("could not parse registration certificate JWT: {0}")]
    JwtParsing(#[source] JwtParseError),
    #[error("could not verify registration certificate JWT: {0}")]
    JwtVerification(#[source] JwtX5cVerifyError),
    #[error("could not parse or verify registration certificate CWT: {0}")]
    Cwt(#[source] WrprcCwtError),
    #[error("could not parse registration-certificate signing-certificate subject: {0}")]
    SigningCertificateSubject(#[source] CertificateError),
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

/// Parse and verify a JAdES-B/B JWT or WRPRC CWT registration-certificate envelope.
///
/// Per the ETSI TS 119 472-2 and 119 475 specs, the passed bytes may be of either format.
/// Therefore this function attempts the passed bytes as either format.
pub fn verify_registration_certificate_envelope(
    registration_certificate: &[u8],
    trust_anchors: &TrustAnchors,
    time: &impl Generator<DateTime<Utc>>,
) -> Result<VerifiedRegistrationCertificateEnvelope, RegistrationCertificateEnvelopeError> {
    let (payload, signing_certificate_dn) = match str::from_utf8(registration_certificate) {
        Ok(compact_jwt) if compact_jwt.split('.').count() == 3 => {
            let unverified: UnverifiedJwt<UncheckedRegistrationCertificate, JadesbbHeader> = compact_jwt
                .parse()
                .map_err(RegistrationCertificateEnvelopeError::JwtParsing)?;
            let (header, payload) = unverified
                .parse_and_verify_against_trust_anchors(trust_anchors, time, None, DEFAULT_VALIDATION.to_owned())
                .map_err(RegistrationCertificateEnvelopeError::JwtVerification)?;
            let signing_certificate_dn = header
                .x5c
                .first()
                .to_canonical_distinguished_name()
                .map_err(RegistrationCertificateEnvelopeError::SigningCertificateSubject)?;

            (payload, signing_certificate_dn)
        }
        _ => {
            let verified = UnverifiedWrprcCwt::<UncheckedRegistrationCertificate>::from_slice(registration_certificate)
                .map_err(RegistrationCertificateEnvelopeError::Cwt)?
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
