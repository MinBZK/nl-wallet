use std::error::Error;

#[derive(Debug, thiserror::Error)]
pub enum NonceStoreError<E> {
    #[error("nonce is already present: {0}")]
    DuplicateNonce(String),

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

    async fn store_nonce(&self, nonce: String) -> Result<(), NonceStoreError<Self::Error>>;
    async fn check_nonce_status_and_remove<'a>(
        &self,
        nonces: impl IntoIterator<Item = &'a str> + Send,
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

    use super::super::C_NONCE_VALIDITY;
    use super::NonceStatus;
    use super::NonceStore;
    use super::NonceStoreError;

    pub async fn test_nonce_store<N>(store: N, mock_time: Arc<RwLock<DateTime<Utc>>>)
    where
        N: NonceStore,
    {
        // Storing distinct nonces should succeed.
        store
            .store_nonce("foo".to_string())
            .await
            .expect("storing nonce should succeed");

        store
            .store_nonce("bar".to_string())
            .await
            .expect("storing nonce should succeed");

        store
            .store_nonce("barfoo".to_string())
            .await
            .expect("storing nonce should succeed");

        // Storing a nonce that is already stored should result in an error.
        let error = store
            .store_nonce("foo".to_string())
            .await
            .expect_err("storing nonce should fail");

        assert_matches!(error, NonceStoreError::DuplicateNonce(nonce) if nonce == "foo");

        // Retrieving a nonce once should be valid, after that the nonce should be absent.
        // If multiple nonces are checked and at least one of them is absent, this counts as a failure.
        let status = store
            .check_nonce_status_and_remove(["foo", "bar", "foo"])
            .await
            .expect("checking nonce status should succeed");

        assert_matches!(status, NonceStatus::AllValid);

        let status = store
            .check_nonce_status_and_remove(["foo", "barfoo"])
            .await
            .expect("checking nonce status should succeed");

        assert_matches!(status, NonceStatus::AtLeastOneAbsentOrExpired);

        let status = store
            .check_nonce_status_and_remove(["foo"])
            .await
            .expect("checking nonce status should succeed");

        assert_matches!(status, NonceStatus::AtLeastOneAbsentOrExpired);

        let status = store
            .check_nonce_status_and_remove(["barfoo"])
            .await
            .expect("checking nonce status should succeed");

        assert_matches!(status, NonceStatus::AtLeastOneAbsentOrExpired);

        // Retrieving the nonce after the validity period should cause it to be expired.
        store
            .store_nonce("foobar".to_string())
            .await
            .expect("storing nonce should succeed");

        *mock_time.write() += C_NONCE_VALIDITY + Duration::from_millis(1);

        let status = store
            .check_nonce_status_and_remove(["foobar"])
            .await
            .expect("checking nonce status should succeed");

        assert_matches!(status, NonceStatus::AtLeastOneAbsentOrExpired);
    }
}
