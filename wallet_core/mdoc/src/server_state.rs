use std::{fmt::Display, sync::Arc, time::Duration};

use chrono::{DateTime, Local};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::{task::JoinHandle, time};
use wallet_common::utils::random_string;

#[derive(Debug, Clone)]
pub struct SessionState<T> {
    pub session_data: T,
    pub token: SessionToken,
    pub last_active: DateTime<Local>,
}

pub trait SessionStore {
    type Data: Clone;

    fn get(&self, id: &SessionToken) -> Option<Self::Data>;
    fn write(&self, session: &Self::Data);
    fn cleanup(&self);

    fn start_cleanup_task(sessions: Arc<Self>, interval: Duration) -> JoinHandle<()>
    where
        Self: Send + Sync + 'static,
    {
        let mut interval = time::interval(interval);
        tokio::spawn(async move {
            loop {
                interval.tick().await;
                sessions.cleanup();
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
            session_data: data,
            token,
            last_active: Local::now(),
        }
    }
}

/// After this amount of inactivity, a session should be cleaned up.
pub const SESSION_EXPIRY_MINUTES: u64 = 5;

/// The cleanup task that removes stale sessions runs every so often.
pub const CLEANUP_INTERVAL_SECONDS: u64 = 10;

impl<T: Clone> SessionStore for MemorySessionStore<T> {
    type Data = SessionState<T>;

    fn get(&self, token: &SessionToken) -> Option<SessionState<T>> {
        self.sessions.get(token).map(|s| s.clone())
    }

    fn write(&self, session: &SessionState<T>) {
        self.sessions.insert(session.token.clone(), session.clone());
    }

    fn cleanup(&self) {
        let now = Local::now();
        let cutoff = chrono::Duration::minutes(SESSION_EXPIRY_MINUTES as i64);
        self.sessions.retain(|_, session| now - session.last_active < cutoff);
    }
}

/// Identifies a session in a URL, as passed from the issuer/RP to the holder using the `url` field of
/// [`ServiceEngagement`](super::iso::ServiceEngagement)) or [`ReaderEngagement`](super::iso::ReaderEngagement).
///
/// In issuance, this token is the part of the `ServiceEngagement` that identifies the session. During the session, the
/// issuer additionally chooses a `SessionId` that must after that be present in each protocol message. The
/// `SessionToken` is distict from `SessionId` because the `ServiceEngagement` that contains the `SessionToken` may be
/// transmitted over an insecure channel (e.g. a QR code). By not using the `SessionId` for this, the issuer transmits
/// this to the holder in response to its first HTTPS request, so that it remains secret between them. Since in later
/// protocol messages the issuer enforces that the correct session ID is present, this means that only the party that
/// sends the first HTTP request can send later HTTP requests for the session.
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct SessionToken(pub(crate) String);

impl SessionToken {
    pub fn new() -> Self {
        random_string(32).into()
    }
}

impl From<String> for SessionToken {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Display for SessionToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
