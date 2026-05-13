use chrono::Duration;

/// TTL for PAR entries. Per [RFC 9126 §2.2], the `expires_in` value should be short.
///
/// [RFC 9126 §2.2]: https://www.rfc-editor.org/rfc/rfc9126#section-2.2
pub const PAR_TTL: Duration = Duration::seconds(60);

#[cfg(any(test, feature = "test"))]
pub mod test {
    use crate::authorization::VciAuthorizationRequest;
    use crate::pkce::PkcePair;
    use crate::pkce::S256PkcePair;
    use crate::store::Store;

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
        P: Store<String, VciAuthorizationRequest>,
        F: AsyncFnMut(&P) -> usize,
    {
        let request_uri = "urn:ietf:params:oauth:request_uri:test".to_string();

        // Store an entry and consume it.
        store.store(request_uri.clone(), example_request()).await.unwrap();
        assert_eq!(count_entries(&store).await, 1);

        let result = store.consume(request_uri.as_str()).await.unwrap();
        assert!(result.is_some());
        assert_eq!(count_entries(&store).await, 0);

        // Consuming the same entry a second time returns None.
        let result = store.consume(request_uri.as_str()).await.unwrap();
        assert!(result.is_none());

        // Consuming an unknown URI returns None.
        let result = store
            .consume("urn:ietf:params:oauth:request_uri:unknown")
            .await
            .unwrap();
        assert!(result.is_none());

        // Cleanup runs without error.
        store.cleanup().await.unwrap();
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::test::test_par_store;
    use crate::authorization::VciAuthorizationRequest;
    use crate::store::MemoryStore;

    #[tokio::test]
    async fn test_memory_par_store() {
        let store: MemoryStore<String, VciAuthorizationRequest> = MemoryStore::new(Duration::seconds(60));
        test_par_store(store, async |s| s.len()).await;
    }
}
