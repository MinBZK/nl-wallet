use std::collections::HashMap;
use std::convert::Infallible;
use std::error::Error;
use std::sync::Mutex;

use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;

use crate::authorization::AuthorizationRequest;

/// TTL for PAR entries. Per [RFC 9126 §2.2], the `expires_in` value should be short.
///
/// [RFC 9126 §2.2]: https://www.rfc-editor.org/rfc/rfc9126#section-2.2
pub const PAR_TTL: Duration = Duration::seconds(60);

#[trait_variant::make(Send)]
pub trait ParStore {
    type Error: Error + Send + Sync + 'static;

    async fn store(
        &self,
        request_uri: String,
        data: AuthorizationRequest,
        expires_at: DateTime<Utc>,
    ) -> Result<(), Self::Error>;

    async fn consume(&self, request_uri: &str) -> Result<Option<AuthorizationRequest>, Self::Error>;

    async fn cleanup(&self) -> Result<(), Self::Error>;
}

#[derive(Debug, Default)]
pub struct MemoryParStore(Mutex<HashMap<String, (AuthorizationRequest, DateTime<Utc>)>>);

impl MemoryParStore {
    fn store_inner(&self, request_uri: String, data: AuthorizationRequest, expires_at: DateTime<Utc>) {
        let Self(entries) = self;
        let mut entries = entries.lock().expect("there should be no panic while the lock is held");
        entries.insert(request_uri, (data, expires_at));
    }

    fn consume_inner(&self, request_uri: &str) -> Option<AuthorizationRequest> {
        let Self(entries) = self;
        let mut entries = entries.lock().expect("there should be no panic while the lock is held");

        let (data, expires_at) = entries.remove(request_uri)?;

        if Utc::now() > expires_at { None } else { Some(data) }
    }

    fn cleanup_inner(&self) {
        let Self(entries) = self;
        let mut entries = entries.lock().expect("there should be no panic while the lock is held");
        let now = Utc::now();
        entries.retain(|_, (_, expires_at)| *expires_at > now);
    }
}

impl ParStore for MemoryParStore {
    type Error = Infallible;

    async fn store(
        &self,
        request_uri: String,
        data: AuthorizationRequest,
        expires_at: DateTime<Utc>,
    ) -> Result<(), Self::Error> {
        self.store_inner(request_uri, data, expires_at);
        Ok(())
    }

    async fn consume(&self, request_uri: &str) -> Result<Option<AuthorizationRequest>, Self::Error> {
        Ok(self.consume_inner(request_uri))
    }

    async fn cleanup(&self) -> Result<(), Self::Error> {
        self.cleanup_inner();
        Ok(())
    }
}

/// A no-op [`ParStore`] implementation for use when PAR is not needed.
impl ParStore for () {
    type Error = Infallible;

    async fn store(&self, _: String, _: AuthorizationRequest, _: DateTime<Utc>) -> Result<(), Infallible> {
        Ok(())
    }

    async fn consume(&self, _: &str) -> Result<Option<AuthorizationRequest>, Infallible> {
        Ok(None)
    }

    async fn cleanup(&self) -> Result<(), Infallible> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::MemoryParStore;
    use super::ParStore;
    use crate::authorization::AuthorizationRequest;
    use crate::authorization::ResponseType;

    fn example_request() -> AuthorizationRequest {
        AuthorizationRequest {
            response_type: ResponseType::Code.into(),
            client_id: "client-1".to_string(),
            redirect_uri: None,
            state: None,
            authorization_details: None,
            code_challenge: None,
            scope: None,
            nonce: None,
            response_mode: None,
        }
    }

    #[tokio::test]
    async fn test_store_and_consume() {
        let store = MemoryParStore::default();
        let request_uri = "urn:ietf:params:oauth:request_uri:test".to_string();
        let expires_at = chrono::Utc::now() + Duration::seconds(60);

        store
            .store(request_uri.clone(), example_request(), expires_at)
            .await
            .unwrap();

        let result = store.consume(&request_uri).await.unwrap();
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_consume_removes_entry() {
        let store = MemoryParStore::default();
        let request_uri = "urn:ietf:params:oauth:request_uri:test".to_string();
        let expires_at = chrono::Utc::now() + Duration::seconds(60);

        store
            .store(request_uri.clone(), example_request(), expires_at)
            .await
            .unwrap();

        store.consume(&request_uri).await.unwrap();

        let result = store.consume(&request_uri).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_consume_expired() {
        let store = MemoryParStore::default();
        let request_uri = "urn:ietf:params:oauth:request_uri:test".to_string();
        let expires_at = chrono::Utc::now() - Duration::seconds(1);

        store
            .store(request_uri.clone(), example_request(), expires_at)
            .await
            .unwrap();

        let result = store.consume(&request_uri).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_consume_unknown_uri() {
        let store = MemoryParStore::default();

        let result = store.consume("unknown").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cleanup_removes_expired() {
        let store = MemoryParStore::default();
        let expired_uri = "urn:ietf:params:oauth:request_uri:expired".to_string();
        let valid_uri = "urn:ietf:params:oauth:request_uri:valid".to_string();

        store
            .store(
                expired_uri.clone(),
                example_request(),
                chrono::Utc::now() - Duration::seconds(1),
            )
            .await
            .unwrap();
        store
            .store(
                valid_uri.clone(),
                example_request(),
                chrono::Utc::now() + Duration::seconds(60),
            )
            .await
            .unwrap();

        store.cleanup().await.unwrap();

        assert!(store.consume(&expired_uri).await.unwrap().is_none());
        assert!(store.consume(&valid_uri).await.unwrap().is_some());
    }
}
