use chrono::DateTime;
use chrono::Utc;
use crypto::EcdsaKey;
use crypto::server_keys::KeyPair;
use jsonwebtoken::Header;
use serde::Deserialize;
use serde::Serialize;
use utils::generator::Generator;

use crate::JwtTyp;
use crate::SignedJwt;
use crate::error::JwtParseError;
use crate::error::JwtSignError;
use crate::headers::HeaderWithTyp;
use crate::headers::HeaderWithX5c;

/// JAdES B-B JWT type string, to be used as the `typ` field in JAdES B-B JWT headers.
///
/// Use this by implementing `JwtTyp` for the payload type:
/// ```example
/// use crate::jwt::JAdES_B_B_JWT_TYP;
///
/// struct MyPayload {
///     // ...
/// }
///
/// impl JwtTyp for MyPayload {
///     const TYP: &'static str = JADES_B_B_JWT_TYP;
/// }
/// ```
pub const JADES_B_B_JWT_TYP: &str = "rc-wrp+jwt";

/// JAdES B-B JWT header: a standard [`HeaderWithX5c`] carrying JAdES-specific fields in its inner
/// header.
///
/// Access JAdES fields via [`HeaderWithX5c::inner`]: `header.inner().iat`.
pub type JadesbbHeader = HeaderWithX5c<JadesbbInnerHeader>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JadesbbInnerHeader {
    #[serde(flatten)]
    pub inner: HeaderWithTyp,

    // required by the spec, but optional for interoperability concerns. Note: after decoding this will always be
    // `None` because `jsonwebtoken::Header` does not include it
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "chrono::serde::ts_seconds_option"
    )]
    pub iat: Option<DateTime<Utc>>,
    // `sigT` field is not allowed, but ignored for interoperability concerns
}

impl From<JadesbbInnerHeader> for Header {
    fn from(value: JadesbbInnerHeader) -> Self {
        let mut header: Header = value.inner.into();

        // manually set `iat` as an extra field
        if let Some(iat) = value.iat {
            header.extras.insert("iat".to_string(), iat.timestamp());
        }

        header
    }
}

impl TryFrom<Header> for JadesbbInnerHeader {
    type Error = JwtParseError;

    fn try_from(value: Header) -> Result<Self, Self::Error> {
        let iat = value
            .extras
            .get::<i64>("iat")
            .map_err(JwtParseError::InvalidIat)? // the `iat` field is present but not a valid timestamp
            .map(|t| DateTime::from_timestamp(t, 0).ok_or(JwtParseError::IatOutOfRange(t))) // the `iat` field is a valid i64 but out of range for DateTime<Utc>
            .transpose()?;

        Ok(JadesbbInnerHeader {
            inner: HeaderWithTyp::try_from(value)?,
            iat,
        })
    }
}

impl<C: Serialize + JwtTyp> SignedJwt<C, JadesbbHeader> {
    pub async fn sign_with_iat<K: EcdsaKey>(
        payload: &C,
        keypair: &KeyPair<K>,
        time: &impl Generator<DateTime<Utc>>,
    ) -> Result<SignedJwt<C, JadesbbHeader>, JwtSignError> {
        let header = JadesbbInnerHeader {
            inner: HeaderWithTyp::new::<C>(),
            iat: Some(time.generate()),
        };

        SignedJwt::<C, JadesbbHeader>::sign_with_header_and_certificate(payload, header, keypair).await
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;
    use std::panic;

    use base64::prelude::*;
    use chrono::TimeZone;
    use chrono::Timelike;
    use chrono::Utc;
    use crypto::server_keys::generate::Ca;
    use crypto::trust_anchor::TrustAnchors;
    use jsonwebtoken::Algorithm;
    use serde_json::Value;
    use serde_json::json;
    use utils::generator::mock::MockTimeGenerator;
    use utils::vec_nonempty;

    use super::*;
    use crate::DEFAULT_VALIDATIONS;
    use crate::UnverifiedJwt;
    use crate::headers::HeaderWithX5c;

    fn test_header(iat: i64) -> JadesbbHeader {
        let inner = JadesbbInnerHeader {
            inner: HeaderWithTyp::default(),
            iat: Some(Utc.timestamp_opt(iat, 0).unwrap()),
        };

        let ca = Ca::generate_mock();
        let keypair = ca.generate_reader_mock().unwrap();
        HeaderWithX5c::new(inner, vec_nonempty![keypair.certificate().to_owned()])
    }

    #[test]
    fn test_serialize_roundtrip_with_iat() {
        let iat = 1337;
        let header = test_header(iat);

        let json = serde_json::to_value(&header).unwrap();

        assert!(json.get("iat").is_some_and(|v| *v == Value::Number(iat.into())));
        assert!(json.get("typ").is_some()); // will be set to "JAdES-B-B" by `sign_with_header_and_certificate`, but is "jwt" if not signed
        assert!(json.get("x5c").is_some());
        assert!(json.get("alg").is_some_and(|v| *v == Value::String("ES256".to_owned())));

        let header: JadesbbHeader = serde_json::from_value(json).unwrap();
        assert_eq!(header.inner().iat, Some(chrono::Utc.timestamp_opt(iat, 0).unwrap()));
        assert_eq!(header.x5c.len(), NonZeroUsize::MIN);
        assert_eq!(header.inner().inner.alg, Algorithm::ES256);
    }

    #[test]
    fn test_serialize_roundtrip_without_iat() {
        let inner = JadesbbInnerHeader {
            inner: HeaderWithTyp::default(),
            iat: None,
        };

        let ca = Ca::generate_mock();
        let keypair = ca.generate_reader_mock().unwrap();
        let header = HeaderWithX5c::new(inner, vec_nonempty![keypair.certificate().to_owned()]);

        let json = serde_json::to_value(&header).unwrap();

        assert!(json.get("iat").is_none());
        assert!(json.get("typ").is_some()); // will be set to "JAdES-B-B" by `sign_with_header_and_certificate`, but is "jwt" if not signed
        assert!(json.get("x5c").is_some());
        assert!(json.get("alg").is_some_and(|v| *v == Value::String("ES256".to_owned())));

        let header: JadesbbHeader = serde_json::from_value(json).unwrap();
        assert_eq!(header.inner().iat, None);
        assert_eq!(header.x5c.len(), NonZeroUsize::MIN);
        assert_eq!(header.inner().inner.alg, Algorithm::ES256);
    }

    #[test]
    fn test_roundtrip_to_string() {
        let iat = 1337;
        let header = test_header(iat);

        let json = serde_json::to_string(&header).unwrap();
        let roundtripped: JadesbbHeader = serde_json::from_str(&json).unwrap();
        assert_eq!(header, roundtripped);
    }

    #[test]
    fn test_deserialize_without_iat() {
        let json = json!({
            "typ": "rc-wrp+jwt",
            "alg": "ES256"
        });

        let header: JadesbbInnerHeader = serde_json::from_value(json).unwrap();
        assert_eq!(header.iat, None);
        assert_eq!(header.inner.alg, Algorithm::ES256);
        assert_eq!(header.inner.typ, JADES_B_B_JWT_TYP);
    }

    #[test]
    fn test_deserialize_with_sig_t_field() {
        let json = json!({
            "typ": "rc-wrp+jwt",
            "alg": "ES256",
            "sigT": "2024-11-14T12:00:00Z"
        });

        let header: JadesbbInnerHeader = serde_json::from_value(json).unwrap();
        assert_eq!(header.iat, None);
        assert_eq!(header.inner.alg, Algorithm::ES256);
        assert_eq!(header.inner.typ, JADES_B_B_JWT_TYP);
    }

    #[test]
    fn test_missing_x5c_fails_deserialization() {
        let json_val = json!({
            "alg": "ES256",
            "typ": "rc-wrp+jwt",
            "iat": 1337,
            // "x5c" intentionally absent
        });
        let result: Result<JadesbbHeader, _> = serde_json::from_value(json_val);
        assert!(result.is_err(), "deserialization must fail without x5c");
    }

    #[test]
    fn test_empty_x5c_fails_deserialization() {
        let json_val = json!({
            "alg": "ES256",
            "typ": "rc-wrp+jwt",
            "x5c": [],
            "iat": 1_700_000_000
        });
        let result: Result<JadesbbHeader, _> = serde_json::from_value(json_val);
        assert!(result.is_err(), "deserialization must fail with an empty x5c array");
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct ToyJadesbbPayload {}

    impl JwtTyp for ToyJadesbbPayload {
        const TYP: &'static str = JADES_B_B_JWT_TYP;
    }

    #[tokio::test]
    async fn test_sign_and_verify_jades_b_b_token_roundtrip() {
        let ca = Ca::generate_mock();
        let keypair = ca.generate_wrpac_issuer_mock().unwrap();

        let toy_payload = ToyJadesbbPayload {};

        let now = Utc.timestamp_opt(1337, 900_000_000).unwrap();
        let signed_jwt =
            SignedJwt::<_, JadesbbHeader>::sign_with_iat(&toy_payload, &keypair, &MockTimeGenerator::new(now))
                .await
                .unwrap();

        let verified = signed_jwt.clone().into_verified();
        assert!(verified.header().inner().iat.is_some());
        assert_eq!(verified.header().inner().inner.typ, JADES_B_B_JWT_TYP);
        assert_eq!(verified.header().inner().inner.alg, Algorithm::ES256);

        let unverified = signed_jwt.into_unverified();
        let (header, payload) = unverified
            .parse_and_verify_against_trust_anchors(
                &TrustAnchors::from(&ca),
                &MockTimeGenerator::default(),
                None,
                &DEFAULT_VALIDATIONS,
            )
            .unwrap();

        assert_eq!(header.inner().iat, Some(now.with_nanosecond(0).unwrap())); // will be rounded down to 0 ns
        assert_eq!(header.inner().inner.typ, JADES_B_B_JWT_TYP);
        assert_eq!(header.inner().inner.alg, Algorithm::ES256);
        assert_eq!(payload, toy_payload);
    }

    #[tokio::test]
    async fn test_verify_jades_b_b_without_iat() {
        let ca = Ca::generate_mock();
        let keypair = ca.generate_wrpac_issuer_mock().unwrap();

        let toy_payload = ToyJadesbbPayload {};

        let signed_jwt = SignedJwt::<_, HeaderWithX5c>::sign_with_certificate(&toy_payload, &keypair)
            .await
            .unwrap();

        let unverified = signed_jwt.into_unverified();

        let json: serde_json::Value = serde_json::from_slice(
            &BASE64_URL_SAFE_NO_PAD
                .decode(unverified.serialization().split('.').take(1).last().unwrap())
                .unwrap(),
        )
        .unwrap();

        assert!(json["iat"].is_null());
        assert_eq!(json["typ"], JADES_B_B_JWT_TYP);

        // reinterpret as to be able to parse the JAdES-B-B header
        let unverified: UnverifiedJwt<ToyJadesbbPayload, JadesbbHeader> = unverified.serialization().parse().unwrap();
        let (header, payload): (JadesbbHeader, ToyJadesbbPayload) = unverified
            .parse_and_verify_against_trust_anchors(
                &TrustAnchors::from(&ca),
                &MockTimeGenerator::default(),
                None,
                &DEFAULT_VALIDATIONS,
            )
            .unwrap(); // should parse even without an `iat` field

        assert!(header.inner().iat.is_none());
        assert_eq!(header.inner().inner.typ, JADES_B_B_JWT_TYP);
        assert_eq!(header.inner().inner.alg, Algorithm::ES256);
        assert_eq!(payload, toy_payload);
    }
}
