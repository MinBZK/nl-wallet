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
use crypto::EcdsaKey;
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

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use ciborium::Value;
    use cose::COSE_TYP_HEADER_LABEL;
    use cose::cbor_serialize;
    use coset::CoseSign1;
    use coset::Label;
    use coset::TaggedCborSerializable;
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

        let payload = cose.verify_and_parse(keypair.private_key().verifying_key()).unwrap();
        let claims = claims_map(&payload);
        assert_eq!(claims[&2], &Value::Text(sub.as_str().to_owned()));
        assert_eq!(claims[&4], &Value::from(SLT_EXP));
        assert_eq!(claims[&65533], &cbor_value(&CborStatusList(&status_list)));
        assert_eq!(claims[&65534], &Value::from(SLT_TTL));
    }
}
