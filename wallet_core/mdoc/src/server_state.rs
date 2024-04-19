use std::{future::Future, sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use nutype::nutype;
use tokio::{task::JoinHandle, time};
use tracing::warn;

use wallet_common::utils::random_string;

#[derive(Debug, Clone)]
pub struct SessionState<T> {
    pub data: T,
    pub token: SessionToken,
    pub last_active: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum SessionStoreError {
    #[error("key not found")]
    NotFound,
    #[error("error while serializing: {0}")]
    Serialize(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("error while deserializing: {0}")]
    Deserialize(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("something went wrong: {0}")]
    Other(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
}

// For this trait we cannot use the `trait_variant::make()` macro to add the `Send` trait to the return type
// of the async methods, as the `start_cleanup_task()` default method itself needs that specific trait.
pub trait SessionStore<T> {
    fn get(
        &self,
        token: &SessionToken,
    ) -> impl Future<Output = Result<Option<SessionState<T>>, SessionStoreError>> + Send;
    fn write(&self, session: SessionState<T>) -> impl Future<Output = Result<(), SessionStoreError>> + Send;
    fn cleanup(&self) -> impl Future<Output = Result<(), SessionStoreError>> + Send;

    fn start_cleanup_task(self: Arc<Self>, interval: Duration) -> JoinHandle<()>
    where
        Self: Send + Sync + 'static,
    {
        let mut interval = time::interval(interval);
        tokio::spawn(async move {
            loop {
                interval.tick().await;
                if let Err(e) = self.cleanup().await {
                    warn!("error during session cleanup: {e}");
                }
            }
        })
    }
}

#[derive(Debug, Default)]
pub struct MemorySessionStore<T> {
    pub(crate) sessions: DashMap<SessionToken, SessionState<T>>,
}

impl<T> MemorySessionStore<T> {
    pub fn new() -> Self {
        Self {
            sessions: DashMap::new(),
        }
    }
}

impl<T> SessionState<T> {
    pub fn new(token: SessionToken, data: T) -> SessionState<T> {
        SessionState {
            data,
            token,
            last_active: Utc::now(),
        }
    }
}

/// After this amount of inactivity, a session should be cleaned up.
pub const SESSION_EXPIRY_MINUTES: u64 = 5;

/// The cleanup task that removes stale sessions runs every so often.
pub const CLEANUP_INTERVAL_SECONDS: u64 = 120;

impl<T: Clone + Send + Sync> SessionStore<T> for MemorySessionStore<T> {
    async fn get(&self, token: &SessionToken) -> Result<Option<SessionState<T>>, SessionStoreError> {
        Ok(self.sessions.get(token).map(|s| s.clone()))
    }

    async fn write(&self, session: SessionState<T>) -> Result<(), SessionStoreError> {
        self.sessions.insert(session.token.clone(), session);
        Ok(())
    }

    async fn cleanup(&self) -> Result<(), SessionStoreError> {
        let now = Utc::now();
        let cutoff = chrono::Duration::minutes(SESSION_EXPIRY_MINUTES as i64);
        self.sessions.retain(|_, session| now - session.last_active < cutoff);
        Ok(())
    }
}

/// Identifies a session in a URL, as passed from the issuer/RP to the holder using the `url` field of
/// [`ServiceEngagement`](super::iso::ServiceEngagement)) or [`ReaderEngagement`](super::iso::ReaderEngagement).
///
/// In issuance, this token is the part of the `ServiceEngagement` that identifies the session. During the session, the
/// issuer additionally chooses a `SessionId` that must after that be present in each protocol message. The
/// `SessionToken` is distinct from `SessionId` because the `ServiceEngagement` that contains the `SessionToken` may be
/// transmitted over an insecure channel (e.g. a QR code). By not using the `SessionId` for this, the issuer transmits
/// this to the holder in response to its first HTTPS request, so that it remains secret between them. Since in later
/// protocol messages the issuer enforces that the correct session ID is present, this means that only the party that
/// sends the first HTTP request can send later HTTP requests for the session.
#[nutype(derive(Debug, Clone, PartialEq, Eq, Hash, From, Into, Display, Serialize, Deserialize))]
pub struct SessionToken(String);

impl SessionToken {
    pub fn new_random() -> Self {
        random_string(32).into()
    }
}
