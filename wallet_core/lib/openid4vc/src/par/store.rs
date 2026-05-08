use std::collections::HashMap;
use std::convert::Infallible;
use std::error::Error;
use std::sync::Mutex;

use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;

use crate::authorization::VciAuthorizationRequest;

/// TTL for PAR entries. Per [RFC 9126 §2.2], the `expires_in` value should be short.
///
/// [RFC 9126 §2.2]: https://www.rfc-editor.org/rfc/rfc9126#section-2.2
pub const PAR_TTL: Duration = Duration::seconds(60);

#[trait_variant::make(Send)]
pub trait ParStore {
    type Error: Error + Send + Sync + 'static;

    async fn store(
        &self,
        request_uri: String,
        data: VciAuthorizationRequest,
        expires_at: DateTime<Utc>,
    ) -> Result<(), Self::Error>;

    async fn consume(&self, request_uri: &str) -> Result<Option<VciAuthorizationRequest>, Self::Error>;

    async fn cleanup(&self) -> Result<(), Self::Error>;
}

#[derive(Debug, Default)]
pub struct MemoryParStore(Mutex<HashMap<String, (VciAuthorizationRequest, DateTime<Utc>)>>);

impl MemoryParStore {
    fn store_inner(&self, request_uri: String, data: VciAuthorizationRequest, expires_at: DateTime<Utc>) {
        let Self(entries) = self;
        let mut entries = entries.lock().expect("there should be no panic while the lock is held");
        entries.insert(request_uri, (data, expires_at));
    }

    fn consume_inner(&self, request_uri: &str) -> Option<VciAuthorizationRequest> {
        let Self(entries) = self;
        let mut entries = entries.lock().expect("there should be no panic while the lock is held");

        let (data, expires_at) = entries.remove(request_uri)?;

        if Utc::now() > expires_at { None } else { Some(data) }
    }

    fn cleanup_inner(&self) {
        let Self(entries) = self;
        let mut entries = entries.lock().expect("there should be no panic while the lock is held");
        let now = Utc::now();
        entries.retain(|_, (_, expires_at)| *expires_at > now);
    }
}

impl ParStore for MemoryParStore {
    type Error = Infallible;

    async fn store(
        &self,
        request_uri: String,
        data: VciAuthorizationRequest,
        expires_at: DateTime<Utc>,
    ) -> Result<(), Self::Error> {
        self.store_inner(request_uri, data, expires_at);
        Ok(())
    }

    async fn consume(&self, request_uri: &str) -> Result<Option<VciAuthorizationRequest>, Self::Error> {
        Ok(self.consume_inner(request_uri))
    }

    async fn cleanup(&self) -> Result<(), Self::Error> {
        self.cleanup_inner();
        Ok(())
    }
}

/// A no-op [`ParStore`] implementation for use when PAR is not needed.
impl ParStore for () {
    type Error = Infallible;

    async fn store(&self, _: String, _: VciAuthorizationRequest, _: DateTime<Utc>) -> Result<(), Infallible> {
        Ok(())
    }

    async fn consume(&self, _: &str) -> Result<Option<VciAuthorizationRequest>, Infallible> {
        Ok(None)
    }

    async fn cleanup(&self) -> Result<(), Infallible> {
        Ok(())
    }
}

#[cfg(any(test, feature = "test"))]
pub mod test {
    use chrono::Duration;
    use chrono::Utc;

    use super::ParStore;
    use crate::authorization::VciAuthorizationRequest;
    use crate::pkce::PkcePair;
    use crate::pkce::S256PkcePair;

    fn example_request() -> VciAuthorizationRequest {
        VciAuthorizationRequest::for_par(
            String::from("client-1"),
            "uri://redirect_uri".parse().unwrap(),
            String::from("state"),
            &S256PkcePair::generate(),
        )
    }

    pub async fn test_par_store<P, F>(store: P, mut count_entries: F)
    where
        P: ParStore,
        F: AsyncFnMut(&P) -> usize,
    {
        let request_uri = "urn:ietf:params:oauth:request_uri:test".to_string();
        let valid_expiry = Utc::now() + Duration::seconds(60);
        let expired_expiry = Utc::now() - Duration::seconds(1);

        // Store a valid entry and consume it.
        store
            .store(request_uri.clone(), example_request(), valid_expiry)
            .await
            .unwrap();
        assert_eq!(count_entries(&store).await, 1);

        let result = store.consume(&request_uri).await.unwrap();
        assert!(result.is_some());
        assert_eq!(count_entries(&store).await, 0);

        // Consuming the same entry a second time returns None.
        let result = store.consume(&request_uri).await.unwrap();
        assert!(result.is_none());

        // Consuming an unknown URI returns None.
        let result = store
            .consume("urn:ietf:params:oauth:request_uri:unknown")
            .await
            .unwrap();
        assert!(result.is_none());

        // Consuming an expired entry returns None (but still removes it).
        let expired_uri = "urn:ietf:params:oauth:request_uri:expired".to_string();
        store
            .store(expired_uri.clone(), example_request(), expired_expiry)
            .await
            .unwrap();
        assert_eq!(count_entries(&store).await, 1);

        let result = store.consume(&expired_uri).await.unwrap();
        assert!(result.is_none());
        assert_eq!(count_entries(&store).await, 0);

        // Cleanup removes expired entries but leaves valid ones intact.
        let valid_uri = "urn:ietf:params:oauth:request_uri:valid".to_string();
        store
            .store(expired_uri.clone(), example_request(), expired_expiry)
            .await
            .unwrap();
        store
            .store(valid_uri.clone(), example_request(), valid_expiry)
            .await
            .unwrap();
        assert_eq!(count_entries(&store).await, 2);

        store.cleanup().await.unwrap();
        assert_eq!(count_entries(&store).await, 1);

        assert!(store.consume(&expired_uri).await.unwrap().is_none());
        assert!(store.consume(&valid_uri).await.unwrap().is_some());
    }
}

#[cfg(test)]
mod tests {
    use super::MemoryParStore;
    use super::test::test_par_store;

    #[tokio::test]
    async fn test_memory_par_store() {
        let store = MemoryParStore::default();
        test_par_store(store, async |store| {
            let MemoryParStore(map) = store;
            map.lock().unwrap().len()
        })
        .await;
    }
}
