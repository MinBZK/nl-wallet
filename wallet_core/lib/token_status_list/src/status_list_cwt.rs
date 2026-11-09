use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use chrono::serde::ts_seconds;
use chrono::serde::ts_seconds_option;
use ciborium::tag::Required;
use cose::CborIndexedFields;
use cose::CborIntMap;
use cose::CoseError;
use cose::CoseTyp;
use cose::TaggedCoseSign1;
use cose::TypedCose;
use crypto::keys::EcdsaKey;
use crypto::server_keys::KeyPair;
use derive_more::AsRef;
use derive_more::Into;
use mdoc_derive::CborIndexedFields;
use serde::Deserialize;
use serde::Serialize;
use serde_with::DurationSeconds;
use serde_with::serde_as;
use url::Url;

use crate::status_list::CborPackedStatusList;
use crate::status_list::PackedStatusList;
use crate::status_list_token::StatusListTokenClaims;

/// COSE `typ` header parameter value (RFC 9596) for a Status List Token in CWT format.
///
/// <https://www.ietf.org/archive/id/draft-ietf-oauth-status-list-21.html#section-5.2>
pub static TOKEN_STATUS_LIST_CWT_TYP: &str = "application/statuslist+cwt";

/// Status List Token encoded as CWT
///
/// <https://www.ietf.org/archive/id/draft-ietf-oauth-status-list-21.html#section-5.2>
#[derive(Debug, Clone, AsRef, Into, Serialize, Deserialize)]
pub struct StatusListCwt(TaggedCoseSign1<CborIntMap<StatusListCwtClaims>>);

impl StatusListCwt {
    pub async fn sign(claims: StatusListTokenClaims, keypair: &KeyPair<impl EcdsaKey>) -> Result<Self, CoseError> {
        let claims = CborIntMap(StatusListCwtClaims::from(claims));
        let cose = TypedCose::sign_with_typ_and_certificate(&claims, keypair).await?;
        Ok(Self(Required(cose)))
    }

    /// Parse a tagged `COSE_Sign1` structure, without verifying it.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, ciborium::de::Error<std::io::Error>> {
        ciborium::from_reader(bytes)
    }
}

/// [`StatusListTokenClaims`] encoded as a CWT Claims Set, using the claim keys from the IANA "CBOR Web Token (CWT)
/// Claims" registry (RFC 8392).
///
/// Note that the keys of `status_list` (65533) and `ttl` (65534) are requested by the status list spec, but not yet
/// assigned by IANA.
///
/// <https://www.ietf.org/archive/id/draft-ietf-oauth-status-list-21.html#section-5.2>
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, CborIndexedFields)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct StatusListCwtClaims {
    #[cbor_index = 2]
    sub: Url,

    #[cbor_index = 4]
    #[serde(default)]
    #[serde(with = "ts_seconds_option")]
    exp: Option<DateTime<Utc>>,

    #[cbor_index = 6]
    #[serde(with = "ts_seconds")]
    iat: DateTime<Utc>,

    #[cbor_index = 65533]
    #[serde(with = "CborPackedStatusList")]
    status_list: PackedStatusList,

    #[cbor_index = 65534]
    #[serde(default)]
    #[serde_as(as = "Option<DurationSeconds<u64>>")]
    ttl: Option<Duration>,
}

impl CoseTyp for StatusListCwtClaims {
    const TYP: &'static str = TOKEN_STATUS_LIST_CWT_TYP;
}

impl From<StatusListTokenClaims> for StatusListCwtClaims {
    fn from(claims: StatusListTokenClaims) -> Self {
        Self {
            sub: claims.sub,
            exp: claims.exp,
            iat: claims.iat,
            status_list: claims.status_list,
            ttl: claims.ttl,
        }
    }
}

impl From<StatusListCwtClaims> for StatusListTokenClaims {
    fn from(claims: StatusListCwtClaims) -> Self {
        Self {
            iat: claims.iat,
            exp: claims.exp,
            sub: claims.sub,
            ttl: claims.ttl,
            status_list: claims.status_list,
        }
    }
}

#[cfg(feature = "verification")]
pub mod verification {
    use chrono::DateTime;
    use chrono::Utc;
    use cose::CoseError;
    use crypto::trust_anchor::TrustAnchors;
    use url::Url;
    use utils::generator::Generator;

    use crate::status_list_cwt::StatusListCwt;
    use crate::status_list_token::StatusListTokenClaims;
    use crate::status_list_token::verification::StatusListTokenClaimsVerificationError;

    #[derive(Debug, thiserror::Error)]
    pub enum StatusListCwtVerificationError {
        #[error("CWT verification failed: {0}")]
        CwtVerification(#[source] CoseError),

        #[error("CWT claims verification failed: {0}")]
        Claims(#[source] StatusListTokenClaimsVerificationError),
    }

    impl StatusListCwt {
        pub fn parse_and_verify(
            &self,
            trust_anchors: &TrustAnchors,
            url: &Url,
            time: &impl Generator<DateTime<Utc>>,
        ) -> Result<StatusListTokenClaims, StatusListCwtVerificationError> {
            let claims = self
                .0
                .0
                .verify_with_typ_against_trust_anchors(trust_anchors, time, None)
                .map_err(StatusListCwtVerificationError::CwtVerification)?;

            let claims = StatusListTokenClaims::from(claims.0);
            claims
                .verify(url, time)
                .map_err(StatusListCwtVerificationError::Claims)?;

            Ok(claims)
        }
    }
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use chrono::DateTime;
    use chrono::Utc;
    use crypto::keys::EcdsaKey;
    use crypto::server_keys::KeyPair;
    use utils::generator::TimeGenerator;

    use crate::status_list::PackedStatusList;
    use crate::status_list_cwt::StatusListCwt;
    use crate::status_list_token::StatusListToken;
    use crate::status_list_token::StatusListTokenClaims;

    pub async fn mock_status_list_cwt<S>(
        keypair: &KeyPair<S>,
        exp: Option<DateTime<Utc>>,
    ) -> (StatusListCwt, StatusListTokenClaims)
    where
        S: EcdsaKey,
    {
        let claims = StatusListToken::builder(
            "https://example.com/statuslists/1".parse().unwrap(),
            PackedStatusList::new(10),
        )
        .exp(exp)
        .build(&TimeGenerator);
        let status_list_cwt = StatusListCwt::sign(claims.clone(), keypair).await.unwrap();

        (status_list_cwt, claims)
    }
}

#[cfg(all(test, feature = "verification"))]
mod verification_test {
    use std::assert_matches;
    use std::ops::Add;

    use chrono::Days;
    use chrono::SubsecRound;
    use chrono::Utc;
    use crypto::server_keys::generate::Ca;
    use crypto::trust_anchor::TrustAnchors;
    use utils::generator::mock::MockTimeGenerator;

    use crate::status_list_cwt::StatusListCwt;
    use crate::status_list_cwt::mock::mock_status_list_cwt;
    use crate::status_list_cwt::verification::StatusListCwtVerificationError;
    use crate::status_list_token::StatusListTokenClaims;
    use crate::status_list_token::verification::StatusListTokenClaimsVerificationError;

    async fn mock_signed_status_list_cwt() -> (StatusListCwt, StatusListTokenClaims, Ca) {
        let ca = Ca::generate_mock();
        let keypair = ca.generate_issuer_status_list_mock().unwrap();
        let exp = Utc::now().add(Days::new(1)).trunc_subsecs(0);
        let (signed, claims) = mock_status_list_cwt(&keypair, Some(exp)).await;

        (signed, claims, ca)
    }

    #[tokio::test]
    async fn test_status_list_cwt_verification() {
        let (signed, expected_claims, ca) = mock_signed_status_list_cwt().await;

        let claims = signed
            .parse_and_verify(
                &ca.to_trust_anchors(),
                &expected_claims.sub,
                &MockTimeGenerator::default(),
            )
            .unwrap();

        // `iat` is encoded in whole seconds, so it loses the subseconds of the signing time
        let expected_claims = StatusListTokenClaims {
            iat: expected_claims.iat.trunc_subsecs(0),
            ..expected_claims
        };
        assert_eq!(claims, expected_claims);
    }

    #[tokio::test]
    async fn test_status_list_cwt_verification_rejects_untrusted_signer() {
        let (signed, claims, _) = mock_signed_status_list_cwt().await;

        let err = signed
            .parse_and_verify(&TrustAnchors::empty(), &claims.sub, &MockTimeGenerator::default())
            .unwrap_err();

        assert_matches!(err, StatusListCwtVerificationError::CwtVerification(_));
    }

    #[tokio::test]
    async fn test_status_list_cwt_verification_rejects_unexpected_subject() {
        let (signed, _, ca) = mock_signed_status_list_cwt().await;

        let err = signed
            .parse_and_verify(
                &ca.to_trust_anchors(),
                &"http://example.com/sub".parse().unwrap(),
                &MockTimeGenerator::default(),
            )
            .unwrap_err();

        assert_matches!(
            err,
            StatusListCwtVerificationError::Claims(StatusListTokenClaimsVerificationError::UnexpectedSubject { .. })
        );
    }

    #[tokio::test]
    async fn test_status_list_cwt_verification_rejects_expired() {
        let (signed, claims, ca) = mock_signed_status_list_cwt().await;

        let err = signed
            .parse_and_verify(
                &ca.to_trust_anchors(),
                &claims.sub,
                &MockTimeGenerator::new(claims.exp.unwrap().add(Days::new(1))),
            )
            .unwrap_err();

        assert_matches!(
            err,
            StatusListCwtVerificationError::Claims(StatusListTokenClaimsVerificationError::Expired)
        );
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use ciborium::Value;
    use cose::COSE_TYP_HEADER_LABEL;
    use cose::cbor_serialize;
    use coset::CoseSign1;
    use coset::Label;
    use coset::TaggedCborSerializable;
    use crypto::keys::PublicKey;
    use crypto::server_keys::generate::Ca;
    use utils::generator::mock::MockTimeGenerator;

    use super::*;
    use crate::status_list_token::StatusListToken;

    const SLT_EXP: i64 = 2_291_720_170;
    const SLT_TTL: u64 = 43_200;

    #[derive(Serialize)]
    struct CborStatusList<'a>(#[serde(with = "CborPackedStatusList")] &'a PackedStatusList);

    fn cbor_value(value: &impl Serialize) -> Value {
        Value::serialized(value).unwrap()
    }

    fn claims_map(value: &Value) -> HashMap<i128, &Value> {
        value
            .as_map()
            .unwrap()
            .iter()
            .map(|(key, value)| (key.as_integer().unwrap().into(), value))
            .collect()
    }

    /// Check that the CBOR encoding of the claims uses the CWT Claim Keys registered by the status list spec (2, 4, 6,
    /// 65533, 65534) instead of the string keys used by the JWT profile.
    ///
    /// <https://www.ietf.org/archive/id/draft-ietf-oauth-status-list-21.html#section-5.2>
    #[test]
    fn test_status_list_cwt_claims_use_registered_claim_keys() {
        let status_list = PackedStatusList::new(10);
        let claims = CborIntMap(StatusListCwtClaims::from(StatusListTokenClaims {
            iat: DateTime::from_timestamp(1_686_920_170, 0).unwrap(),
            exp: Some(DateTime::from_timestamp(SLT_EXP, 0).unwrap()),
            sub: "https://example.com/statuslists/1".parse().unwrap(),
            ttl: Some(Duration::from_secs(SLT_TTL)),
            status_list: status_list.clone(),
        }));

        let value = cbor_value(&claims);
        let claims = claims_map(&value);

        assert_eq!(
            claims,
            HashMap::from([
                (2, &Value::Text("https://example.com/statuslists/1".to_owned())),
                (4, &Value::from(SLT_EXP)),
                (6, &Value::from(1_686_920_170)),
                (65533, &cbor_value(&CborStatusList(&status_list))),
                (65534, &Value::from(SLT_TTL)),
            ])
        );
    }

    #[test]
    fn test_status_list_cwt_claims_ignore_unknown_claims() {
        let claims = CborIntMap(StatusListCwtClaims::from(StatusListTokenClaims {
            iat: DateTime::from_timestamp(1_686_920_170, 0).unwrap(),
            exp: Some(DateTime::from_timestamp(SLT_EXP, 0).unwrap()),
            sub: "https://example.com/statuslists/1".parse().unwrap(),
            ttl: Some(Duration::from_secs(SLT_TTL)),
            status_list: PackedStatusList::new(10),
        }));

        let Value::Map(mut map) = cbor_value(&claims) else {
            panic!("claims should serialize to a map");
        };
        map.push((Value::from(1), Value::Text("https://example.com".to_owned())));
        map.push((Value::Text("private".to_owned()), Value::Bool(true)));

        let deserialized = Value::Map(map)
            .deserialized::<CborIntMap<StatusListCwtClaims>>()
            .unwrap();

        assert_eq!(deserialized.0, claims.0);
    }

    #[tokio::test]
    async fn test_status_list_cwt_is_tagged_typed_and_signed_by_key_pair() {
        let ca = Ca::generate_mock();
        let keypair = ca.generate_issuer_status_list_mock().unwrap();
        let sub: Url = "https://example.com/statuslists/1".parse().unwrap();
        let status_list = PackedStatusList::new(10);

        let claims = StatusListToken::builder(sub.clone(), status_list.clone())
            .exp(Some(DateTime::from_timestamp(SLT_EXP, 0).unwrap()))
            .ttl(Some(Duration::from_secs(SLT_TTL)))
            .build(&MockTimeGenerator::default());
        let token = StatusListCwt::sign(claims, &keypair).await.unwrap();

        let tagged = cbor_serialize(&token).unwrap();
        let cose = TypedCose::<CoseSign1, Value>::from(CoseSign1::from_tagged_slice(&tagged).unwrap());
        let typ = cose
            .protected_header()
            .rest
            .iter()
            .find(|(label, _)| label == &Label::Int(COSE_TYP_HEADER_LABEL))
            .and_then(|(_, value)| value.as_text());
        assert_eq!(typ, Some(TOKEN_STATUS_LIST_CWT_TYP));

        let payload = cose
            .verify_and_parse(&PublicKey::from(keypair.verifying_key().await.unwrap()))
            .unwrap();
        let claims = claims_map(&payload);
        assert_eq!(claims[&2], &Value::Text(sub.as_str().to_owned()));
        assert_eq!(claims[&4], &Value::from(SLT_EXP));
        assert_eq!(claims[&65533], &cbor_value(&CborStatusList(&status_list)));
        assert_eq!(claims[&65534], &Value::from(SLT_TTL));
    }
}
