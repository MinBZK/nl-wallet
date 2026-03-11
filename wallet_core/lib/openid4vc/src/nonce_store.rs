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
    async fn find_and_remove_nonce(&self, nonce: &str) -> Result<NoncePresence, Self::Error>;

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

    async fn find_and_remove_nonce(&self, nonce: &str) -> Result<NoncePresence, Self::Error> {
        let presence = self.remove(nonce);

        Ok(presence)
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use futures::FutureExt;

    use super::MemoryNonceStore;
    use super::NoncePresence;
    use super::NonceStore;
    use super::NonceStoreError;

    #[test]
    fn test_memory_nonce_store() {
        let store = MemoryNonceStore::default();

        store
            .store_nonce("foobar".to_string())
            .now_or_never()
            .unwrap()
            .expect("storing nonce should succeed");

        store
            .store_nonce("barfoo".to_string())
            .now_or_never()
            .unwrap()
            .expect("storing nonce should succeed");

        let error = store
            .store_nonce("foobar".to_string())
            .now_or_never()
            .unwrap()
            .expect_err("storing nonce should fail");

        assert_matches!(error, NonceStoreError::DuplicateNonce(nonce) if nonce == "foobar");

        let presence = store
            .find_and_remove_nonce("foobar")
            .now_or_never()
            .unwrap()
            .expect("finding and removing nonce should succeed");

        assert_matches!(presence, NoncePresence::Present);

        let presence = store
            .find_and_remove_nonce("foobar")
            .now_or_never()
            .unwrap()
            .expect("finding and removing nonce should succeed");

        assert_matches!(presence, NoncePresence::Absent);

        store
            .store_nonce("foobar".to_string())
            .now_or_never()
            .unwrap()
            .expect("storing nonce should succeed");
    }
}
