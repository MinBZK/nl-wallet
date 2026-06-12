use std::collections::HashMap;
use std::convert::Infallible;
use std::error::Error;
use std::hash::Hash;
use std::sync::Mutex;

use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use utils::generator::Generator;
use utils::generator::TimeGenerator;

#[trait_variant::make(Send)]
pub trait Store<K, V> {
    type Error: Error + Send + Sync + 'static;

    async fn store(&self, key: K, value: V) -> Result<(), Self::Error>;

    async fn consume(&self, key: impl Into<K> + Send) -> Result<Option<V>, Self::Error>;

    async fn cleanup(&self) -> Result<(), Self::Error>;
}

#[derive(Debug, Default)]
pub struct MemoryStore<K, V, G = TimeGenerator> {
    ttl: Duration,
    time: G,
    entries: Mutex<HashMap<K, (V, DateTime<Utc>)>>,
}

impl<K: Hash + Eq, V> MemoryStore<K, V> {
    pub fn new(ttl: Duration) -> Self {
        Self::new_with_time(ttl, TimeGenerator)
    }
}

impl<K: Hash + Eq, V, G> MemoryStore<K, V, G> {
    pub fn new_with_time(ttl: Duration, time: G) -> Self {
        Self {
            ttl,
            time,
            entries: Mutex::default(),
        }
    }
}

impl<K, V, G> MemoryStore<K, V, G> {
    pub fn len(&self) -> usize {
        self.entries
            .lock()
            .expect("there should be no panic while the lock is held")
            .len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<K: Hash + Eq, V, G: Generator<DateTime<Utc>>> MemoryStore<K, V, G> {
    pub fn store_inner(&self, key: K, value: V) {
        let expires_at = self.time.generate() + self.ttl;
        let mut entries = self
            .entries
            .lock()
            .expect("there should be no panic while the lock is held");
        entries.insert(key, (value, expires_at));
    }

    pub fn consume_inner(&self, key: &K) -> Option<V> {
        let now = self.time.generate();
        let mut entries = self
            .entries
            .lock()
            .expect("there should be no panic while the lock is held");
        match entries.remove(key) {
            Some((value, expires_at)) if now < expires_at => Some(value),
            _ => None,
        }
    }

    pub fn cleanup_inner(&self) {
        let now = self.time.generate();
        let mut entries = self
            .entries
            .lock()
            .expect("there should be no panic while the lock is held");
        entries.retain(|_, (_, expires_at)| *expires_at > now);
    }
}

impl<K, V, G> Store<K, V> for MemoryStore<K, V, G>
where
    K: Hash + Eq + Send + Sync,
    V: Send,
    G: Generator<DateTime<Utc>> + Send + Sync,
{
    type Error = Infallible;

    async fn store(&self, key: K, value: V) -> Result<(), Self::Error> {
        self.store_inner(key, value);
        Ok(())
    }

    async fn consume(&self, key: impl Into<K> + Send) -> Result<Option<V>, Self::Error> {
        Ok(self.consume_inner(&key.into()))
    }

    async fn cleanup(&self) -> Result<(), Self::Error> {
        self.cleanup_inner();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::DateTime;
    use chrono::Duration;
    use chrono::Utc;
    use parking_lot::RwLock;
    use utils::generator::mock::MockTimeGenerator;

    use super::MemoryStore;
    use super::Store;

    type TestMemoryStore = MemoryStore<String, String, MockTimeGenerator>;

    fn memory_store_with_mock_time() -> (TestMemoryStore, Arc<RwLock<DateTime<Utc>>>) {
        let generator = MockTimeGenerator::default();
        let mock_time = Arc::clone(&generator.time);
        let store = MemoryStore::new_with_time(Duration::seconds(60), generator);
        (store, mock_time)
    }

    #[tokio::test]
    async fn test_store_and_consume() {
        let store: MemoryStore<String, String> = MemoryStore::new(Duration::seconds(60));

        store.store("key".to_string(), "value".to_string()).await.unwrap();
        assert_eq!(store.len(), 1);

        assert!(store.consume("key").await.unwrap().is_some());
        assert_eq!(store.len(), 0);

        assert!(store.consume("key").await.unwrap().is_none());
        assert!(store.consume("unknown").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_consume_expired() {
        let (store, mock_time) = memory_store_with_mock_time();

        store.store("key".to_string(), "value".to_string()).await.unwrap();

        *mock_time.write() += Duration::seconds(61);

        assert!(store.consume("key").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_cleanup_removes_expired_leaves_valid() {
        let (store, mock_time) = memory_store_with_mock_time();

        let t0 = *mock_time.read();

        store.store("expired".to_string(), "v1".to_string()).await.unwrap();

        *mock_time.write() = t0 + Duration::seconds(61);

        store.store("valid".to_string(), "v2".to_string()).await.unwrap();

        store.cleanup().await.unwrap();

        assert!(store.consume("expired").await.unwrap().is_none());
        assert!(store.consume("valid").await.unwrap().is_some());
    }
}
