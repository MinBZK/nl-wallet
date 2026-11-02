use std::cmp::min;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use attestation_types::status_claim::StatusClaim;
use attestation_types::status_claim::StatusListClaim;
use chrono::DateTime;
use chrono::Utc;
use crypto::trust_anchor::TrustAnchors;
use moka::Expiry;
use moka::future::Cache;
use serde::Deserialize;
use serde::Serialize;
use tracing::warn;
use url::Url;
use utils::generator::Generator;

use crate::status_list::StatusType;
use crate::status_list_jwt::StatusListJwt;
use crate::status_list_jwt::verification::StatusListJwtVerificationError;
use crate::status_list_token::StatusListTokenClaims;
use crate::verification::client::StatusListClient;
use crate::verification::client::StatusListClientError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum RevocationStatus {
    Valid,
    Revoked,
    Undetermined,
    Corrupted,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum StatusListVerificationError {
    #[error("status list client error: {0}")]
    Networking(#[from] Arc<StatusListClientError>),

    #[error("status list token verification error: {0}")]
    Verification(#[from] Arc<StatusListJwtVerificationError>),
}

#[derive(Debug, Clone)]
struct CacheValidity {
    exp: Option<DateTime<Utc>>,
    ttl: Option<Duration>,
}

#[derive(Debug, Clone)]
struct CachedToken {
    token: StatusListJwt,
    fetched_at: Instant,
    // Only use verified claims to determine retention. A failed verification uses the shorter error TTL.
    cache_validity: Option<CacheValidity>,
}

type CachedResult = Result<CachedToken, Arc<StatusListClientError>>;

const ZERO_DURATION: Duration = Duration::from_secs(0);
const DEFAULT_CACHE_CAPACITY: u64 = 100;
const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(180);
const DEFAULT_CACHE_ERROR_TTL: Duration = Duration::from_secs(10);

struct TokenExpiry<G> {
    /// TTL when Status List Token has no `ttl` specified
    default_ttl: Duration,
    /// TTL when an error occurred, te prevent retrying always on an error
    error_ttl: Duration,
    /// Time generator for token expiration calculation
    time_generator: G,
}

#[derive(Debug)]
pub struct RevocationVerifier<C> {
    cache: Cache<Url, CachedResult>,
    client: Arc<C>,
}

impl<C> RevocationVerifier<C>
where
    C: StatusListClient,
{
    pub fn new_with_defaults<G>(client: Arc<C>, time_generator: G) -> Self
    where
        G: Generator<DateTime<Utc>> + Send + Sync + 'static,
    {
        Self::new(
            client,
            DEFAULT_CACHE_CAPACITY,
            DEFAULT_CACHE_TTL,
            DEFAULT_CACHE_ERROR_TTL,
            time_generator,
        )
    }

    pub fn new<G>(
        client: Arc<C>,
        cache_capacity: u64,
        default_ttl: Duration,
        error_ttl: Duration,
        time_generator: G,
    ) -> Self
    where
        G: Generator<DateTime<Utc>> + Send + Sync + 'static,
    {
        let cache = Cache::builder()
            .max_capacity(cache_capacity)
            .expire_after(TokenExpiry {
                default_ttl,
                error_ttl,
                time_generator,
            })
            .build();

        Self { cache, client }
    }

    #[cfg(any(test, feature = "mock"))]
    pub fn new_without_caching(client: Arc<C>) -> Self {
        let cache = Cache::builder().max_capacity(0).build();

        Self { cache, client }
    }

    pub async fn verify(
        &self,
        issuer_trust_anchors: &TrustAnchors,
        status_claim: StatusClaim,
        time: &impl Generator<DateTime<Utc>>,
    ) -> RevocationStatus {
        let StatusClaim::StatusList(StatusListClaim { uri, idx }) = status_claim;

        let result = self.fetch_status_list_claims(uri, issuer_trust_anchors, time).await;

        match result {
            Ok(claims) => {
                let Ok(index) = usize::try_from(idx) else {
                    warn!("status list index {idx} does not fit in usize");
                    return RevocationStatus::Corrupted;
                };
                match claims.status_list.get(index) {
                    Some(StatusType::Valid) => RevocationStatus::Valid,
                    Some(_) => RevocationStatus::Revoked,
                    None => {
                        warn!("status list index {idx} is out of bounds");
                        RevocationStatus::Corrupted
                    }
                }
            }
            Err(err) => match err {
                StatusListVerificationError::Networking(e) => {
                    warn!("Status list token fetching fails: {e}");
                    RevocationStatus::Undetermined
                }
                StatusListVerificationError::Verification(e) => {
                    warn!("Status list token fails verification: {e}");
                    RevocationStatus::Corrupted
                }
            },
        }
    }

    async fn fetch_status_list_claims(
        &self,
        url: Url,
        issuer_trust_anchors: &TrustAnchors,
        time: &impl Generator<DateTime<Utc>>,
    ) -> Result<StatusListTokenClaims, StatusListVerificationError> {
        loop {
            let mut verified_claims = None;
            let cached = self
                .cache
                .get_with(url.clone(), async {
                    let token = self.client.fetch(url.clone()).await.map_err(Arc::new)?;
                    let fetched_at = Instant::now();
                    let claims = token.parse_and_verify(issuer_trust_anchors, url.clone(), time);
                    let cache_validity = claims.as_ref().ok().map(|claims| CacheValidity {
                        exp: claims.exp,
                        ttl: claims.ttl,
                    });
                    verified_claims = Some(claims);

                    Ok(CachedToken {
                        token,
                        fetched_at,
                        cache_validity,
                    })
                })
                .await?;

            // The caller that fetched the token already verified it to determine its cache lifetime. Every other
            // caller verifies the cached token independently, using its own trust anchors and current time.
            if let Some(claims) = verified_claims {
                return claims.map_err(|error| StatusListVerificationError::Verification(Arc::new(error)));
            }
            let claims = cached
                .token
                .parse_and_verify(issuer_trust_anchors, url.clone(), time)
                .map_err(Arc::new)?;

            // An earlier caller may not have trusted this token, leaving its TTL unknown when it was cached.
            // Honor the now-verified limits before reusing it, including expiry within the verification leeway.
            if claims.ttl.is_some_and(|ttl| cached.fetched_at.elapsed() >= ttl)
                || claims.exp.is_some_and(|exp| exp <= time.generate())
            {
                self.cache.invalidate(&url).await;
                continue;
            }

            return Ok(claims);
        }
    }
}

impl<K, G> Expiry<K, CachedResult> for TokenExpiry<G>
where
    G: Generator<DateTime<Utc>>,
{
    fn expire_after_create(&self, _key: &K, value: &CachedResult, _created_at: Instant) -> Option<Duration> {
        let duration = match value.as_ref().ok().and_then(|cached| cached.cache_validity.as_ref()) {
            Some(validity) => {
                let ttl = validity.ttl.unwrap_or(self.default_ttl);
                match validity.exp {
                    None => ttl,
                    Some(exp) => min(
                        ttl,
                        // `.to_std` errors on negative duration
                        (exp - self.time_generator.generate()).to_std().unwrap_or(ZERO_DURATION),
                    ),
                }
            }
            None => self.error_ttl,
        };
        Some(duration)
    }
}

#[cfg(test)]
mod test {
    use std::ops::Add;
    use std::sync::Arc;
    use std::time::Duration;
    use std::time::Instant;

    use attestation_types::status_claim::StatusClaim::StatusList;
    use attestation_types::status_claim::StatusListClaim;
    use chrono::DateTime;
    use chrono::Days;
    use chrono::Utc;
    use crypto::server_keys::generate::Ca;
    use crypto::trust_anchor::TrustAnchors;
    use crypto::x509::CertificateConfiguration;
    use crypto::x509::DistinguishedName;
    use futures::FutureExt;
    use jwt::error::JwtParseError;
    use mockall::predicate::eq;
    use moka::Expiry;
    use rstest::rstest;
    use url::Url;
    use utils::generator::Generator;
    use utils::generator::mock::MockTimeGenerator;

    use crate::status_list_jwt::mock::create_status_list_jwt;
    use crate::verification::client::StatusListClientError;
    use crate::verification::client::mock::MockStatusListClient;
    use crate::verification::client::mock::StatusListClientStub;
    use crate::verification::verifier::CacheValidity;
    use crate::verification::verifier::CachedResult;
    use crate::verification::verifier::CachedToken;
    use crate::verification::verifier::RevocationStatus;
    use crate::verification::verifier::RevocationVerifier;
    use crate::verification::verifier::TokenExpiry;

    const TEN_MINUTES: Duration = Duration::from_secs(600);

    #[test]
    fn test_verify() {
        let ca = Ca::generate_mock();
        let keypair = ca.generate_issuer_status_list_mock().unwrap();

        let verifier = RevocationVerifier::new_without_caching(Arc::new(StatusListClientStub::new(keypair)));

        // Index 1 is valid
        let status = verifier
            .verify(
                &ca.to_trust_anchors(),
                StatusList(StatusListClaim {
                    uri: "https://example.com/statuslists/1".parse().unwrap(),
                    idx: 1,
                }),
                &MockTimeGenerator::default(),
            )
            .now_or_never()
            .unwrap();
        assert_eq!(RevocationStatus::Valid, status);

        // Index 3 is invalid
        let status = verifier
            .verify(
                &ca.to_trust_anchors(),
                StatusList(StatusListClaim {
                    uri: "https://example.com/statuslists/1".parse().unwrap(),
                    idx: 3,
                }),
                &MockTimeGenerator::default(),
            )
            .now_or_never()
            .unwrap();
        assert_eq!(RevocationStatus::Revoked, status);

        // An out-of-bounds index corrupts the reference instead of panicking
        let status = verifier
            .verify(
                &ca.to_trust_anchors(),
                StatusList(StatusListClaim {
                    uri: "https://example.com/statuslists/1".parse().unwrap(),
                    idx: u32::MAX,
                }),
                &MockTimeGenerator::default(),
            )
            .now_or_never()
            .unwrap();
        assert_eq!(RevocationStatus::Corrupted, status);

        // Corrupted when the sub claim doesn't match
        let status = verifier
            .verify(
                &ca.to_trust_anchors(),
                StatusList(StatusListClaim {
                    uri: "https://different_uri".parse().unwrap(),
                    idx: 1,
                }),
                &MockTimeGenerator::default(),
            )
            .now_or_never()
            .unwrap();
        assert_eq!(RevocationStatus::Corrupted, status);

        // Corrupted when the JWT doesn't validate
        let status = verifier
            .verify(
                &TrustAnchors::empty(),
                StatusList(StatusListClaim {
                    uri: "https://example.com/statuslists/1".parse().unwrap(),
                    idx: 1,
                }),
                &MockTimeGenerator::default(),
            )
            .now_or_never()
            .unwrap();
        assert_eq!(RevocationStatus::Corrupted, status);

        // Corrupted when the JWT is expired
        let status = verifier
            .verify(
                &ca.to_trust_anchors(),
                StatusList(StatusListClaim {
                    uri: "https://example.com/statuslists/1".parse().unwrap(),
                    idx: 1,
                }),
                &MockTimeGenerator::new(Utc::now().add(Days::new(2))),
            )
            .now_or_never()
            .unwrap();
        assert_eq!(RevocationStatus::Corrupted, status);

        // Undetermined when retrieving the status list fails
        let mut client = MockStatusListClient::new();
        client
            .expect_fetch()
            .returning(|_| Err(StatusListClientError::JwtParsing(JwtParseError::MissingKid)));
        let verifier = RevocationVerifier::new(
            Arc::new(client),
            10,
            TEN_MINUTES,
            TEN_MINUTES,
            MockTimeGenerator::default(),
        );
        let status = verifier
            .verify(
                &ca.to_trust_anchors(),
                StatusList(StatusListClaim {
                    uri: "https://example.com/statuslists/1".parse().unwrap(),
                    idx: 1,
                }),
                &MockTimeGenerator::default(),
            )
            .now_or_never()
            .unwrap();
        assert_eq!(RevocationStatus::Undetermined, status);
    }

    #[rstest]
    #[case::trusted_first(false)]
    #[case::untrusted_first(true)]
    fn test_verify_cached_token_with_each_callers_trust_anchors(#[case] untrusted_first: bool) {
        let ca = Ca::generate_mock();
        let other_ca = Ca::generate_mock();
        let keypair = ca.generate_issuer_status_list_mock().unwrap();
        let (_, _, token) = create_status_list_jwt(&keypair, None, None).now_or_never().unwrap();
        let mut cases = [
            (
                TrustAnchors::try_from(vec![ca.certificate().to_vec(), other_ca.certificate().to_vec()]).unwrap(),
                RevocationStatus::Valid,
            ),
            (
                TrustAnchors::try_from(vec![other_ca.certificate().to_vec(), ca.certificate().to_vec()]).unwrap(),
                RevocationStatus::Valid,
            ),
            (other_ca.to_trust_anchors(), RevocationStatus::Corrupted),
            (TrustAnchors::empty(), RevocationStatus::Corrupted),
        ];
        if untrusted_first {
            cases.reverse();
        }

        let uri: Url = "https://example.com/statuslists/1".parse().unwrap();
        let mut client = MockStatusListClient::new();
        client
            .expect_fetch()
            .with(eq(uri.clone()))
            .times(1)
            .returning(move |_| Ok(token.clone()));
        let verifier = RevocationVerifier::new(
            Arc::new(client),
            10,
            TEN_MINUTES,
            TEN_MINUTES,
            MockTimeGenerator::default(),
        );
        let time = MockTimeGenerator::default();
        let status_claim = StatusList(StatusListClaim { uri, idx: 1 });

        // Every trust-anchor set shares the fetched token, but no caller reuses a verification result.
        for (trust_anchors, expected) in cases.iter().chain(&cases) {
            let status = verifier
                .verify(trust_anchors, status_claim.clone(), &time)
                .now_or_never()
                .unwrap();
            assert_eq!(*expected, status);
        }
    }

    #[rstest]
    #[case::token_expiry(true)]
    #[case::signer_expiry(false)]
    fn test_verify_cached_token_with_each_callers_time(#[case] token_expiry: bool) {
        let now = Utc::now();
        let expires = now + TEN_MINUTES;
        let ca = Ca::generate_mock();
        let keypair = ca
            .generate_key_pair(
                DistinguishedName::create_mock("status list signer"),
                CertificateConfiguration {
                    not_after: (!token_expiry).then_some(expires),
                    ..Default::default()
                },
            )
            .unwrap();
        let (_, _, token) = create_status_list_jwt(&keypair, token_expiry.then_some(expires.timestamp()), None)
            .now_or_never()
            .unwrap();
        let uri: Url = "https://example.com/statuslists/1".parse().unwrap();
        let mut client = MockStatusListClient::new();
        client
            .expect_fetch()
            .with(eq(uri.clone()))
            .times(1)
            .returning(move |_| Ok(token.clone()));
        let verifier = RevocationVerifier::new(
            Arc::new(client),
            10,
            TEN_MINUTES,
            TEN_MINUTES,
            MockTimeGenerator::new(now),
        );
        let status_claim = StatusList(StatusListClaim { uri, idx: 1 });

        // Only the caller's verification time changes; the cache's monotonic TTL has not elapsed.
        for (time, expected) in [
            (now, RevocationStatus::Valid),
            (expires + Duration::from_secs(61), RevocationStatus::Corrupted),
            (now, RevocationStatus::Valid),
        ] {
            let status = verifier
                .verify(
                    &ca.to_trust_anchors(),
                    status_claim.clone(),
                    &MockTimeGenerator::new(time),
                )
                .now_or_never()
                .unwrap();
            assert_eq!(expected, status);
        }
    }

    #[test]
    fn test_cache_fetch_errors_across_trust_anchors() {
        let uri: Url = "https://example.com/statuslists/1".parse().unwrap();
        let mut client = MockStatusListClient::new();
        client
            .expect_fetch()
            .with(eq(uri.clone()))
            .times(1)
            .returning(|_| Err(StatusListClientError::JwtParsing(JwtParseError::MissingKid)));
        let time = MockTimeGenerator::default();
        let verifier = RevocationVerifier::new(Arc::new(client), 10, TEN_MINUTES, TEN_MINUTES, time.clone());
        let status_claim = StatusList(StatusListClaim { uri, idx: 1 });

        for trust_anchors in [TrustAnchors::empty(), Ca::generate_mock().to_trust_anchors()] {
            let status = verifier
                .verify(&trust_anchors, status_claim.clone(), &time)
                .now_or_never()
                .unwrap();
            assert_eq!(RevocationStatus::Undetermined, status);
        }
    }

    #[test]
    fn test_honor_token_ttl_after_untrusted_fetch() {
        let ca = Ca::generate_mock();
        let keypair = ca.generate_issuer_status_list_mock().unwrap();
        let (_, _, token) = create_status_list_jwt(&keypair, None, Some(0)).now_or_never().unwrap();
        let uri: Url = "https://example.com/statuslists/1".parse().unwrap();
        let mut client = MockStatusListClient::new();
        client
            .expect_fetch()
            .with(eq(uri.clone()))
            .times(3)
            .returning(move |_| Ok(token.clone()));
        let time = MockTimeGenerator::default();
        let verifier = RevocationVerifier::new(Arc::new(client), 10, TEN_MINUTES, TEN_MINUTES, time.clone());
        let status_claim = StatusList(StatusListClaim { uri, idx: 1 });

        for (trust_anchors, expected) in [
            (TrustAnchors::empty(), RevocationStatus::Corrupted),
            (ca.to_trust_anchors(), RevocationStatus::Valid),
            (ca.to_trust_anchors(), RevocationStatus::Valid),
        ] {
            let status = verifier
                .verify(&trust_anchors, status_claim.clone(), &time)
                .now_or_never()
                .unwrap();
            assert_eq!(expected, status);
        }
    }

    fn cached_result(exp: Option<DateTime<Utc>>, ttl: Option<Duration>) -> (Url, CachedResult) {
        let url: Url = "http://localhost".parse().unwrap();
        let cached_result = Ok(CachedToken {
            token: "header.payload.signature".parse().unwrap(),
            fetched_at: Instant::now(),
            cache_validity: Some(CacheValidity { exp, ttl }),
        });
        (url, cached_result)
    }

    #[test]
    fn should_cache_for_default_ttl() {
        let expiry = TokenExpiry {
            default_ttl: Duration::from_secs(2),
            error_ttl: Duration::from_secs(1),
            time_generator: MockTimeGenerator::default(),
        };
        let (url, cached_result) = cached_result(None, None);
        let duration = expiry.expire_after_create(&url, &cached_result, Instant::now());
        assert_eq!(Some(Duration::from_secs(2)), duration);
    }

    #[test]
    fn should_cache_for_explicit_ttl() {
        let expiry = TokenExpiry {
            default_ttl: Duration::from_secs(2),
            error_ttl: Duration::from_secs(1),
            time_generator: MockTimeGenerator::default(),
        };
        let (url, cached_result) = cached_result(None, Some(Duration::from_secs(3)));
        let duration = expiry.expire_after_create(&url, &cached_result, Instant::now());
        assert_eq!(Some(Duration::from_secs(3)), duration);
    }

    #[test]
    fn should_cache_on_exp_if_lower_than_ttl() {
        let time = MockTimeGenerator::default();
        let expiry = TokenExpiry {
            default_ttl: Duration::from_secs(2),
            error_ttl: Duration::from_secs(1),
            time_generator: time.clone(),
        };
        let (url, cached_result) = cached_result(
            Some(time.generate() + Duration::from_secs(3)),
            Some(Duration::from_secs(4)),
        );
        let duration = expiry.expire_after_create(&url, &cached_result, Instant::now());
        assert_eq!(Some(Duration::from_secs(3)), duration);
    }

    #[test]
    fn should_cache_on_exp_if_lower_than_default_ttl() {
        let time = MockTimeGenerator::default();
        let expiry = TokenExpiry {
            default_ttl: Duration::from_secs(3),
            error_ttl: Duration::from_secs(1),
            time_generator: time.clone(),
        };
        let (url, cached_result) = cached_result(Some(time.generate() + Duration::from_secs(2)), None);
        let duration = expiry.expire_after_create(&url, &cached_result, Instant::now());
        assert_eq!(Some(Duration::from_secs(2)), duration);
    }

    #[rstest]
    #[case::fetch_error(Err(Arc::new(StatusListClientError::JwtParsing(JwtParseError::MissingKid))))]
    #[case::verification_error(Ok(CachedToken {
        token: "header.payload.signature".parse().unwrap(),
        fetched_at: Instant::now(),
        cache_validity: None,
    }))]
    fn should_cache_error_ttl_on_err(#[case] cached_result: CachedResult) {
        let time = MockTimeGenerator::default();
        let expiry = TokenExpiry {
            default_ttl: Duration::from_secs(3),
            error_ttl: Duration::from_secs(1),
            time_generator: time.clone(),
        };
        let duration = expiry.expire_after_create(
            &"http://localhost".parse::<Url>().unwrap(),
            &cached_result,
            Instant::now(),
        );
        assert_eq!(Some(Duration::from_secs(1)), duration);
    }
}
