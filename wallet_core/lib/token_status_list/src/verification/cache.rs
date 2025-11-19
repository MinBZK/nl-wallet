use std::cmp::min;
use std::time::Duration;
use std::time::Instant;

use chrono::Utc;
use moka::Expiry;
use moka::future::Cache;
use url::Url;

use crate::status_list_token::StatusListToken;
use crate::verification::client::StatusListClient;
use crate::verification::client::StatusListClientError;

type CachedResult = Result<StatusListToken, StatusListClientError>;

#[derive(Debug, Clone)]
pub struct CachedStatusListClient<C> {
    cache: Cache<Url, CachedResult>,
    client: C,
}

struct CachedExpiry {
    /// TTL when Status List Token has no `ttl` specified
    default_ttl: Duration,
    /// TTL when an error occurred, te prevent retrying always on an error
    error_ttl: Duration,
}

const ZERO_DURATION: Duration = Duration::from_secs(0);

impl Expiry<Url, CachedResult> for CachedExpiry {
    fn expire_after_create(&self, _key: &Url, value: &CachedResult, _created_at: Instant) -> Option<Duration> {
        let duration = match value.as_ref() {
            Ok(token) => token
                .as_ref()
                // TODO: PVW-5222 Only cache with verified `ttl` and `exp`
                .dangerous_parse_unverified()
                .map(|(_, claims)| {
                    let ttl = claims.ttl.unwrap_or(self.default_ttl);
                    match claims.exp {
                        None => ttl,
                        Some(exp) => min(
                            ttl,
                            // `.to_std` errors on negative duration
                            (exp - Utc::now()).to_std().unwrap_or(ZERO_DURATION),
                        ),
                    }
                })
                .unwrap_or(self.error_ttl),
            Err(_) => self.error_ttl,
        };
        Some(duration)
    }
}

impl<C: StatusListClient> StatusListClient for CachedStatusListClient<C> {
    async fn fetch(&self, url: Url) -> Result<StatusListToken, StatusListClientError> {
        self.cache.get_with(url.clone(), self.client.fetch(url)).await
    }
}

impl<C> CachedStatusListClient<C> {
    pub fn new(client: C, capacity: u64, default_ttl: Duration, error_ttl: Duration) -> Self {
        let cache = Cache::builder()
            .max_capacity(capacity)
            .expire_after(CachedExpiry { default_ttl, error_ttl })
            .build();
        Self { cache, client }
    }
}

#[cfg(test)]
mod test {
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;

    use futures::future::join_all;
    use futures::future::try_join_all;

    use crypto::server_keys::generate::Ca;
    use jwt::error::JwtError;
    use jwt::headers::HeaderWithTyp;
    use jwt::headers::HeaderWithX5c;

    use crate::status_list_token::StatusListClaims;
    use crate::status_list_token::mock::create_status_list_token;
    use crate::verification::client::MockStatusListClient;

    use super::*;

    const TEN_MINUTES: Duration = Duration::from_secs(600);

    async fn setup_mock_tokens<I, F>(tokens: I) -> MockStatusListClient
    where
        F: Future<Output = (HeaderWithX5c<HeaderWithTyp>, StatusListClaims, StatusListToken)>,
        I: IntoIterator<Item = F>,
    {
        setup_mock_results(
            join_all(tokens.into_iter())
                .await
                .into_iter()
                .map(|result| Ok(result.2)),
        )
    }

    fn setup_mock_results(results: impl IntoIterator<Item = CachedResult>) -> MockStatusListClient {
        let results = results.into_iter().collect::<Vec<_>>();
        let mut client = MockStatusListClient::new();
        let index = AtomicUsize::default();
        client
            .expect_fetch()
            .times(results.len())
            .returning(move |_| results[index.fetch_add(1, Ordering::Relaxed)].clone());
        client
    }

    #[tokio::test]
    async fn should_use_cache_for_same_url() {
        let ca = Ca::generate("test", Default::default()).unwrap();
        let keypair = ca.generate_status_list_mock().unwrap();

        let client = setup_mock_tokens([create_status_list_token(&keypair, None, None)]).await;
        let cached = CachedStatusListClient::new(client, 10, TEN_MINUTES, TEN_MINUTES);

        let res1 = cached.fetch("https://localhost:8080".parse().unwrap()).await.unwrap();
        let res2 = cached.fetch("https://localhost:8080".parse().unwrap()).await.unwrap();

        assert_eq!(res1, res2);
    }

    #[tokio::test]
    async fn should_not_use_cache_for_different_url() {
        let ca = Ca::generate("test", Default::default()).unwrap();
        let keypair = ca.generate_status_list_mock().unwrap();

        let client = setup_mock_tokens([
            create_status_list_token(&keypair, None, Some(10)),
            create_status_list_token(&keypair, None, Some(20)),
        ])
        .await;
        let cached = CachedStatusListClient::new(client, 10, TEN_MINUTES, TEN_MINUTES);

        let res1 = cached.fetch("https://localhost:8080".parse().unwrap()).await.unwrap();
        let res2 = cached.fetch("https://localhost:8008".parse().unwrap()).await.unwrap();

        assert_ne!(res1, res2);
    }

    #[tokio::test]
    async fn should_cache_with_default_ttl() {
        let ca = Ca::generate("test", Default::default()).unwrap();
        let keypair = ca.generate_status_list_mock().unwrap();

        let now = Utc::now().timestamp();
        let client = setup_mock_tokens([
            create_status_list_token(&keypair, Some(now + 100), None),
            create_status_list_token(&keypair, Some(now + 200), None),
        ])
        .await;
        let cached = CachedStatusListClient::new(client, 10, Duration::from_millis(100), TEN_MINUTES);

        let res1a = cached.fetch("https://localhost:8080".parse().unwrap()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(10)).await;
        let res1b = cached.fetch("https://localhost:8080".parse().unwrap()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        let res2 = cached.fetch("https://localhost:8080".parse().unwrap()).await.unwrap();

        assert_eq!(res1a, res1b);
        assert_ne!(res1a, res2);
    }

    #[tokio::test]
    async fn should_cache_with_explicit_ttl() {
        let ca = Ca::generate("test", Default::default()).unwrap();
        let keypair = ca.generate_status_list_mock().unwrap();

        let client = setup_mock_tokens([
            create_status_list_token(&keypair, None, Some(1)),
            create_status_list_token(&keypair, None, Some(2)),
        ])
        .await;
        let cached = CachedStatusListClient::new(client, 10, TEN_MINUTES, TEN_MINUTES);

        let res1a = cached.fetch("https://localhost:8080".parse().unwrap()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        let res1b = cached.fetch("https://localhost:8080".parse().unwrap()).await.unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;
        let res2 = cached.fetch("https://localhost:8080".parse().unwrap()).await.unwrap();

        assert_eq!(res1a, res1b);
        assert_ne!(res1a, res2);
    }

    #[tokio::test]
    async fn should_cache_with_explicit_ttl_preferred() {
        let ca = Ca::generate("test", Default::default()).unwrap();
        let keypair = ca.generate_status_list_mock().unwrap();

        let client = setup_mock_tokens([
            create_status_list_token(&keypair, None, Some(1)),
            create_status_list_token(&keypair, None, Some(2)),
        ])
        .await;
        let cached = CachedStatusListClient::new(client, 10, Duration::from_millis(100), TEN_MINUTES);

        let res1a = cached.fetch("https://localhost:8080".parse().unwrap()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(200)).await;
        let res1b = cached.fetch("https://localhost:8080".parse().unwrap()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(900)).await;
        let res2 = cached.fetch("https://localhost:8080".parse().unwrap()).await.unwrap();

        assert_eq!(res1a, res1b);
        assert_ne!(res1a, res2);
    }

    #[tokio::test]
    async fn should_cache_on_exp_if_lower_than_ttl() {
        let ca = Ca::generate("test", Default::default()).unwrap();
        let keypair = ca.generate_status_list_mock().unwrap();

        let now = Utc::now().timestamp();
        let client = setup_mock_tokens([
            create_status_list_token(&keypair, Some(now), Some(100)),
            create_status_list_token(&keypair, Some(now + 100), None),
        ])
        .await;
        let cached = CachedStatusListClient::new(client, 10, TEN_MINUTES, TEN_MINUTES);

        let res1 = cached.fetch("https://localhost:8080".parse().unwrap()).await.unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;
        let res2 = cached.fetch("https://localhost:8080".parse().unwrap()).await.unwrap();

        assert_ne!(res1, res2);
    }

    #[tokio::test]
    async fn should_cache_on_exp_if_lower_than_default_ttl() {
        let ca = Ca::generate("test", Default::default()).unwrap();
        let keypair = ca.generate_status_list_mock().unwrap();

        let now = Utc::now().timestamp();
        let client = setup_mock_tokens([
            create_status_list_token(&keypair, Some(now), None),
            create_status_list_token(&keypair, Some(now + 100), None),
        ])
        .await;
        let cached = CachedStatusListClient::new(client, 10, TEN_MINUTES, TEN_MINUTES);

        let res1 = cached.fetch("https://localhost:8080".parse().unwrap()).await.unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;
        let res2 = cached.fetch("https://localhost:8080".parse().unwrap()).await.unwrap();

        assert_ne!(res1, res2);
    }

    #[tokio::test]
    async fn should_cache_error_ttl_on_err() {
        let ca = Ca::generate("test", Default::default()).unwrap();
        let keypair = ca.generate_status_list_mock().unwrap();

        let client = setup_mock_results([
            Err(StatusListClientError::JwtParsing(JwtError::MissingX5c.into())),
            Ok(create_status_list_token(&keypair, None, None).await.2),
        ]);
        let cached = CachedStatusListClient::new(client, 10, TEN_MINUTES, Duration::from_millis(100));

        let res1a = cached.fetch("https://localhost:8080".parse().unwrap()).await;
        tokio::time::sleep(Duration::from_millis(10)).await;
        let res1b = cached.fetch("https://localhost:8080".parse().unwrap()).await;
        tokio::time::sleep(Duration::from_millis(200)).await;
        let res2 = cached.fetch("https://localhost:8080".parse().unwrap()).await;

        assert!(res1a.is_err());
        assert!(res1b.is_err());
        assert!(res2.is_ok());
    }

    struct SlowStatusListClient(CachedResult, Duration);

    impl StatusListClient for SlowStatusListClient {
        async fn fetch(&self, _url: Url) -> Result<StatusListToken, StatusListClientError> {
            tokio::time::sleep(self.1).await;
            self.0.clone()
        }
    }

    #[tokio::test]
    async fn should_coalesce_results() {
        let ca = Ca::generate("test", Default::default()).unwrap();
        let keypair = ca.generate_status_list_mock().unwrap();

        let expected = create_status_list_token(&keypair, None, None).await.2;
        let client = SlowStatusListClient(Ok(expected.clone()), Duration::from_millis(100));
        let cached = CachedStatusListClient::new(client, 10, TEN_MINUTES, TEN_MINUTES);

        let results = try_join_all((0..3).map(|_| cached.fetch("https://localhost:8080".parse().unwrap())))
            .await
            .unwrap();
        assert_eq!(results.len(), 3);
        for returned in results {
            assert_eq!(returned, expected)
        }
    }
}
