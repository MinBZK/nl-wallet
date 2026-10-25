use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use chrono::serde::ts_seconds;
use chrono::serde::ts_seconds_option;
use crypto::EcdsaKey;
use crypto::server_keys::KeyPair;
use derive_more::AsRef;
use derive_more::FromStr;
use derive_more::Into;
use jwt::JwtTyp;
use jwt::SignedJwt;
use jwt::UnverifiedJwt;
use jwt::error::JwtParseError;
use jwt::error::JwtSignError;
use jwt::headers::HeaderWithX5c;
use serde::Deserialize;
use serde::Serialize;
use serde_with::DurationSeconds;
use serde_with::serde_as;
use url::Url;

use crate::status_list::JsonPackedStatusList;
use crate::status_list::PackedStatusList;
use crate::status_list_token::StatusListTokenClaims;

pub static TOKEN_STATUS_LIST_JWT_TYP: &str = "statuslist+jwt";

/// Status List Token encoded as JWT
///
/// <https://www.ietf.org/archive/id/draft-ietf-oauth-status-list-21.html#section-5.1>
#[derive(Debug, Clone, FromStr, AsRef, Into, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct StatusListJwt(UnverifiedJwt<StatusListJwtClaims, HeaderWithX5c>);

impl StatusListJwt {
    pub async fn sign(claims: StatusListTokenClaims, keypair: &KeyPair<impl EcdsaKey>) -> Result<Self, JwtSignError> {
        let jwt = SignedJwt::sign_with_certificate(&StatusListJwtClaims::from(claims), keypair).await?;
        Ok(Self(jwt.into_unverified()))
    }

    pub fn dangerous_parse_unverified(&self) -> Result<(HeaderWithX5c, StatusListTokenClaims), JwtParseError> {
        let (header, claims) = self.0.dangerous_parse_unverified()?;
        Ok((header, claims.into()))
    }
}

/// [`StatusListTokenClaims`] encoded as a JWT Claims Set.
///
/// <https://www.ietf.org/archive/id/draft-ietf-oauth-status-list-21.html#section-5.1>
#[serde_as]
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct StatusListJwtClaims {
    #[serde(with = "ts_seconds")]
    iat: DateTime<Utc>,

    #[serde(default)]
    #[serde(with = "ts_seconds_option")]
    exp: Option<DateTime<Utc>>,

    sub: Url,

    #[serde(default)]
    #[serde_as(as = "Option<DurationSeconds<u64>>")]
    ttl: Option<Duration>,

    #[serde(with = "JsonPackedStatusList")]
    status_list: PackedStatusList,
}

impl JwtTyp for StatusListJwtClaims {
    const TYP: &'static str = TOKEN_STATUS_LIST_JWT_TYP;
}

impl From<StatusListTokenClaims> for StatusListJwtClaims {
    fn from(claims: StatusListTokenClaims) -> Self {
        Self {
            iat: claims.iat,
            exp: claims.exp,
            sub: claims.sub,
            ttl: claims.ttl,
            status_list: claims.status_list,
        }
    }
}

impl From<StatusListJwtClaims> for StatusListTokenClaims {
    fn from(claims: StatusListJwtClaims) -> Self {
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
    use std::ops::Add;

    use chrono::DateTime;
    use chrono::Duration;
    use chrono::Utc;
    use crypto::trust_anchor::TrustAnchors;
    use jwt::DEFAULT_VALIDATION;
    use jwt::error::JwtX5cVerifyError;
    use url::Url;
    use utils::generator::Generator;

    use crate::status_list_jwt::StatusListJwt;
    use crate::status_list_token::StatusListTokenClaims;

    const EXP_LEEWAY: Duration = Duration::seconds(60);

    #[derive(Debug, thiserror::Error)]
    pub enum StatusListJwtVerificationError {
        #[error("JWT verification failed: {0}")]
        JwtVerification(#[from] JwtX5cVerifyError),

        #[error("JWT is expired")]
        Expired,

        #[error("JWT subject claim ('{sub}') does not match url claim of Reference Token ('{url}')")]
        UnexpectedSubject { sub: Box<Url>, url: Box<Url> },
    }

    impl StatusListJwt {
        pub fn parse_and_verify(
            &self,
            trust_anchors: &TrustAnchors,
            url: Url,
            time: &impl Generator<DateTime<Utc>>,
        ) -> Result<StatusListTokenClaims, StatusListJwtVerificationError> {
            // No extended key usage is required of the signer: neither the Token Status List draft nor ETSI TS 119 475
            // requires one, and the OID in the draft is not yet assigned.
            let (_, claims) = self.0.parse_and_verify_against_trust_anchors(
                trust_anchors,
                time,
                None,
                DEFAULT_VALIDATION.to_owned(),
            )?;
            let claims = StatusListTokenClaims::from(claims);

            if url != claims.sub {
                return Err(StatusListJwtVerificationError::UnexpectedSubject {
                    sub: claims.sub.into(),
                    url: url.into(),
                });
            }

            if claims.exp.is_some_and(|exp| exp.add(EXP_LEEWAY) < time.generate()) {
                return Err(StatusListJwtVerificationError::Expired);
            }

            Ok(claims)
        }
    }
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use base64::Engine;
    use base64::prelude::BASE64_STANDARD;
    use crypto::EcdsaKey;
    use crypto::server_keys::KeyPair;
    use jwt::headers::HeaderWithTyp;
    use jwt::headers::HeaderWithX5c;
    use serde_json::json;

    use crate::status_list_jwt::StatusListJwt;
    use crate::status_list_jwt::StatusListJwtClaims;
    use crate::status_list_jwt::TOKEN_STATUS_LIST_JWT_TYP;
    use crate::status_list_token::StatusListToken;
    use crate::status_list_token::StatusListTokenClaims;

    pub async fn create_status_list_jwt<S>(
        keypair: &KeyPair<S>,
        exp: Option<i64>,
        ttl: Option<i64>,
    ) -> (HeaderWithX5c<HeaderWithTyp>, StatusListTokenClaims, StatusListJwt)
    where
        S: EcdsaKey,
    {
        let example_header = json!({
            "alg": "ES256",
            "typ": "statuslist+jwt",
            "x5c": vec![BASE64_STANDARD.encode(keypair.certificate().to_vec())],
        });
        let example_payload = json!({
            "exp": exp,
            "iat": 1686920170,
            "status_list": {
                "bits": 1,
                "lst": "eNrbuRgAAhcBXQ"
            },
            "sub": "https://example.com/statuslists/1",
            "ttl": ttl,
        });

        let expected_header: HeaderWithX5c<HeaderWithTyp> = serde_json::from_value(example_header).unwrap();
        assert_eq!(expected_header.inner().typ, TOKEN_STATUS_LIST_JWT_TYP);

        let expected_claims: StatusListTokenClaims = serde_json::from_value::<StatusListJwtClaims>(example_payload)
            .unwrap()
            .into();

        let claims = StatusListToken::builder(expected_claims.sub.clone(), expected_claims.status_list.clone())
            .exp(expected_claims.exp)
            .ttl(expected_claims.ttl)
            .build();
        let status_list_jwt = StatusListJwt::sign(claims, keypair).await.unwrap();

        (expected_header, expected_claims, status_list_jwt)
    }
}

#[cfg(all(test, feature = "verification"))]
mod test {
    use std::assert_matches;
    use std::ops::Add;

    use chrono::DateTime;
    use chrono::Days;
    use crypto::PublicKey;
    use crypto::server_keys::generate::Ca;
    use crypto::trust_anchor::TrustAnchors;
    use jwt::ESP256_ONLY_VALIDATION;
    use jwt::JwtDecodingKey;
    use jwt::error::JwtX5cVerifyError;
    use utils::generator::mock::MockTimeGenerator;

    use crate::status_list_jwt::mock::create_status_list_jwt;
    use crate::status_list_jwt::verification::StatusListJwtVerificationError;

    const SLT_EXP: i64 = 2291720170;
    const SLT_TTL: i64 = 43200;

    #[tokio::test]
    async fn test_status_list_jwt() {
        let ca = Ca::generate_mock();
        let keypair = ca.generate_issuer_status_list_mock().unwrap();

        let (expected_header, expected_claims, signed) =
            create_status_list_jwt(&keypair, Some(SLT_EXP), Some(SLT_TTL)).await;

        let verified = signed
            .0
            .into_verified(
                JwtDecodingKey::from(PublicKey::from(*keypair.private_key().verifying_key())),
                &*ESP256_ONLY_VALIDATION,
            )
            .unwrap();
        assert_eq!(*verified.header(), expected_header);
        // the `iat` claim is set when signing the token
        assert_eq!(verified.payload().status_list, expected_claims.status_list);
        assert_eq!(verified.payload().sub, expected_claims.sub);
        assert_eq!(verified.payload().ttl, expected_claims.ttl);
        assert_eq!(verified.payload().exp, expected_claims.exp);
    }

    #[tokio::test]
    async fn test_status_list_jwt_verification() {
        let ca = Ca::generate_mock();
        let keypair = ca.generate_issuer_status_list_mock().unwrap();

        let (_, expected_claims, signed) = create_status_list_jwt(&keypair, Some(SLT_EXP), Some(SLT_TTL)).await;

        let err = signed
            .parse_and_verify(
                &TrustAnchors::empty(),
                expected_claims.sub.clone(),
                &MockTimeGenerator::default(),
            )
            .expect_err("should not verify for empty trust anchors");
        assert_matches!(
            err,
            StatusListJwtVerificationError::JwtVerification(JwtX5cVerifyError::CertificateValidation(_))
        );

        let err = signed
            .parse_and_verify(
                &ca.to_trust_anchors(),
                "http://example.com/sub".parse().unwrap(),
                &MockTimeGenerator::default(),
            )
            .expect_err("should not verify for different sub claim");
        assert_matches!(err, StatusListJwtVerificationError::UnexpectedSubject { .. });

        let err = signed
            .parse_and_verify(
                &ca.to_trust_anchors(),
                expected_claims.sub.clone(),
                &MockTimeGenerator::new(DateTime::from_timestamp(SLT_EXP, 0).unwrap().add(Days::new(1))),
            )
            .expect_err("should not verify when jwt is expired");
        assert_matches!(err, StatusListJwtVerificationError::Expired);

        signed
            .parse_and_verify(
                &ca.to_trust_anchors(),
                expected_claims.sub,
                &MockTimeGenerator::default(),
            )
            .unwrap();
    }

    #[tokio::test]
    async fn test_status_list_jwt_verification_without_status_list_usage() {
        let ca = Ca::generate_mock();
        let keypair = ca.generate_issuer_mock().unwrap();

        let (_, expected_claims, signed) = create_status_list_jwt(&keypair, Some(SLT_EXP), Some(SLT_TTL)).await;

        signed
            .parse_and_verify(
                &ca.to_trust_anchors(),
                expected_claims.sub,
                &MockTimeGenerator::default(),
            )
            .unwrap();
    }
}
