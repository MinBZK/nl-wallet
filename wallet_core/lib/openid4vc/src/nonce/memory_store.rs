use std::collections::HashMap;
use std::sync::Mutex;

use chrono::DateTime;
use chrono::Utc;

use super::store::NoncePresence;
use super::store::NonceStore;
use super::store::NonceStoreError;

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

#[cfg(test)]
mod tests {
    use futures::FutureExt;

    use super::super::store::test::test_nonce_store;
    use super::MemoryNonceStore;

    #[test]
    fn test_memory_nonce_store() {
        test_nonce_store(MemoryNonceStore::default()).now_or_never().unwrap()
    }
}
