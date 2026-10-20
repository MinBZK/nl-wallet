use std::str;

use base64::DecodeError;
use base64::prelude::*;
use chrono::DateTime;
use chrono::Utc;
use cose::wrprc_cwt::UnverifiedWrprcCwt;
use cose::wrprc_cwt::WrprcCwtError;
use crypto::trust_anchor::TrustAnchors;
use jwt::DEFAULT_VALIDATION;
use jwt::UnverifiedJwt;
use jwt::error::JwtParseError;
use jwt::error::JwtX5cVerifyError;
use jwt::jades_b_b::JadesbbHeader;
use utils::generator::Generator;

use super::ParsedRegistrationCertificate;

#[derive(Debug, thiserror::Error)]
pub enum RegistrationCertificateJwtParseError {
    #[error("not valid UTF-8: {0}")]
    Utf8(#[from] str::Utf8Error),
    #[error(transparent)]
    Jwt(#[from] JwtParseError),
}

#[derive(Debug, thiserror::Error)]
pub enum RegistrationCertificateEnvelopeParseError {
    #[error("could not decode registration certificate as Base64: {0}")]
    Base64(#[source] DecodeError),
    #[error("could not parse registration certificate as JWT: {0}")]
    Jwt(#[source] RegistrationCertificateJwtParseError),
    #[error("could not parse registration certificate as JWT ({jwt}) or CWT ({cwt})")]
    NeitherFormat {
        jwt: RegistrationCertificateJwtParseError,
        cwt: Box<WrprcCwtError>,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum RegistrationCertificateEnvelopeError {
    #[error("could not verify registration certificate JWT: {0}")]
    Jwt(#[source] JwtX5cVerifyError),
    #[error("could not verify registration certificate CWT: {0}")]
    Cwt(#[source] WrprcCwtError),
}

/// A registration certificate in JWT or CWT form, with an encoded, unparsed payload.
///
/// This type provides no guarantees about signature trust or payload validity.
#[derive(Debug, Clone)]
pub enum RegistrationCertificateEnvelope {
    Jwt(UnverifiedJwt<ParsedRegistrationCertificate, JadesbbHeader>),
    Cwt(Box<UnverifiedWrprcCwt<ParsedRegistrationCertificate>>),
}

impl TryFrom<&[u8]> for RegistrationCertificateEnvelope {
    type Error = RegistrationCertificateEnvelopeParseError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::parse_jwt(bytes).or_else(|jwt| {
            UnverifiedWrprcCwt::from_slice(bytes)
                .map(Box::new)
                .map(Self::Cwt)
                .map_err(|cwt| RegistrationCertificateEnvelopeParseError::NeitherFormat {
                    jwt,
                    cwt: Box::new(cwt),
                })
        })
    }
}

impl TryFrom<String> for RegistrationCertificateEnvelope {
    type Error = RegistrationCertificateEnvelopeParseError;

    fn try_from(data: String) -> Result<Self, Self::Error> {
        // ETSI TS 119 472-2 requires the base64url encoding of the serialized registration certificate. Since a compact
        // JWT already consists of base64url-encoded parts, some implementations include it without encoding it again.
        // We choose to accept that as well: a compact JWT always contains a '.', while base64url never does.
        if data.contains('.') {
            return Self::parse_jwt(data.as_bytes()).map_err(RegistrationCertificateEnvelopeParseError::Jwt);
        }

        let bytes = BASE64_URL_SAFE_NO_PAD
            .decode(data)
            .map_err(RegistrationCertificateEnvelopeParseError::Base64)?;

        Self::try_from(bytes.as_slice())
    }
}

impl TryFrom<&RegistrationCertificateEnvelope> for String {
    type Error = WrprcCwtError;

    fn try_from(envelope: &RegistrationCertificateEnvelope) -> Result<Self, Self::Error> {
        Ok(BASE64_URL_SAFE_NO_PAD.encode(envelope.to_vec()?))
    }
}

impl RegistrationCertificateEnvelope {
    fn parse_jwt(bytes: &[u8]) -> Result<Self, RegistrationCertificateJwtParseError> {
        let compact_jwt = str::from_utf8(bytes)?;
        let parts = compact_jwt.split('.').count();

        if parts != 3 {
            return Err(JwtParseError::UnexpectedNumberOfParts(parts).into());
        }

        Ok(Self::Jwt(compact_jwt.parse()?))
    }

    pub fn to_vec(&self) -> Result<Vec<u8>, WrprcCwtError> {
        match self {
            Self::Jwt(jwt) => Ok(jwt.serialization().as_bytes().to_vec()),
            Self::Cwt(cwt) => cwt.to_vec(),
        }
    }
}

/// A registration-certificate envelope whose signature and certificate chain have been verified and whose payload
/// satisfies its intrinsic rules. WRPAC binding, current validity, status, and query authorization are separate checks.
pub struct VerifiedRegistrationCertificateEnvelope {
    payload: ParsedRegistrationCertificate,
}

impl VerifiedRegistrationCertificateEnvelope {
    pub fn into_payload(self) -> ParsedRegistrationCertificate {
        self.payload
    }
}

/// Verify a parsed JAdES-B/B JWT or WRPRC CWT registration-certificate envelope.
pub fn verify_registration_certificate_envelope(
    registration_certificate: &RegistrationCertificateEnvelope,
    trust_anchors: &TrustAnchors,
    time: &impl Generator<DateTime<Utc>>,
) -> Result<VerifiedRegistrationCertificateEnvelope, RegistrationCertificateEnvelopeError> {
    let payload = match registration_certificate.clone() {
        RegistrationCertificateEnvelope::Jwt(unverified) => {
            let (_, payload) = unverified
                .parse_and_verify_ignoring_trust_anchor_in_x5c(trust_anchors, time, None, DEFAULT_VALIDATION.to_owned())
                .map_err(RegistrationCertificateEnvelopeError::Jwt)?;
            payload
        }
        RegistrationCertificateEnvelope::Cwt(unverified) => (*unverified)
            .into_verified_against_trust_anchors(trust_anchors, time, None)
            .map_err(RegistrationCertificateEnvelopeError::Cwt)?
            .into_payload(),
    };

    Ok(VerifiedRegistrationCertificateEnvelope { payload })
}
