use std::error::Error;

#[derive(Debug, thiserror::Error)]
pub enum NonceStoreError<E> {
    #[error("nonce is already present: {0}")]
    DuplicateNonce(String),

    #[error("{0}")]
    Error(#[from] E),
}

#[derive(Debug, Clone, Copy)]
pub enum NoncePresence {
    Present,
    Absent,
}

#[trait_variant::make(Send)]
pub trait NonceStore {
    type Error: Error + Send + Sync + 'static;

    async fn store_nonce(&self, nonce: String) -> Result<(), NonceStoreError<Self::Error>>;
    async fn check_nonce_presence_and_then_remove(&self, nonce: &str) -> Result<NoncePresence, Self::Error>;

    // TODO (PVW-5678): Add method for cleaning up nonces that are older than a certain date and time.
}

#[cfg(any(test, feature = "test"))]
pub mod test {
    use assert_matches::assert_matches;

    use super::NoncePresence;
    use super::NonceStore;
    use super::NonceStoreError;

    pub async fn test_nonce_store<N>(store: N)
    where
        N: NonceStore,
    {
        store
            .store_nonce("foobar".to_string())
            .await
            .expect("storing nonce should succeed");

        store
            .store_nonce("barfoo".to_string())
            .await
            .expect("storing nonce should succeed");

        let error = store
            .store_nonce("foobar".to_string())
            .await
            .expect_err("storing nonce should fail");

        assert_matches!(error, NonceStoreError::DuplicateNonce(nonce) if nonce == "foobar");

        let presence = store
            .check_nonce_presence_and_then_remove("foobar")
            .await
            .expect("finding and removing nonce should succeed");

        assert_matches!(presence, NoncePresence::Present);

        let presence = store
            .check_nonce_presence_and_then_remove("foobar")
            .await
            .expect("finding and removing nonce should succeed");

        assert_matches!(presence, NoncePresence::Absent);

        store
            .store_nonce("foobar".to_string())
            .await
            .expect("storing nonce should succeed");
    }
}
