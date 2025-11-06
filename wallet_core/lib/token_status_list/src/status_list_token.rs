use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use chrono::serde::ts_seconds;
use chrono::serde::ts_seconds_option;
use derive_more::FromStr;
use serde::Deserialize;
use serde::Serialize;
use serde_with::DurationSeconds;
use serde_with::serde_as;

use crypto::EcdsaKey;
use crypto::server_keys::KeyPair;
use http_utils::urls::HttpsUri;
use jwt::JwtTyp;
use jwt::SignedJwt;
use jwt::UnverifiedJwt;
use jwt::error::JwtError;
use jwt::headers::HeaderWithX5c;

use crate::status_list::PackedStatusList;

static TOKEN_STATUS_LIST_JWT_TYP: &str = "statuslist+jwt";

/// A Status List Token embeds a Status List into a token that is cryptographically signed and protects the integrity of
/// the Status List.
///
/// <https://www.ietf.org/archive/id/draft-ietf-oauth-status-list-12.html#name-status-list-token>
#[derive(Debug, Clone, FromStr, Serialize, Deserialize)]
pub struct StatusListToken(UnverifiedJwt<StatusListClaims, HeaderWithX5c>);

impl StatusListToken {
    pub fn builder(sub: HttpsUri, status_list: PackedStatusList) -> StatusListTokenBuilder {
        StatusListTokenBuilder {
            exp: None,
            sub,
            ttl: None,
            status_list,
        }
    }
}

pub struct StatusListTokenBuilder {
    exp: Option<DateTime<Utc>>,
    sub: HttpsUri,
    ttl: Option<Duration>,
    status_list: PackedStatusList,
}

impl StatusListTokenBuilder {
    pub fn exp(mut self, exp: DateTime<Utc>) -> Self {
        self.exp = Some(exp);
        self
    }

    pub fn ttl(mut self, ttl: Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }

    pub async fn sign(self, keypair: &KeyPair<impl EcdsaKey>) -> Result<StatusListToken, JwtError> {
        let claims = StatusListClaims {
            iat: Utc::now(),
            exp: self.exp,
            sub: self.sub,
            ttl: self.ttl,
            status_list: self.status_list,
        };

        let jwt = SignedJwt::sign_with_certificate(&claims, keypair).await?;
        Ok(StatusListToken(jwt.into_unverified()))
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct StatusListClaims {
    #[serde(with = "ts_seconds")]
    iat: DateTime<Utc>,

    #[serde(with = "ts_seconds_option")]
    exp: Option<DateTime<Utc>>,

    /// The sub (subject) claim MUST specify the URI of the Status List Token. The value MUST be equal to that of the
    /// `uri` claim contained in the `status_list` claim of the Referenced Token
    sub: HttpsUri,

    /// If present, MUST specify the maximum amount of time, in seconds, that the Status List Token can be cached by a
    /// consumer before a fresh copy SHOULD be retrieved.
    #[serde_as(as = "Option<DurationSeconds<u64>>")]
    ttl: Option<Duration>,

    status_list: PackedStatusList,
}

impl JwtTyp for StatusListClaims {
    const TYP: &'static str = TOKEN_STATUS_LIST_JWT_TYP;
}

#[cfg(test)]
mod test {
    use base64::Engine;
    use base64::prelude::BASE64_STANDARD;
    use serde_json::json;

    use crypto::server_keys::generate::Ca;
    use jwt::DEFAULT_VALIDATIONS;
    use jwt::headers::HeaderWithTyp;

    use super::*;

    #[tokio::test]
    async fn test_status_list_token() {
        let ca = Ca::generate("test", Default::default()).unwrap();
        let keypair = ca.generate_status_list_mock().unwrap();

        let example_header = json!({
            "alg": "ES256",
            "typ": "statuslist+jwt",
            "x5c": vec![BASE64_STANDARD.encode(keypair.certificate().to_vec())],
        });
        let example_payload = json!({
            "exp": 2291720170_i64,
            "iat": 1686920170,
            "status_list": {
                "bits": 1,
                "lst": "eNrbuRgAAhcBXQ"
            },
            "sub": "https://example.com/statuslists/1",
            "ttl": 43200
        });

        let expected_header: HeaderWithX5c<HeaderWithTyp> = serde_json::from_value(example_header).unwrap();
        assert_eq!(expected_header.inner().typ, TOKEN_STATUS_LIST_JWT_TYP);

        let expected_claims: StatusListClaims = serde_json::from_value(example_payload).unwrap();

        let signed = StatusListToken::builder(expected_claims.sub.clone(), expected_claims.status_list.clone())
            .exp(expected_claims.exp.unwrap())
            .ttl(expected_claims.ttl.unwrap())
            .sign(&keypair)
            .await
            .unwrap();

        let verified = signed
            .0
            .into_verified(&keypair.private_key().verifying_key().into(), &DEFAULT_VALIDATIONS)
            .unwrap();
        assert_eq!(*verified.header(), expected_header);
        // the `iat` claim is set when signing the token
        assert_eq!(verified.payload().status_list, expected_claims.status_list);
        assert_eq!(verified.payload().sub, expected_claims.sub);
        assert_eq!(verified.payload().ttl, expected_claims.ttl);
        assert_eq!(verified.payload().exp, expected_claims.exp);
    }
}
