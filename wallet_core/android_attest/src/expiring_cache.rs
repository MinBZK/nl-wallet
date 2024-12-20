use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use tokio::sync::RwLock;

/// This trait marks data that can expire, for example by expiration time/date.
pub trait Expiring {
    /// Returns true when the data is expired at the moment of calling.
    async fn is_expired(&self) -> bool;
}

/// Provider of [`T`], raises a [`Self::Error`] on failure to provide.
pub trait Provider<T> {
    type Error;

    async fn provide(&self) -> Result<T, Self::Error>;
}

impl<T, E, P> Provider<T> for Arc<P>
where
    P: Provider<T, Error = E>,
{
    type Error = E;

    async fn provide(&self) -> Result<T, Self::Error> {
        self.as_ref().provide().await
    }
}

/// Expiring value.
/// Stores the [`last_retrieval`] timestamp, and a [`max_age`].
#[derive(Debug, Clone)]
pub struct ExpiringValue<T> {
    value: T,
    last_retrieval: DateTime<Utc>,
    max_age: Duration,
}

impl<T> ExpiringValue<T> {
    /// Constructor.
    pub fn new(value: T, last_retrieval: DateTime<Utc>, max_age: Duration) -> Self {
        Self {
            value,
            last_retrieval,
            max_age,
        }
    }

    /// Constructor for an [`ExpiringValue`] retrieved now.
    pub fn now(value: T, max_age: Duration) -> Self {
        Self {
            value,
            last_retrieval: Utc::now(),
            max_age,
        }
    }

    pub fn is_expired_at(&self, timestamp: DateTime<Utc>) -> bool {
        let crl_age = timestamp - self.last_retrieval;
        crl_age.num_seconds() >= self.max_age.as_secs() as i64
    }

    pub fn map<R, F>(self, transform: F) -> ExpiringValue<R>
    where
        F: FnOnce(T) -> R,
    {
        let ExpiringValue {
            value,
            last_retrieval,
            max_age,
        } = self;
        ExpiringValue::new(transform(value), last_retrieval, max_age)
    }
}

impl<T> Deref for ExpiringValue<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> Expiring for ExpiringValue<T> {
    async fn is_expired(&self) -> bool {
        self.is_expired_at(Utc::now())
    }
}

/// Cache that holds an [`Expiring`] value, uses a [`Provider`] to retrieve the data.
/// This cache is lazy, meaning that the [`Provider`] is only invoked when the cached value is requested.
pub struct ExpiringCache<T, P> {
    /// Provider for the cached data
    provider: P,
    /// In memory cache of the provided data
    cache: RwLock<Option<T>>,
}

impl<T, P> ExpiringCache<T, P> {
    pub fn new(provider: P) -> Self {
        Self {
            provider,
            cache: Default::default(),
        }
    }
}

impl<T, P, E> Provider<T> for ExpiringCache<T, P>
where
    T: Expiring + Clone,
    P: Provider<T, Error = E>,
{
    type Error = E;

    async fn provide(&self) -> Result<T, Self::Error> {
        if self.is_expired().await {
            let value = self.provider.provide().await?;
            let mut cache = self.cache.write().await;
            *cache = Some(value.clone());
            Ok(value)
        } else {
            let cache = self.cache.read().await;
            // unwrap is safe because cache is already initialized (i.e. not expired)
            Ok(cache.clone().unwrap())
        }
    }
}

impl<T, P, E> Expiring for ExpiringCache<T, P>
where
    T: Expiring + Clone,
    P: Provider<T, Error = E>,
{
    /// The cache is expired when the cache is not initialized yet, or if the data in the cache is expired.
    async fn is_expired(&self) -> bool {
        let cache = self.cache.read().await;
        match cache.as_ref() {
            Some(value) => value.is_expired().await,
            None => true,
        }
    }
}

/// Extension trait for [`Provider`], provides the [`map`] operation.
pub trait MapProvider<T, R, F>: Provider<T>
where
    F: Fn(T) -> R,
{
    type Provider: Provider<R>;

    fn map(self, transform: F) -> Self::Provider;
}

impl<T, P, E, R, F> MapProvider<T, R, F> for P
where
    R: Expiring + Clone,
    P: Provider<T, Error = E>,
    F: Fn(T) -> R,
{
    type Provider = ExpiringCache<R, MappedProvider<T, Self, F, R>>;

    fn map(self, transform: F) -> Self::Provider {
        let mapped_provider = MappedProvider::new(self, transform);
        ExpiringCache::new(mapped_provider)
    }
}

pub struct MappedProvider<S, PS, F, T> {
    source_provider: PS,
    transform: F,
    _source: PhantomData<S>, // Restrict `S`
    _target: PhantomData<T>, // Restrict `T`
}

impl<S, PS, F, T> MappedProvider<S, PS, F, T>
where
    PS: Provider<S>,
    F: Fn(S) -> T,
{
    fn new(source_provider: PS, transform: F) -> Self {
        Self {
            source_provider,
            transform,
            _source: Default::default(),
            _target: Default::default(),
        }
    }
}

impl<S, PS, F, T, E> Provider<T> for MappedProvider<S, PS, F, T>
where
    PS: Provider<S, Error = E>,
    F: Fn(S) -> T,
{
    type Error = E;

    async fn provide(&self) -> Result<T, Self::Error> {
        let val = self.source_provider.provide().await?;
        let result = (self.transform)(val);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    const PAST: DateTime<Utc> = DateTime::from_timestamp_millis(1_000_000).unwrap();
    const PRESENT: DateTime<Utc> = DateTime::from_timestamp_millis(2_000_000).unwrap();

    const SHORT: Duration = Duration::from_millis(500_000);
    const LONG: Duration = Duration::from_millis(1_500_000);
    const VERY_LONG: Duration = Duration::from_millis(2_500_000);

    #[rstest]
    #[case(ExpiringValue::new((), PAST, SHORT), true)]
    #[case(ExpiringValue::new((), PAST, LONG), false)]
    fn test_expiring_value(#[case] test_subject: ExpiringValue<()>, #[case] expected_to_be_expired: bool) {
        assert_eq!(test_subject.is_expired_at(PRESENT), expected_to_be_expired);
    }

    #[test]
    fn test_expiring_value_map() {
        let test_subject = ExpiringValue::now(3, VERY_LONG);
        assert_eq!(*test_subject.map(|x| x * 2), 6);
    }

    /// [`Provider`] of [`ExpiringData`] to unit test [`ExpiringDataCache`].
    /// Provides the number of times the [`provide()`] function has been invoked.
    #[derive(Default, Clone)]
    struct Counter {
        provide_count: Arc<RwLock<u8>>,
    }

    impl Counter {
        async fn provide_count(&self) -> u8 {
            let provide_count = self.provide_count.read().await;
            *provide_count
        }
    }

    impl Provider<u8> for Counter {
        type Error = ();

        async fn provide(&self) -> Result<u8, Self::Error> {
            let mut provide_count = self.provide_count.write().await;
            *provide_count += 1;
            Ok(*provide_count)
        }
    }

    impl Expiring for u8 {
        async fn is_expired(&self) -> bool {
            // This allows us to test both the initial initialization of the cache, and expiration of the data.
            *self < 2
        }
    }

    impl Expiring for u16 {
        async fn is_expired(&self) -> bool {
            *self < 8
        }
    }

    #[tokio::test]
    async fn test_provider() {
        let provider = Counter::default();

        let actual = provider.provide().await.unwrap();
        assert_eq!(actual, 1);

        let actual = provider.provide().await.unwrap();
        assert_eq!(actual, 2);

        let actual = provider.provide().await.unwrap();
        assert_eq!(actual, 3);

        let actual = provider.provide().await.unwrap();
        assert_eq!(actual, 4);
    }

    #[tokio::test]
    async fn test_cache() {
        let cache: ExpiringCache<u8, Counter> = ExpiringCache::new(Counter::default());
        // Verify provider not yet invoked
        assert_eq!(cache.provider.provide_count().await, 0);

        // Invoke cached provider
        assert!(cache.is_expired().await);
        let actual: u8 = cache.provide().await.unwrap();
        assert_eq!(actual, 1);
        // Verify provider invoked once, because cache not initialized
        assert_eq!(cache.provider.provide_count().await, 1);

        // Invoke cached provider
        assert!(cache.is_expired().await);
        let actual: u8 = cache.provide().await.unwrap();
        assert_eq!(actual, 2);
        // Verify provider invoked again, because data expired
        assert_eq!(cache.provider.provide_count().await, 2);

        // Invoke cached provider
        assert!(!cache.is_expired().await);
        let actual: u8 = cache.provide().await.unwrap();
        assert_eq!(actual, 2);
        // Verify provider not invoked again
        assert_eq!(cache.provider.provide_count().await, 2);

        // Invoke cached provider
        assert!(!cache.is_expired().await);
        let actual: u8 = cache.provide().await.unwrap();
        assert_eq!(actual, 2);
        // Verify provider not invoked again
        assert_eq!(cache.provider.provide_count().await, 2);
    }

    #[tokio::test]
    async fn test_mapped_cache_times_2() {
        let cache: ExpiringCache<u8, Counter> = ExpiringCache::new(Counter::default());
        let inner_cache = Arc::new(cache);

        let cache: ExpiringCache<_, _> = inner_cache.clone().map(|c| c as u16 * 2);

        // Verify provider not yet invoked
        assert_eq!(inner_cache.provider.provide_count().await, 0);

        // Invoke cached provider
        assert!(cache.is_expired().await);
        let actual: u16 = cache.provide().await.unwrap();
        assert_eq!(actual, 2);
        // Verify provider invoked once, because cache not initialized
        assert_eq!(inner_cache.provider.provide_count().await, 1);

        // Invoke cached provider
        assert!(cache.is_expired().await);
        let actual: u16 = cache.provide().await.unwrap();
        assert_eq!(actual, 4);
        // Verify provider invoked again, because data expired
        assert_eq!(inner_cache.provider.provide_count().await, 2);

        // Invoke cached provider
        assert!(cache.is_expired().await);
        let actual: u16 = cache.provide().await.unwrap();
        assert_eq!(actual, 4);
        // Verify provider not invoked again
        assert_eq!(inner_cache.provider.provide_count().await, 2);

        // Invoke cached provider
        assert!(cache.is_expired().await);
        let actual: u16 = cache.provide().await.unwrap();
        assert_eq!(actual, 4);
        // Verify provider not invoked again
        assert_eq!(inner_cache.provider.provide_count().await, 2);
    }

    #[tokio::test]
    async fn test_mapped_cache_times_4() {
        let cache: ExpiringCache<u8, Counter> = ExpiringCache::new(Counter::default());
        let inner_cache = Arc::new(cache);

        let cache: ExpiringCache<_, _> = inner_cache.clone().map(|c| c as u16 * 4);

        // Verify provider not yet invoked
        assert_eq!(inner_cache.provider.provide_count().await, 0);

        // Invoke cached provider
        assert!(cache.is_expired().await);
        let actual: u16 = cache.provide().await.unwrap();
        assert_eq!(actual, 4);
        // Verify provider invoked once, because cache not initialized
        assert_eq!(inner_cache.provider.provide_count().await, 1);

        // Invoke cached provider
        assert!(cache.is_expired().await);
        let actual: u16 = cache.provide().await.unwrap();
        assert_eq!(actual, 8);
        // Verify provider invoked again, because data expired
        assert_eq!(inner_cache.provider.provide_count().await, 2);

        // Invoke cached provider
        assert!(!cache.is_expired().await);
        let actual: u16 = cache.provide().await.unwrap();
        assert_eq!(actual, 8);
        // Verify provider not invoked again
        assert_eq!(inner_cache.provider.provide_count().await, 2);

        // Invoke cached provider
        assert!(!cache.is_expired().await);
        let actual: u16 = cache.provide().await.unwrap();
        assert_eq!(actual, 8);
        // Verify provider not invoked again
        assert_eq!(inner_cache.provider.provide_count().await, 2);
    }
}
