#[cfg(any(test, feature = "test"))]
pub mod test {
    use std::collections::HashSet;

    use oauth::pkce::PkcePair;
    use oauth::pkce::S256PkcePair;

    use crate::authorization::VciAuthorizationRequest;
    use crate::store::Store;

    fn example_request() -> VciAuthorizationRequest {
        VciAuthorizationRequest::for_auth_code(
            String::from("client-1"),
            "uri://redirect_uri".parse().unwrap(),
            String::from("state"),
            None,
            HashSet::new(),
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
        assert!(result.live().is_some());
        assert_eq!(count_entries(&store).await, 0);

        // Consuming the same entry a second time yields nothing.
        let result = store.consume(request_uri.as_str()).await.unwrap();
        assert!(result.live().is_none());

        // Consuming an unknown URI yields nothing.
        let result = store
            .consume("urn:ietf:params:oauth:request_uri:unknown")
            .await
            .unwrap();
        assert!(result.live().is_none());

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
