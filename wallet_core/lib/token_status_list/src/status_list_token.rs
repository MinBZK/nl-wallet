use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
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
    use url::Url;
    use utils::generator::Generator;

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
}
