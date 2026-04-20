use std::error::Error;

use jwt::nonce::Nonce;

#[derive(Debug, thiserror::Error)]
pub enum NonceStoreError<E> {
    #[error("nonce is already present: {0}")]
    DuplicateNonce(Nonce),

    #[error("{0}")]
    Error(#[from] E),
}

#[derive(Debug, Clone, Copy)]
pub enum NonceStatus {
    AtLeastOneAbsentOrExpired,
    AllValid,
}

#[trait_variant::make(Send)]
pub trait NonceStore {
    type Error: Error + Send + Sync + 'static;

    async fn store_nonce(&self, nonce: Nonce) -> Result<(), NonceStoreError<Self::Error>>;
    async fn check_nonce_status_and_remove<'a>(
        &self,
        nonces: impl IntoIterator<Item = &'a Nonce> + Send,
    ) -> Result<NonceStatus, Self::Error>;

    // TODO (PVW-5678): Add method for cleaning up nonces that are older than a certain date and time.
}

#[cfg(any(test, feature = "test"))]
pub mod test {
    use std::sync::Arc;
    use std::time::Duration;

    use assert_matches::assert_matches;
    use chrono::DateTime;
    use chrono::Utc;
    use parking_lot::RwLock;

    use jwt::nonce::Nonce;

    use super::super::C_NONCE_VALIDITY;
    use super::NonceStatus;
    use super::NonceStore;
    use super::NonceStoreError;

    pub async fn test_nonce_store<N, F>(store: N, mock_time: Arc<RwLock<DateTime<Utc>>>, mut count_nonces: F)
    where
        N: NonceStore,
        F: AsyncFnMut(&N) -> usize,
    {
        // Storing distinct nonces should succeed.
        store
            .store_nonce(Nonce::from("foo".to_string()))
            .await
            .expect("storing nonce should succeed");

        store
            .store_nonce(Nonce::from("bar".to_string()))
            .await
            .expect("storing nonce should succeed");

        store
            .store_nonce(Nonce::from("barfoo".to_string()))
            .await
            .expect("storing nonce should succeed");

        // Storing a nonce that is already stored should result in an error.
        let error = store
            .store_nonce(Nonce::from("foo".to_string()))
            .await
            .expect_err("storing nonce should fail");

        assert_matches!(error, NonceStoreError::DuplicateNonce(nonce) if nonce.as_ref() == "foo");

        // Retrieving a nonce once should be valid, after that the nonce should be absent.
        // If multiple nonces are checked and at least one of them is absent, this counts as a failure.
        assert_eq!(count_nonces(&store).await, 3);

        let status = store
            .check_nonce_status_and_remove(&[
                Nonce::from("foo".to_string()),
                Nonce::from("bar".to_string()),
                Nonce::from("foo".to_string()),
            ])
            .await
            .expect("checking nonce status should succeed");

        assert_matches!(status, NonceStatus::AllValid);
        assert_eq!(count_nonces(&store).await, 1);

        let status = store
            .check_nonce_status_and_remove(&[Nonce::from("foo".to_string()), Nonce::from("barfoo".to_string())])
            .await
            .expect("checking nonce status should succeed");

        assert_matches!(status, NonceStatus::AtLeastOneAbsentOrExpired);
        assert_eq!(count_nonces(&store).await, 0);

        let status = store
            .check_nonce_status_and_remove(&[Nonce::from("foo".to_string())])
            .await
            .expect("checking nonce status should succeed");

        assert_matches!(status, NonceStatus::AtLeastOneAbsentOrExpired);
        assert_eq!(count_nonces(&store).await, 0);

        let status = store
            .check_nonce_status_and_remove(&[Nonce::from("barfoo".to_string())])
            .await
            .expect("checking nonce status should succeed");

        assert_matches!(status, NonceStatus::AtLeastOneAbsentOrExpired);
        assert_eq!(count_nonces(&store).await, 0);

        // Retrieving the nonce after the validity period should cause it to be expired and removed.
        store
            .store_nonce(Nonce::from("foobar".to_string()))
            .await
            .expect("storing nonce should succeed");

        assert_eq!(count_nonces(&store).await, 1);

        *mock_time.write() += C_NONCE_VALIDITY + Duration::from_millis(1);

        let status = store
            .check_nonce_status_and_remove(&[Nonce::from("foobar".to_string())])
            .await
            .expect("checking nonce status should succeed");

        assert_matches!(status, NonceStatus::AtLeastOneAbsentOrExpired);
        assert_eq!(count_nonces(&store).await, 0);
    }
}
