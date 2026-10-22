use std::str;

use base64::DecodeError;
use base64::prelude::*;
use chrono::DateTime;
use chrono::Utc;
use cose::wrprc_cwt::UnverifiedWrprcCwt;
use cose::wrprc_cwt::WrprcCwtError;
use crypto::trust_anchor::TrustAnchors;
use jwt::UnverifiedJwt;
use jwt::error::JwtParseError;
use jwt::error::JwtX5cVerifyError;
use jwt::jades_b_b::JadesbbHeader;
use mdoc::ItemsRequest;
use utils::generator::Generator;

use super::ParsedRegistrationCertificate;
use super::RegistrationCertificateValidationError;

pub const REQUEST_INFO_WRPRC_KEY: &str = "euWrprc";

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
    #[error("could not parse registration certificate as JWT ({jwt}) or CWT ({cwt})")]
    NeitherFormat {
        jwt: RegistrationCertificateJwtParseError,
        cwt: Box<WrprcCwtError>,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum RegistrationCertificateRequestParseError {
    #[error("missing requestInfo.euWrprc")]
    Missing,
    #[error("requestInfo.euWrprc must be a CBOR byte string")]
    NotByteString,
    #[error("could not parse requestInfo.euWrprc as a WRPRC CWT: {0}")]
    Cwt(#[source] WrprcCwtError),
}

#[derive(Debug, thiserror::Error)]
pub enum RegistrationCertificateEnvelopeError {
    #[error("could not verify registration certificate JWT: {0}")]
    Jwt(#[source] JwtX5cVerifyError),
    #[error("could not verify registration certificate CWT: {0}")]
    Cwt(#[source] WrprcCwtError),
    #[error("registration certificate validity check failed: {0}")]
    Validity(#[source] RegistrationCertificateValidationError),
}

/// A registration certificate in JWT or CWT form, with an encoded, unparsed payload.
///
/// This type provides no guarantees about signature trust or payload validity.
#[derive(Debug, Clone)]
pub enum RegistrationCertificateEnvelope {
    Jwt(UnverifiedJwt<ParsedRegistrationCertificate, JadesbbHeader>),
    Cwt(Box<UnverifiedWrprcCwt<ParsedRegistrationCertificate>>),
}

impl RegistrationCertificateEnvelope {
    pub fn from_slice(bytes: &[u8]) -> Result<Self, RegistrationCertificateEnvelopeParseError> {
        Self::parse_jwt(bytes).or_else(|jwt| {
            UnverifiedWrprcCwt::from_slice(bytes)
                .map(|cwt| Self::Cwt(Box::new(cwt)))
                .map_err(|cwt| RegistrationCertificateEnvelopeParseError::NeitherFormat {
                    jwt,
                    cwt: Box::new(cwt),
                })
        })
    }

    pub fn from_request_info(items_request: &ItemsRequest) -> Result<Self, RegistrationCertificateRequestParseError> {
        let bytes = items_request
            .request_info
            .as_ref()
            .and_then(|request_info| request_info.get(REQUEST_INFO_WRPRC_KEY))
            .ok_or(RegistrationCertificateRequestParseError::Missing)?
            .as_bytes()
            .ok_or(RegistrationCertificateRequestParseError::NotByteString)?;

        UnverifiedWrprcCwt::from_slice(bytes)
            .map(|cwt| Self::Cwt(Box::new(cwt)))
            .map_err(RegistrationCertificateRequestParseError::Cwt)
    }
}

impl TryFrom<String> for RegistrationCertificateEnvelope {
    type Error = RegistrationCertificateEnvelopeParseError;

    fn try_from(data: String) -> Result<Self, Self::Error> {
        // ETSI TS 119 472-2 requires the base64url encoding of the serialized registration certificate. Since a
        // registration certificate encoded as a compact JWT already consists of base64url-encoded parts, some
        // implementations wrongly include it without encoding it. Even though this is not spec-compliant, we leniently
        // accept that as well. Try parsing a compact JWT first: the JWT parser requires a '.', which never occurs
        // in base64url-encoded data.
        if let Ok(jwt) = data.parse() {
            return Ok(Self::Jwt(jwt));
        }

        let bytes = BASE64_URL_SAFE_NO_PAD
            .decode(data)
            .map_err(RegistrationCertificateEnvelopeParseError::Base64)?;

        Self::from_slice(bytes.as_slice())
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
        Ok(Self::Jwt(str::from_utf8(bytes)?.parse()?))
    }

    pub fn to_vec(&self) -> Result<Vec<u8>, WrprcCwtError> {
        match self {
            Self::Jwt(jwt) => Ok(jwt.serialization().as_bytes().to_vec()),
            Self::Cwt(cwt) => cwt.to_vec(),
        }
    }
}

/// A registration-certificate envelope whose signature and certificate chain have been verified and whose payload
/// satisfies its intrinsic rules and has not expired.
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
                .parse_and_verify_jades_b_b(trust_anchors, time, None)
                .map_err(RegistrationCertificateEnvelopeError::Jwt)?;
            payload
        }
        RegistrationCertificateEnvelope::Cwt(unverified) => (*unverified)
            .into_verified_against_trust_anchors(trust_anchors, time, None)
            .map_err(RegistrationCertificateEnvelopeError::Cwt)?
            .into_payload(),
    };

    payload
        .validate_expiry(time.generate())
        .map_err(RegistrationCertificateEnvelopeError::Validity)?;

    Ok(VerifiedRegistrationCertificateEnvelope { payload })
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use ciborium::value::Value;
    use indexmap::IndexMap;
    use mdoc::ItemsRequest;
    use mdoc::utils::serialization::cbor_deserialize;
    use mdoc::utils::serialization::cbor_serialize;
    use rstest::rstest;
    use utils::generator::TimeGenerator;

    use super::REQUEST_INFO_WRPRC_KEY;
    use super::RegistrationCertificateEnvelope;
    use super::RegistrationCertificateRequestParseError;
    use super::verify_registration_certificate_envelope;
    use crate::registration_certificate::mock::MockRegistrationCertificateAuthority;
    use crate::registration_certificate::mock::RegistrationCertificateFixture;
    use crate::registration_certificate::test::valid_payload_json;

    fn items_request(request_info: Option<IndexMap<String, Value>>) -> ItemsRequest {
        ItemsRequest {
            doc_type: "org.example.document".to_string(),
            name_spaces: IndexMap::from_iter([(
                "org.example.namespace".to_string(),
                IndexMap::from_iter([("name".to_string(), true)]).try_into().unwrap(),
            )])
            .try_into()
            .unwrap(),
            request_info,
        }
    }

    #[rstest]
    #[case::untagged(&[])]
    #[case::cose_sign1(&[0xd2])]
    #[case::cwt(&[0xd8, 0x3d, 0xd2])]
    fn parse_items_request_registration_certificate(#[case] tags: &[u8]) {
        let authority = MockRegistrationCertificateAuthority::new();
        let certificate = authority.sign_cwt(&RegistrationCertificateFixture(valid_payload_json()));
        let encoded_certificate = [tags, certificate.as_slice()].concat();
        let request = items_request(Some(IndexMap::from_iter([
            ("unknownBefore".to_string(), Value::Bool(true)),
            (REQUEST_INFO_WRPRC_KEY.to_string(), Value::Bytes(encoded_certificate)),
            ("unknownAfter".to_string(), Value::Text("preserve me".to_string())),
        ])));
        let encoded_request = cbor_serialize(&request).unwrap();
        let request: ItemsRequest = cbor_deserialize(encoded_request.as_slice()).unwrap();

        let envelope = RegistrationCertificateEnvelope::from_request_info(&request).unwrap();

        assert_matches!(&envelope, RegistrationCertificateEnvelope::Cwt(_));
        assert_eq!(envelope.to_vec().unwrap(), certificate);
        // Preserve the original CWT tags, unknown fields, and map order used by reader authentication.
        assert_eq!(cbor_serialize(&request).unwrap(), encoded_request);

        let payload = verify_registration_certificate_envelope(&envelope, &authority.trust_anchors, &TimeGenerator)
            .unwrap()
            .into_payload();
        assert_eq!(payload.payload().sub, "LEIXG-529900T8BM49AURSDO55");
    }

    #[rstest]
    #[case::no_request_info(None)]
    #[case::empty_request_info(Some(IndexMap::new()))]
    #[case::no_eu_wrprc(Some(IndexMap::from_iter([("other".to_string(), Value::Null)])))]
    fn reject_missing_items_request_registration_certificate(#[case] request_info: Option<IndexMap<String, Value>>) {
        assert_matches!(
            RegistrationCertificateEnvelope::from_request_info(&items_request(request_info)),
            Err(RegistrationCertificateRequestParseError::Missing)
        );
    }

    #[rstest]
    #[case::text(Value::Text("base64url-is-not-a-byte-string".to_string()))]
    #[case::array(Value::Array(vec![Value::Integer(0.into())]))]
    #[case::null(Value::Null)]
    fn reject_items_request_registration_certificate_wrong_type(#[case] value: Value) {
        let request = items_request(Some(IndexMap::from_iter([(REQUEST_INFO_WRPRC_KEY.to_string(), value)])));

        assert_matches!(
            RegistrationCertificateEnvelope::from_request_info(&request),
            Err(RegistrationCertificateRequestParseError::NotByteString)
        );
    }

    #[rstest]
    #[case::empty(vec![])]
    #[case::malformed_cbor(vec![0xff])]
    #[case::not_a_cwt(vec![0xa0])]
    fn reject_malformed_items_request_registration_certificate(#[case] bytes: Vec<u8>) {
        let request = items_request(Some(IndexMap::from_iter([(
            REQUEST_INFO_WRPRC_KEY.to_string(),
            Value::Bytes(bytes),
        )])));

        assert_matches!(
            RegistrationCertificateEnvelope::from_request_info(&request),
            Err(RegistrationCertificateRequestParseError::Cwt(_))
        );
    }

    #[test]
    fn reject_jwt_items_request_registration_certificate() {
        let authority = MockRegistrationCertificateAuthority::new();
        let certificate = authority.sign_jwt(&RegistrationCertificateFixture(valid_payload_json()));
        assert_matches!(
            RegistrationCertificateEnvelope::from_slice(certificate.as_slice()),
            Ok(RegistrationCertificateEnvelope::Jwt(_))
        );
        let request = items_request(Some(IndexMap::from_iter([(
            REQUEST_INFO_WRPRC_KEY.to_string(),
            Value::Bytes(certificate),
        )])));

        assert_matches!(
            RegistrationCertificateEnvelope::from_request_info(&request),
            Err(RegistrationCertificateRequestParseError::Cwt(_))
        );
    }
}
