use std::str;
use std::str::Utf8Error;
use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use jwt::error::JwtParseError;
use url::Url;
use utils::generator::Generator;

use crate::status_list::PackedStatusList;
use crate::status_list_cwt::StatusListCwt;
use crate::status_list_jwt::StatusListJwt;

/// A Status List Token in either JWT or CWT format.
///
/// <https://www.ietf.org/archive/id/draft-ietf-oauth-status-list-21.html#section-5>
#[derive(Debug, Clone)]
pub enum StatusListToken {
    Jwt(StatusListJwt),
    Cwt(Box<StatusListCwt>),
}

impl StatusListToken {
    /// Parse a Status List Token in the given format, without verifying it.
    pub fn from_bytes_as(format: StatusListFormat, bytes: &[u8]) -> Result<Self, StatusListTokenParseError> {
        let token = match format {
            StatusListFormat::Jwt => str::from_utf8(bytes)
                .map_err(StatusListTokenParseError::JwtEncoding)?
                .parse::<StatusListJwt>()
                .map_err(StatusListTokenParseError::Jwt)?
                .into(),
            StatusListFormat::Cwt => StatusListCwt::from_slice(bytes)
                .map_err(StatusListTokenParseError::Cwt)?
                .into(),
        };

        Ok(token)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StatusListTokenParseError {
    #[error("status list token JWT is not valid UTF-8: {0}")]
    JwtEncoding(#[source] Utf8Error),

    #[error("could not parse status list token JWT: {0}")]
    Jwt(#[source] JwtParseError),

    #[error("could not parse status list token CWT: {0}")]
    Cwt(#[source] ciborium::de::Error<std::io::Error>),
}

impl From<StatusListJwt> for StatusListToken {
    fn from(jwt: StatusListJwt) -> Self {
        Self::Jwt(jwt)
    }
}

impl From<StatusListCwt> for StatusListToken {
    fn from(cwt: StatusListCwt) -> Self {
        Self::Cwt(Box::new(cwt))
    }
}

/// The formats of a Status List Token, identified by their media type.
///
/// <https://www.ietf.org/archive/id/draft-ietf-oauth-status-list-21.html#section-8.2>
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumString, strum::IntoStaticStr)]
#[strum(ascii_case_insensitive)]
pub enum StatusListFormat {
    #[strum(serialize = "application/statuslist+jwt")]
    Jwt,
    #[strum(serialize = "application/statuslist+cwt")]
    Cwt,
}

impl StatusListFormat {
    /// The media type of a Status List Token in this format, as used in the `Accept` and `Content-Type` headers.
    pub fn media_type(self) -> &'static str {
        self.into()
    }
}

impl StatusListToken {
    /// Build the claims of a Status List Token, which can then be signed in either format using
    /// [`StatusListJwt::sign`] or [`StatusListCwt::sign`].
    pub fn builder(sub: Url, status_list: PackedStatusList) -> StatusListTokenBuilder {
        StatusListTokenBuilder {
            exp: None,
            sub,
            ttl: None,
            status_list,
        }
    }
}

/// The claims of a Status List Token, independent of the format of the token.
///
/// <https://www.ietf.org/archive/id/draft-ietf-oauth-status-list-21.html#section-5>
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct StatusListTokenClaims {
    pub iat: DateTime<Utc>,

    pub exp: Option<DateTime<Utc>>,

    /// The sub (subject) claim MUST specify the URI of the Status List Token. The value MUST be equal to that of the
    /// `uri` claim contained in the `status_list` claim of the Referenced Token
    pub sub: Url,

    /// If present, MUST specify the maximum amount of time, in seconds, that the Status List Token can be cached by a
    /// consumer before a fresh copy SHOULD be retrieved.
    pub ttl: Option<Duration>,

    pub status_list: PackedStatusList,
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

    /// Build the claims, setting the `iat` claim to the current time.
    pub fn build(self, time: &impl Generator<DateTime<Utc>>) -> StatusListTokenClaims {
        StatusListTokenClaims {
            iat: time.generate(),
            exp: self.exp,
            sub: self.sub,
            ttl: self.ttl,
            status_list: self.status_list,
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
    use url::Url;
    use utils::generator::Generator;

    use crate::status_list_cwt::verification::StatusListCwtVerificationError;
    use crate::status_list_jwt::verification::StatusListJwtVerificationError;
    use crate::status_list_token::StatusListToken;
    use crate::status_list_token::StatusListTokenClaims;

    const EXP_LEEWAY: Duration = Duration::seconds(60);

    #[derive(Debug, thiserror::Error)]
    pub enum StatusListTokenClaimsVerificationError {
        #[error("status list token is expired")]
        Expired,

        #[error("status list token subject claim ('{sub}') does not match url claim of Reference Token ('{url}')")]
        UnexpectedSubject { sub: Box<Url>, url: Box<Url> },
    }

    impl StatusListTokenClaims {
        /// Check the claims of a verified Status List Token in either format, as required by step 4 of
        /// <https://www.ietf.org/archive/id/draft-ietf-oauth-status-list-21.html#section-8.3>.
        pub fn verify(
            &self,
            url: &Url,
            time: &impl Generator<DateTime<Utc>>,
        ) -> Result<(), StatusListTokenClaimsVerificationError> {
            if *url != self.sub {
                return Err(StatusListTokenClaimsVerificationError::UnexpectedSubject {
                    sub: Box::new(self.sub.clone()),
                    url: Box::new(url.to_owned()),
                });
            }

            if self.exp.is_some_and(|exp| exp.add(EXP_LEEWAY) < time.generate()) {
                return Err(StatusListTokenClaimsVerificationError::Expired);
            }

            Ok(())
        }
    }

    #[derive(Debug, thiserror::Error)]
    pub enum StatusListTokenVerificationError {
        #[error(transparent)]
        Jwt(StatusListJwtVerificationError),

        #[error(transparent)]
        Cwt(StatusListCwtVerificationError),
    }

    impl StatusListToken {
        pub fn parse_and_verify(
            &self,
            trust_anchors: &TrustAnchors,
            url: &Url,
            time: &impl Generator<DateTime<Utc>>,
        ) -> Result<StatusListTokenClaims, StatusListTokenVerificationError> {
            match self {
                Self::Jwt(jwt) => jwt
                    .parse_and_verify(trust_anchors, url, time)
                    .map_err(StatusListTokenVerificationError::Jwt),
                Self::Cwt(cwt) => cwt
                    .parse_and_verify(trust_anchors, url, time)
                    .map_err(StatusListTokenVerificationError::Cwt),
            }
        }
    }
}

#[cfg(all(test, feature = "verification"))]
mod verification_test {
    use crypto::server_keys::generate::Ca;
    use rstest::rstest;
    use utils::generator::mock::MockTimeGenerator;

    use crate::status_list_cwt::mock::mock_status_list_cwt;
    use crate::status_list_jwt::mock::mock_status_list_jwt;
    use crate::status_list_token::StatusListFormat;
    use crate::status_list_token::StatusListToken;

    #[rstest]
    #[case::jwt(StatusListFormat::Jwt)]
    #[case::cwt(StatusListFormat::Cwt)]
    #[tokio::test]
    async fn test_status_list_token_parse_and_verify(#[case] format: StatusListFormat) {
        let ca = Ca::generate_mock();
        let keypair = ca.generate_issuer_status_list_mock().unwrap();
        let (sub, token): (_, StatusListToken) = match format {
            StatusListFormat::Jwt => {
                let (_, claims, jwt) = mock_status_list_jwt(&keypair, None, None).await;
                (claims.sub, jwt.into())
            }
            StatusListFormat::Cwt => {
                let (cwt, claims) = mock_status_list_cwt(&keypair, None).await;
                (claims.sub, cwt.into())
            }
        };

        let claims = token
            .parse_and_verify(&ca.to_trust_anchors(), &sub, &MockTimeGenerator::default())
            .unwrap();

        assert_eq!(claims.sub, sub);
    }
}
