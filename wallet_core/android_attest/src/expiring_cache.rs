use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;

/// Expiring Data.
/// This trait marks data that can expire, for example by expiration time/date.
pub trait Expiring {
    /// Returns true when the data is expired at the moment of calling.
    fn is_expired(&self) -> bool;
}

pub struct ExpiringValue<T> {
    value: T,
    last_retrieval: DateTime<Utc>,
    max_age: Duration,
}

impl<T> ExpiringValue<T> {
    pub fn new(value: T, max_age: Duration) -> Self {
        Self {
            value,
            last_retrieval: Utc::now(),
            max_age,
        }
    }
}

impl<T> Deref for ExpiringValue<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> Expiring for ExpiringValue<T> {
    fn is_expired(&self) -> bool {
        let now = Utc::now();
        let crl_age = now - self.last_retrieval;
        crl_age.num_seconds() >= self.max_age.as_secs() as i64
    }
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

/// Cache that holds [`ExpiringData`], uses [`provider`] to retrieve the data.
/// This cache is lazy, meaning that [`provider`] is only invoked when needed.
pub struct ExpiringCache<T, P> {
    /// Provider for the cached data
    provider: P,
    /// In memory cache of the provided data
    cache: Arc<Mutex<Option<T>>>,
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
        if self.is_expired() {
            // Invoke provider outside of the scope which holds the mutex lock.
            let item = self.provider.provide().await?;

            // Lock the mutex in an as small as possible scope, so that locking will not err.
            let mut lock = self.cache.lock().unwrap();
            *lock = Some(item.clone());
            Ok(item)
        } else {
            // Lock the mutex in an as small as possible scope, so that locking will not err.
            let lock = self.cache.lock().unwrap();
            // Unwrap is safe, because of the `is_expired()` call above and the fact that we never set the
            // cache to `None`, apart from [`ExpiringDataCache::new`].
            Ok(lock.clone().unwrap())
        }
    }
}

impl<T, P, E> Expiring for ExpiringCache<T, P>
where
    T: Expiring + Clone,
    P: Provider<T, Error = E>,
{
    /// The cache is expired when the cache is not initialized yet, or if the data in the cache is expired.
    fn is_expired(&self) -> bool {
        // Lock the mutex in an as small as possible scope, so that locking will not err.
        let cache = self.cache.lock().unwrap();

        match cache.as_ref() {
            Some(cache) => cache.is_expired(),
            None => true,
        }
    }
}

pub trait MapProvider<T, R, F>: Provider<T>
where
    F: Fn(T) -> R,
{
    type Provider: Provider<R>;

    fn map(self, transform: F) -> Self::Provider;
}

// impl<T, P, E, R, F> MapProvider<T, R, F> for ExpiringCache<T, P>
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

pub struct MappedProvider<S, PS, F, T>
where
    PS: Provider<S>,
    F: Fn(S) -> T,
{
    source_provider: PS,
    transform: F,
    _source: PhantomData<S>,
    _target: PhantomData<T>,
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
    use super::*;

    /// [`Provider`] of [`ExpiringData`] to unit test [`ExpiringDataCache`].
    /// Provides the number of times the [`provide()`] function has been invoked.
    #[derive(Default, Clone)]
    struct Counter {
        provide_count: Arc<Mutex<u8>>,
    }

    impl Counter {
        fn provide_count(&self) -> u8 {
            let provide_count = self.provide_count.lock().unwrap();
            *provide_count
        }
    }

    impl Provider<u8> for Counter {
        type Error = ();

        async fn provide(&self) -> Result<u8, Self::Error> {
            let mut provide_count = self.provide_count.lock().unwrap();
            *provide_count += 1;
            Ok(*provide_count)
        }
    }

    impl Expiring for u8 {
        fn is_expired(&self) -> bool {
            // This allows us to test both the initial initialization of the cache, and expiration of the data.
            *self < 2
        }
    }

    impl Expiring for u16 {
        fn is_expired(&self) -> bool {
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
        assert_eq!(cache.provider.provide_count(), 0);

        // Invoke cached provider
        assert!(cache.is_expired());
        let actual: u8 = cache.provide().await.unwrap();
        assert_eq!(actual, 1);
        // Verify provider invoked once, because cache not initialized
        assert_eq!(cache.provider.provide_count(), 1);

        // Invoke cached provider
        assert!(cache.is_expired());
        let actual: u8 = cache.provide().await.unwrap();
        assert_eq!(actual, 2);
        // Verify provider invoked again, because data expired
        assert_eq!(cache.provider.provide_count(), 2);

        // Invoke cached provider
        assert!(!cache.is_expired());
        let actual: u8 = cache.provide().await.unwrap();
        assert_eq!(actual, 2);
        // Verify provider not invoked again
        assert_eq!(cache.provider.provide_count(), 2);

        // Invoke cached provider
        assert!(!cache.is_expired());
        let actual: u8 = cache.provide().await.unwrap();
        assert_eq!(actual, 2);
        // Verify provider not invoked again
        assert_eq!(cache.provider.provide_count(), 2);
    }

    #[tokio::test]
    async fn test_mapped_cache_times_2() {
        let cache: ExpiringCache<u8, Counter> = ExpiringCache::new(Counter::default());
        let inner_cache = Arc::new(cache);

        let cache: ExpiringCache<_, _> = inner_cache.clone().map(|c| c as u16 * 2);

        // Verify provider not yet invoked
        assert_eq!(inner_cache.provider.provide_count(), 0);

        // Invoke cached provider
        assert!(cache.is_expired());
        let actual: u16 = cache.provide().await.unwrap();
        assert_eq!(actual, 2);
        // Verify provider invoked once, because cache not initialized
        assert_eq!(inner_cache.provider.provide_count(), 1);

        // Invoke cached provider
        assert!(cache.is_expired());
        let actual: u16 = cache.provide().await.unwrap();
        assert_eq!(actual, 4);
        // Verify provider invoked again, because data expired
        assert_eq!(inner_cache.provider.provide_count(), 2);

        // Invoke cached provider
        assert!(cache.is_expired());
        let actual: u16 = cache.provide().await.unwrap();
        assert_eq!(actual, 4);
        // Verify provider not invoked again
        assert_eq!(inner_cache.provider.provide_count(), 2);

        // Invoke cached provider
        assert!(cache.is_expired());
        let actual: u16 = cache.provide().await.unwrap();
        assert_eq!(actual, 4);
        // Verify provider not invoked again
        assert_eq!(inner_cache.provider.provide_count(), 2);
    }

    #[tokio::test]
    async fn test_mapped_cache_times_4() {
        let cache: ExpiringCache<u8, Counter> = ExpiringCache::new(Counter::default());
        let inner_cache = Arc::new(cache);

        let cache: ExpiringCache<_, _> = inner_cache.clone().map(|c| c as u16 * 4);

        // Verify provider not yet invoked
        assert_eq!(inner_cache.provider.provide_count(), 0);

        // Invoke cached provider
        assert!(cache.is_expired());
        let actual: u16 = cache.provide().await.unwrap();
        assert_eq!(actual, 4);
        // Verify provider invoked once, because cache not initialized
        assert_eq!(inner_cache.provider.provide_count(), 1);

        // Invoke cached provider
        assert!(cache.is_expired());
        let actual: u16 = cache.provide().await.unwrap();
        assert_eq!(actual, 8);
        // Verify provider invoked again, because data expired
        assert_eq!(inner_cache.provider.provide_count(), 2);

        // Invoke cached provider
        assert!(!cache.is_expired());
        let actual: u16 = cache.provide().await.unwrap();
        assert_eq!(actual, 8);
        // Verify provider not invoked again
        assert_eq!(inner_cache.provider.provide_count(), 2);

        // Invoke cached provider
        assert!(!cache.is_expired());
        let actual: u16 = cache.provide().await.unwrap();
        assert_eq!(actual, 8);
        // Verify provider not invoked again
        assert_eq!(inner_cache.provider.provide_count(), 2);
    }
}
