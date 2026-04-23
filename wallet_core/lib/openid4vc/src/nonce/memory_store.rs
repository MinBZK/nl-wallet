use std::collections::HashMap;
use std::sync::Mutex;

use chrono::DateTime;
use chrono::Utc;
use itertools::Itertools;
use jwt::nonce::Nonce;
use utils::generator::Generator;
use utils::generator::TimeGenerator;

use super::C_NONCE_VALIDITY;
use super::store::NonceStatus;
use super::store::NonceStore;
use super::store::NonceStoreError;

#[derive(Debug, Clone, Copy)]
pub enum NonceStoreResult {
    Stored,
    DuplicateEntry,
}

#[derive(Debug)]
pub struct MemoryNonceStore<T = TimeGenerator> {
    nonces: Mutex<HashMap<Nonce, DateTime<Utc>>>,
    time_generator: T,
}

impl MemoryNonceStore {
    pub fn new() -> Self {
        Self {
            nonces: Mutex::new(HashMap::new()),
            time_generator: TimeGenerator,
        }
    }
}

impl Default for MemoryNonceStore {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> MemoryNonceStore<T>
where
    T: Generator<DateTime<Utc>>,
{
    fn is_valid(created_date_time: DateTime<Utc>, now: DateTime<Utc>) -> bool {
        created_date_time + C_NONCE_VALIDITY >= now
    }

    fn now(&self) -> DateTime<Utc> {
        self.time_generator.generate()
    }

    pub fn store(&self, nonce: Nonce) -> NonceStoreResult {
        let mut nonces = self
            .nonces
            .lock()
            .expect("there should be no panic while the lock is held");

        if nonces.contains_key(&nonce) {
            return NonceStoreResult::DuplicateEntry;
        }

        nonces.insert(nonce, self.now());

        NonceStoreResult::Stored
    }

    pub fn remove_and_check<'a>(&self, nonces: impl IntoIterator<Item = &'a Nonce>) -> NonceStatus {
        let mut stored_nonces = self
            .nonces
            .lock()
            .expect("there should be no panic while the lock is held");

        // Make sure all requested nonces are removed before checking their dates by collecting into a `Vec`.
        let removed_nonce_datetimes = nonces
            .into_iter()
            .unique()
            .map(|nonce| stored_nonces.remove(nonce))
            .collect_vec();

        let now = self.now();
        if removed_nonce_datetimes
            .into_iter()
            .all(|date_time| date_time.is_some_and(|created_date_time| Self::is_valid(created_date_time, now)))
        {
            NonceStatus::AllValid
        } else {
            NonceStatus::AtLeastOneAbsentOrExpired
        }
    }

    pub fn remove_expired(&self) {
        let mut nonces = self
            .nonces
            .lock()
            .expect("there should be no panic while the lock is held");

        let now = self.now();
        nonces.retain(|_nonce, created_date_time| Self::is_valid(*created_date_time, now));
    }
}

impl<T> NonceStore for MemoryNonceStore<T>
where
    T: Generator<DateTime<Utc>> + Send + Sync,
{
    type Error = std::convert::Infallible;

    async fn store_nonce(&self, nonce: Nonce) -> Result<(), NonceStoreError<Self::Error>> {
        match self.store(nonce.clone()) {
            NonceStoreResult::Stored => Ok(()),
            NonceStoreResult::DuplicateEntry => Err(NonceStoreError::DuplicateNonce(nonce)),
        }
    }

    async fn check_nonce_status_and_remove<'a>(
        &self,
        nonces: impl IntoIterator<Item = &'a Nonce> + Send,
    ) -> Result<NonceStatus, Self::Error> {
        let presence = self.remove_and_check(nonces);

        Ok(presence)
    }

    async fn remove_expired_nonces(&self) -> Result<(), NonceStoreError<Self::Error>> {
        self.remove_expired();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::sync::Mutex;

    use chrono::DateTime;
    use futures::FutureExt;
    use utils::generator::mock::MockTimeGenerator;

    use super::super::store::test::test_nonce_store;
    use super::MemoryNonceStore;

    #[test]
    fn test_memory_nonce_store() {
        let time_generator = MockTimeGenerator::new(DateTime::from_timestamp_secs(1_000_000_000).unwrap());
        let mock_time = Arc::clone(&time_generator.time);

        let store = MemoryNonceStore {
            nonces: Mutex::new(HashMap::new()),
            time_generator,
        };

        test_nonce_store(store, mock_time, async |store| store.nonces.lock().unwrap().len())
            .now_or_never()
            .unwrap();
    }
}
