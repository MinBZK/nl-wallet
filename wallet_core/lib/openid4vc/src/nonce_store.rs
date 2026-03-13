use std::collections::HashMap;
use std::error::Error;
use std::sync::Mutex;

use chrono::DateTime;
use chrono::Utc;

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

#[derive(Debug, Clone, Copy)]
pub enum NonceStoreResult {
    Stored,
    DuplicateEntry,
}

#[derive(Debug, Default)]
pub struct MemoryNonceStore(Mutex<HashMap<String, DateTime<Utc>>>);

impl MemoryNonceStore {
    pub fn store(&self, nonce: String) -> NonceStoreResult {
        let Self(nonces) = self;

        let mut nonces = nonces.lock().expect("there should be no panic while the lock is held");

        if nonces.contains_key(&nonce) {
            return NonceStoreResult::DuplicateEntry;
        }

        nonces.insert(nonce, Utc::now());

        NonceStoreResult::Stored
    }

    pub fn remove(&self, nonce: &str) -> NoncePresence {
        let Self(nonces) = self;

        let mut nonces = nonces.lock().expect("there should be no panic while the lock is held");

        if nonces.remove(nonce).is_some() {
            NoncePresence::Present
        } else {
            NoncePresence::Absent
        }
    }
}

impl NonceStore for MemoryNonceStore {
    type Error = std::convert::Infallible;

    async fn store_nonce(&self, nonce: String) -> Result<(), NonceStoreError<Self::Error>> {
        match self.store(nonce.clone()) {
            NonceStoreResult::Stored => Ok(()),
            NonceStoreResult::DuplicateEntry => Err(NonceStoreError::DuplicateNonce(nonce)),
        }
    }

    async fn check_nonce_presence_and_then_remove(&self, nonce: &str) -> Result<NoncePresence, Self::Error> {
        let presence = self.remove(nonce);

        Ok(presence)
    }
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

#[cfg(test)]
mod tests {
    use futures::FutureExt;

    use super::MemoryNonceStore;
    use super::test::test_nonce_store;

    #[test]
    fn test_memory_nonce_store() {
        test_nonce_store(MemoryNonceStore::default()).now_or_never().unwrap()
    }
}
