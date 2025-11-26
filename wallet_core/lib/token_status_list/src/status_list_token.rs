use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use chrono::serde::ts_seconds;
use chrono::serde::ts_seconds_option;
use derive_more::AsRef;
use derive_more::FromStr;
use serde::Deserialize;
use serde::Serialize;
use serde_with::DurationSeconds;
use serde_with::serde_as;
use url::Url;

use crypto::EcdsaKey;
use crypto::server_keys::KeyPair;
use jwt::JwtTyp;
use jwt::SignedJwt;
use jwt::UnverifiedJwt;
use jwt::error::JwtError;
use jwt::headers::HeaderWithX5c;

use crate::status_list::PackedStatusList;

pub static TOKEN_STATUS_LIST_JWT_TYP: &str = "statuslist+jwt";

/// A Status List Token embeds a Status List into a token that is cryptographically signed and protects the integrity of
/// the Status List.
///
/// <https://www.ietf.org/archive/id/draft-ietf-oauth-status-list-12.html#name-status-list-token>
#[derive(Debug, Clone, FromStr, AsRef, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct StatusListToken(UnverifiedJwt<StatusListClaims, HeaderWithX5c>);

impl StatusListToken {
    pub fn builder(sub: Url, status_list: PackedStatusList) -> StatusListTokenBuilder {
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
    sub: Url,
    ttl: Option<Duration>,
    status_list: PackedStatusList,
}

impl StatusListTokenBuilder {
    pub fn exp(mut self, exp: Option<DateTime<Utc>>) -> Self {
        self.exp = exp;
        self
    }

    pub fn ttl(mut self, ttl: Option<Duration>) -> Self {
        self.ttl = ttl;
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
    pub iat: DateTime<Utc>,

    #[serde(with = "ts_seconds_option")]
    pub exp: Option<DateTime<Utc>>,

    /// The sub (subject) claim MUST specify the URI of the Status List Token. The value MUST be equal to that of the
    /// `uri` claim contained in the `status_list` claim of the Referenced Token
    pub sub: Url,

    /// If present, MUST specify the maximum amount of time, in seconds, that the Status List Token can be cached by a
    /// consumer before a fresh copy SHOULD be retrieved.
    #[serde_as(as = "Option<DurationSeconds<u64>>")]
    pub ttl: Option<Duration>,

    pub status_list: PackedStatusList,
}

impl JwtTyp for StatusListClaims {
    const TYP: &'static str = TOKEN_STATUS_LIST_JWT_TYP;
}

#[cfg(feature = "verification")]
pub mod verification {
    use std::ops::Add;

    use chrono::DateTime;
    use chrono::Duration;
    use chrono::Utc;
    use rustls_pki_types::TrustAnchor;
    use url::Url;

    use crypto::x509::BorrowingCertificate;
    use crypto::x509::CertificateError;
    use crypto::x509::CertificateUsage;
    use jwt::DEFAULT_VALIDATIONS;
    use jwt::error::JwtX5cError;
    use utils::generator::Generator;

    use crate::status_list::PackedStatusList;
    use crate::status_list_token::StatusListToken;

    const EXP_LEEWAY: Duration = Duration::seconds(60);

    #[derive(Debug, thiserror::Error)]
    pub enum StatusListTokenVerificationError {
        #[error("JWT verification failed: {0}")]
        JwtVerification(#[from] JwtX5cError),

        #[error("JWT is expired")]
        Expired,

        #[error("JWT subject claim ('{sub}') does not match url claim of Reference Token ('{url}')")]
        UnexpectedSubject { sub: String, url: String },

        #[error("DN is missing in certificate")]
        MissingDN(#[source] CertificateError),

        #[error("DN from SLT ('{slt}') is different from attestation ('{attestation}')")]
        DifferentDN { slt: String, attestation: String },
    }

    impl StatusListToken {
        pub fn parse_and_verify(
            &self,
            issuer_trust_anchors: &[TrustAnchor],
            attestation_signing_certificate: &BorrowingCertificate,
            url: &Url,
            time: &impl Generator<DateTime<Utc>>,
        ) -> Result<PackedStatusList, StatusListTokenVerificationError> {
            let (header, claims) = self.0.parse_and_verify_against_trust_anchors(
                issuer_trust_anchors,
                time,
                CertificateUsage::OAuthStatusSigning,
                &DEFAULT_VALIDATIONS,
            )?;

            let slt_dn = header
                .x5c
                .first()
                .distinguished_name()
                .map_err(StatusListTokenVerificationError::MissingDN)?;
            let attestation_dn = attestation_signing_certificate
                .distinguished_name()
                .map_err(StatusListTokenVerificationError::MissingDN)?;
            if slt_dn != attestation_dn {
                return Err(StatusListTokenVerificationError::DifferentDN {
                    slt: slt_dn,
                    attestation: attestation_dn,
                });
            }

            if *url != claims.sub {
                return Err(StatusListTokenVerificationError::UnexpectedSubject {
                    sub: claims.sub.to_string(),
                    url: url.to_string(),
                });
            }

            if claims.exp.is_some_and(|exp| exp.add(EXP_LEEWAY) < time.generate()) {
                return Err(StatusListTokenVerificationError::Expired);
            }

            Ok(claims.status_list)
        }
    }
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use base64::Engine;
    use base64::prelude::BASE64_STANDARD;
    use serde_json::json;

    use crypto::EcdsaKey;
    use crypto::server_keys::KeyPair;
    use jwt::headers::HeaderWithTyp;
    use jwt::headers::HeaderWithX5c;

    use crate::status_list_token::StatusListClaims;
    use crate::status_list_token::StatusListToken;
    use crate::status_list_token::TOKEN_STATUS_LIST_JWT_TYP;

    pub async fn create_status_list_token<S>(
        keypair: &KeyPair<S>,
        exp: Option<i64>,
        ttl: Option<i64>,
    ) -> (HeaderWithX5c<HeaderWithTyp>, StatusListClaims, StatusListToken)
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

        let expected_claims: StatusListClaims = serde_json::from_value(example_payload).unwrap();

        let status_list_token =
            StatusListToken::builder(expected_claims.sub.clone(), expected_claims.status_list.clone())
                .exp(expected_claims.exp)
                .ttl(expected_claims.ttl)
                .sign(keypair)
                .await
                .unwrap();

        (expected_header, expected_claims, status_list_token)
    }
}

#[cfg(all(test, feature = "verification"))]
mod test {
    use std::ops::Add;

    use assert_matches::assert_matches;
    use chrono::Days;

    use crypto::server_keys::generate::Ca;
    use jwt::DEFAULT_VALIDATIONS;
    use jwt::error::JwtX5cError;
    use utils::generator::mock::MockTimeGenerator;

    use crate::status_list_token::mock::create_status_list_token;
    use crate::status_list_token::verification::StatusListTokenVerificationError;

    use super::*;

    const SLT_EXP: i64 = 2291720170;
    const SLT_TTL: i64 = 43200;

    #[tokio::test]
    async fn test_status_list_token() {
        let ca = Ca::generate("test", Default::default()).unwrap();
        let keypair = ca.generate_status_list_mock().unwrap();

        let (expected_header, expected_claims, signed) =
            create_status_list_token(&keypair, Some(SLT_EXP), Some(SLT_TTL)).await;

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

    #[tokio::test]
    async fn test_status_list_token_verification() {
        let ca = Ca::generate("test", Default::default()).unwrap();
        let keypair = ca.generate_status_list_mock().unwrap();
        let iss_keypair = ca.generate_issuer_mock().unwrap();

        let (_, expected_claims, signed) = create_status_list_token(&keypair, Some(SLT_EXP), Some(SLT_TTL)).await;

        let err = signed
            .parse_and_verify(
                &[],
                iss_keypair.certificate(),
                &expected_claims.sub,
                &MockTimeGenerator::default(),
            )
            .expect_err("should not verify for empty trust anchors");
        assert_matches!(
            err,
            StatusListTokenVerificationError::JwtVerification(JwtX5cError::CertificateValidation(_))
        );

        let err = signed
            .parse_and_verify(
                &[ca.to_trust_anchor()],
                ca.generate_pid_issuer_mock().unwrap().certificate(),
                &expected_claims.sub,
                &MockTimeGenerator::default(),
            )
            .expect_err("should not verify for attestation signing certificate with different DN");
        assert_matches!(err, StatusListTokenVerificationError::DifferentDN { .. });

        let err = signed
            .parse_and_verify(
                &[ca.to_trust_anchor()],
                iss_keypair.certificate(),
                &"http://example.com/sub".parse().unwrap(),
                &MockTimeGenerator::default(),
            )
            .expect_err("should not verify for attestation signing certificate with different sub claim");
        assert_matches!(err, StatusListTokenVerificationError::UnexpectedSubject { .. });

        let err = signed
            .parse_and_verify(
                &[ca.to_trust_anchor()],
                iss_keypair.certificate(),
                &expected_claims.sub,
                &MockTimeGenerator::new(DateTime::from_timestamp(SLT_EXP, 0).unwrap().add(Days::new(1))),
            )
            .expect_err("should not verify when jwt is expired");
        assert_matches!(err, StatusListTokenVerificationError::Expired);

        signed
            .parse_and_verify(
                &[ca.to_trust_anchor()],
                iss_keypair.certificate(),
                &expected_claims.sub,
                &MockTimeGenerator::default(),
            )
            .unwrap();
    }
}
