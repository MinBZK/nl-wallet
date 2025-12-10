use std::cmp::min;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use chrono::DateTime;
use chrono::Utc;
use moka::Expiry;
use moka::future::Cache;
use rustls_pki_types::TrustAnchor;
use serde::Deserialize;
use serde::Serialize;
use tracing::warn;
use url::Url;

use attestation_types::status_claim::StatusClaim;
use attestation_types::status_claim::StatusClaim::StatusList;
use attestation_types::status_claim::StatusListClaim;
use crypto::x509::DistinguishedName;
use utils::generator::Generator;

use crate::status_list::StatusType;
use crate::status_list_token::StatusListClaims;
use crate::status_list_token::verification::StatusListTokenVerificationError;
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
    Verification(#[from] Arc<StatusListTokenVerificationError>),
}

type CachedResult = Result<StatusListClaims, StatusListVerificationError>;

const ZERO_DURATION: Duration = Duration::from_secs(0);

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
        issuer_trust_anchors: &[TrustAnchor<'_>],
        attestation_signing_certificate_dn: DistinguishedName,
        status_claim: StatusClaim,
        time: &impl Generator<DateTime<Utc>>,
    ) -> RevocationStatus {
        let StatusList(StatusListClaim { uri, idx }) = status_claim;

        let result = self
            .cache
            .get_with(
                uri.clone(),
                self.fetch_status_list_claims(uri, issuer_trust_anchors, attestation_signing_certificate_dn, time),
            )
            .await;

        match result {
            Ok(claims) => match claims.status_list.single_unpack(idx.try_into().unwrap()) {
                StatusType::Valid => RevocationStatus::Valid,
                _ => RevocationStatus::Revoked,
            },
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
        issuer_trust_anchors: &[TrustAnchor<'_>],
        attestation_signing_certificate_dn: DistinguishedName,
        time: &impl Generator<DateTime<Utc>>,
    ) -> Result<StatusListClaims, StatusListVerificationError> {
        let status_list_token = self.client.fetch(url.clone()).await.map_err(Arc::new)?;

        let claims = status_list_token
            .parse_and_verify(issuer_trust_anchors, attestation_signing_certificate_dn, &url, time)
            .map_err(Arc::new)?;

        Ok(claims)
    }
}

impl<G> Expiry<Url, CachedResult> for TokenExpiry<G>
where
    G: Generator<DateTime<Utc>>,
{
    fn expire_after_create(&self, _key: &Url, value: &CachedResult, _created_at: Instant) -> Option<Duration> {
        let duration = match value.as_ref() {
            Ok(claims) => {
                let ttl = claims.ttl.unwrap_or(self.default_ttl);
                match claims.exp {
                    None => ttl,
                    Some(exp) => min(
                        ttl,
                        // `.to_std` errors on negative duration
                        (exp - self.time_generator.generate()).to_std().unwrap_or(ZERO_DURATION),
                    ),
                }
            }
            Err(_) => self.error_ttl,
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

    use chrono::DateTime;
    use chrono::Days;
    use chrono::Utc;
    use futures::FutureExt;
    use moka::Expiry;
    use url::Url;

    use attestation_types::status_claim::StatusClaim::StatusList;
    use attestation_types::status_claim::StatusListClaim;
    use crypto::server_keys::generate::Ca;
    use crypto::x509::DistinguishedName;
    use jwt::error::JwtError;
    use utils::generator::Generator;
    use utils::generator::mock::MockTimeGenerator;

    use crate::status_list::PackedStatusList;
    use crate::status_list_token::StatusListClaims;
    use crate::status_list_token::verification::StatusListTokenVerificationError;
    use crate::verification::client::StatusListClientError;
    use crate::verification::client::mock::MockStatusListClient;
    use crate::verification::client::mock::StatusListClientStub;
    use crate::verification::verifier::RevocationStatus;
    use crate::verification::verifier::RevocationVerifier;
    use crate::verification::verifier::StatusListVerificationError;
    use crate::verification::verifier::TokenExpiry;

    const TEN_MINUTES: Duration = Duration::from_secs(600);

    #[test]
    fn test_verify() {
        let ca = Ca::generate("test", Default::default()).unwrap();
        let keypair = ca.generate_status_list_mock().unwrap();
        let iss_keypair = ca.generate_issuer_mock().unwrap();

        let verifier = RevocationVerifier::new_without_caching(Arc::new(StatusListClientStub::new(keypair)));

        // Index 1 is valid
        let status = verifier
            .verify(
                &[ca.to_trust_anchor()],
                iss_keypair.certificate().distinguished_name_canonical().unwrap(),
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
                &[ca.to_trust_anchor()],
                iss_keypair.certificate().distinguished_name_canonical().unwrap(),
                StatusList(StatusListClaim {
                    uri: "https://example.com/statuslists/1".parse().unwrap(),
                    idx: 3,
                }),
                &MockTimeGenerator::default(),
            )
            .now_or_never()
            .unwrap();
        assert_eq!(RevocationStatus::Revoked, status);

        // Corrupted when the sub claim doesn't match
        let status = verifier
            .verify(
                &[ca.to_trust_anchor()],
                iss_keypair.certificate().distinguished_name_canonical().unwrap(),
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
                &[],
                iss_keypair.certificate().distinguished_name_canonical().unwrap(),
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
                &[ca.to_trust_anchor()],
                iss_keypair.certificate().distinguished_name_canonical().unwrap(),
                StatusList(StatusListClaim {
                    uri: "https://example.com/statuslists/1".parse().unwrap(),
                    idx: 1,
                }),
                &MockTimeGenerator::new(Utc::now().add(Days::new(2))),
            )
            .now_or_never()
            .unwrap();
        assert_eq!(RevocationStatus::Corrupted, status);

        // Corrupted when the attestation is signed with a different certificate
        let status = verifier
            .verify(
                &[ca.to_trust_anchor()],
                DistinguishedName::new(String::from("CN=Different CA")),
                StatusList(StatusListClaim {
                    uri: "https://example.com/statuslists/1".parse().unwrap(),
                    idx: 1,
                }),
                &MockTimeGenerator::default(),
            )
            .now_or_never()
            .unwrap();
        assert_eq!(RevocationStatus::Corrupted, status);

        // Undetermined when retrieving the status list fails
        let mut client = MockStatusListClient::new();
        client
            .expect_fetch()
            .returning(|_| Err(StatusListClientError::JwtParsing(JwtError::MissingX5c)));
        let verifier = RevocationVerifier::new(
            Arc::new(client),
            10,
            TEN_MINUTES,
            TEN_MINUTES,
            MockTimeGenerator::default(),
        );
        let status = verifier
            .verify(
                &[ca.to_trust_anchor()],
                iss_keypair.certificate().distinguished_name_canonical().unwrap(),
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

    #[test]
    fn test_verify_cached() {
        let ca = Ca::generate("test", Default::default()).unwrap();
        let keypair = ca.generate_status_list_mock().unwrap();
        let iss_keypair = ca.generate_issuer_mock().unwrap();

        let verifier = RevocationVerifier::new(
            Arc::new(StatusListClientStub::new(keypair)),
            10,
            TEN_MINUTES,
            TEN_MINUTES,
            MockTimeGenerator::default(),
        );

        // Index 1 is valid
        let status = verifier
            .verify(
                &[ca.to_trust_anchor()],
                iss_keypair.certificate().distinguished_name_canonical().unwrap(),
                StatusList(StatusListClaim {
                    uri: "https://example.com/statuslists/1".parse().unwrap(),
                    idx: 1,
                }),
                &MockTimeGenerator::default(),
            )
            .now_or_never()
            .unwrap();
        assert_eq!(RevocationStatus::Valid, status);

        // Empty trust anchors would normally fail verification, but we're using a cached result here
        let status = verifier
            .verify(
                &[],
                iss_keypair.certificate().distinguished_name_canonical().unwrap(),
                StatusList(StatusListClaim {
                    uri: "https://example.com/statuslists/1".parse().unwrap(),
                    idx: 1,
                }),
                &MockTimeGenerator::default(),
            )
            .now_or_never()
            .unwrap();
        assert_eq!(RevocationStatus::Valid, status);
    }

    fn cached_result(
        exp: Option<DateTime<Utc>>,
        ttl: Option<Duration>,
    ) -> (Url, Result<StatusListClaims, StatusListVerificationError>) {
        let url: Url = "http://localhost".parse().unwrap();
        let cached_result = Ok(StatusListClaims {
            iat: Utc::now(),
            exp,
            sub: url.clone(),
            ttl,
            status_list: PackedStatusList::default(),
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

    #[test]
    fn should_cache_error_ttl_on_err() {
        let time = MockTimeGenerator::default();
        let expiry = TokenExpiry {
            default_ttl: Duration::from_secs(3),
            error_ttl: Duration::from_secs(1),
            time_generator: time.clone(),
        };
        let duration = expiry.expire_after_create(
            &"http://localhost".parse().unwrap(),
            &Err(StatusListVerificationError::Verification(Arc::new(
                StatusListTokenVerificationError::Expired,
            ))),
            Instant::now(),
        );
        assert_eq!(Some(Duration::from_secs(1)), duration);
    }
}
