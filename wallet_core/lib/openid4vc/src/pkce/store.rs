use std::collections::HashMap;
use std::convert::Infallible;
use std::error::Error;
use std::sync::Mutex;

use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;

/// TTL for [`PkceFlowStore`] entries. Bounds how long the user has to complete
/// the upstream authentication between `/authorize` and `/token`.
pub const PKCE_FLOW_TTL: Duration = Duration::minutes(10);

/// Bridges the wallet ↔ PID-issuer PKCE pair to the PID-issuer ↔ upstream PKCE
/// pair across a single authorization-code exchange. On `/authorize` the
/// handler stores the upstream `code_verifier` (`v2`) keyed by the wallet's
/// `code_challenge` (`c1`); on `/token` it consumes by recomputing `c1` from
/// the wallet's `code_verifier` (`v1`).
#[trait_variant::make(Send)]
pub trait PkceFlowStore {
    type Error: Error + Send + Sync + 'static;

    async fn store(
        &self,
        code_challenge: String,
        code_verifier: String,
        expires_at: DateTime<Utc>,
    ) -> Result<(), Self::Error>;

    async fn consume(&self, code_challenge: &str) -> Result<Option<String>, Self::Error>;

    async fn cleanup(&self) -> Result<(), Self::Error>;
}

#[derive(Debug, Default)]
pub struct MemoryPkceFlowStore(Mutex<HashMap<String, (String, DateTime<Utc>)>>);

impl MemoryPkceFlowStore {
    fn store_inner(&self, code_challenge: String, code_verifier: String, expires_at: DateTime<Utc>) {
        let Self(entries) = self;
        let mut entries = entries.lock().expect("there should be no panic while the lock is held");
        entries.insert(code_challenge, (code_verifier, expires_at));
    }

    fn consume_inner(&self, code_challenge: &str) -> Option<String> {
        let Self(entries) = self;
        let mut entries = entries.lock().expect("there should be no panic while the lock is held");

        let (code_verifier, expires_at) = entries.remove(code_challenge)?;

        if Utc::now() > expires_at {
            None
        } else {
            Some(code_verifier)
        }
    }

    fn cleanup_inner(&self) {
        let Self(entries) = self;
        let mut entries = entries.lock().expect("there should be no panic while the lock is held");
        let now = Utc::now();
        entries.retain(|_, (_, expires_at)| *expires_at > now);
    }
}

impl PkceFlowStore for MemoryPkceFlowStore {
    type Error = Infallible;

    async fn store(
        &self,
        code_challenge: String,
        code_verifier: String,
        expires_at: DateTime<Utc>,
    ) -> Result<(), Self::Error> {
        self.store_inner(code_challenge, code_verifier, expires_at);
        Ok(())
    }

    async fn consume(&self, code_challenge: &str) -> Result<Option<String>, Self::Error> {
        Ok(self.consume_inner(code_challenge))
    }

    async fn cleanup(&self) -> Result<(), Self::Error> {
        self.cleanup_inner();
        Ok(())
    }
}

/// A no-op [`PkceFlowStore`] implementation for use when PKCE bridging is not needed.
impl PkceFlowStore for () {
    type Error = Infallible;

    async fn store(&self, _: String, _: String, _: DateTime<Utc>) -> Result<(), Infallible> {
        Ok(())
    }

    async fn consume(&self, _: &str) -> Result<Option<String>, Infallible> {
        Ok(None)
    }

    async fn cleanup(&self) -> Result<(), Infallible> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::MemoryPkceFlowStore;
    use super::PkceFlowStore;

    #[tokio::test]
    async fn test_store_and_consume() {
        let store = MemoryPkceFlowStore::default();
        let expires_at = chrono::Utc::now() + Duration::minutes(10);

        store
            .store("c1".to_string(), "v2".to_string(), expires_at)
            .await
            .unwrap();

        let result = store.consume("c1").await.unwrap();
        assert_eq!(result.as_deref(), Some("v2"));
    }

    #[tokio::test]
    async fn test_consume_removes_entry() {
        let store = MemoryPkceFlowStore::default();
        let expires_at = chrono::Utc::now() + Duration::minutes(10);

        store
            .store("c1".to_string(), "v2".to_string(), expires_at)
            .await
            .unwrap();

        store.consume("c1").await.unwrap();

        let result = store.consume("c1").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_consume_expired() {
        let store = MemoryPkceFlowStore::default();
        let expires_at = chrono::Utc::now() - Duration::seconds(1);

        store
            .store("c1".to_string(), "v2".to_string(), expires_at)
            .await
            .unwrap();

        let result = store.consume("c1").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_consume_unknown_challenge() {
        let store = MemoryPkceFlowStore::default();

        let result = store.consume("unknown").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cleanup_removes_expired() {
        let store = MemoryPkceFlowStore::default();

        store
            .store(
                "expired".to_string(),
                "v_expired".to_string(),
                chrono::Utc::now() - Duration::seconds(1),
            )
            .await
            .unwrap();
        store
            .store(
                "valid".to_string(),
                "v_valid".to_string(),
                chrono::Utc::now() + Duration::minutes(10),
            )
            .await
            .unwrap();

        store.cleanup().await.unwrap();

        assert!(store.consume("expired").await.unwrap().is_none());
        assert!(store.consume("valid").await.unwrap().is_some());
    }
}
