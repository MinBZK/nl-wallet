use std::{
    future::Future,
    ops::{Deref, DerefMut},
    sync::Arc,
    time::Duration,
};

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use nutype::nutype;
use tokio::{
    task::JoinHandle,
    time::{self, MissedTickBehavior},
};
use tracing::warn;

use wallet_common::utils::random_string;

#[derive(Debug, Clone, Copy)]
pub enum Progress {
    Active,
    Finished { has_succeeded: bool },
}

pub trait HasProgress {
    fn progress(&self) -> Progress;
}

#[derive(Debug, Clone)]
pub struct SessionState<T> {
    pub data: T,
    pub token: SessionToken,
    pub last_active: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum SessionStoreError {
    #[error("token {0} already exists")]
    DuplicateToken(SessionToken),
    #[error("session with token {0} is expired")]
    Expired(SessionToken),
    #[error("error while serializing: {0}")]
    Serialize(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("error while deserializing: {0}")]
    Deserialize(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("something went wrong: {0}")]
    Other(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
}

// For this trait we cannot use the `trait_variant::make()` macro to add the `Send` trait to the return type
// of the async methods, as the `start_cleanup_task()` default method itself needs that specific trait.
pub trait SessionStore<T>
where
    T: HasProgress,
{
    fn get(
        &self,
        token: &SessionToken,
    ) -> impl Future<Output = Result<Option<SessionState<T>>, SessionStoreError>> + Send;
    fn write(
        &self,
        session: SessionState<T>,
        is_new: bool,
    ) -> impl Future<Output = Result<(), SessionStoreError>> + Send;
    fn cleanup(&self) -> impl Future<Output = Result<(), SessionStoreError>> + Send;

    fn start_cleanup_task(self: Arc<Self>, interval: Duration) -> JoinHandle<()>
    where
        Self: Send + Sync + 'static,
    {
        let mut interval = time::interval(interval);
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

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
    // Store the session state and expired boolean as the value
    sessions: DashMap<SessionToken, (SessionState<T>, bool)>,
}

#[cfg(any(test, feature = "mock_time"))]
pub static MEMORY_SESSION_STORE_NOW: once_cell::sync::Lazy<parking_lot::RwLock<Option<DateTime<Utc>>>> =
    once_cell::sync::Lazy::new(|| None.into());

impl<T> MemorySessionStore<T> {
    pub fn new() -> Self {
        Self {
            sessions: DashMap::new(),
        }
    }

    fn now() -> DateTime<Utc> {
        #[cfg(not(any(test, feature = "mock_time")))]
        return Utc::now();

        #[cfg(any(test, feature = "mock_time"))]
        MEMORY_SESSION_STORE_NOW.read().unwrap_or_else(Utc::now)
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

/// The cleanup task that removes stale sessions runs every so often.
pub const CLEANUP_INTERVAL_SECONDS: u64 = 120;

/// After this amount of inactivity, an active session should be expired.
pub const SESSION_EXPIRY_MINUTES: u32 = 30;

/// After this amount of time, a successfully completed session should be removed.
pub const SUCCESSFUL_SESSION_DELETION_MINUTES: u32 = 5;

/// After this amount of time, a failed or expired session should be removed.
pub const FAILED_SESSION_DELETION_MINUTES: u32 = 4 * 60;

impl<T> SessionStore<T> for MemorySessionStore<T>
where
    T: HasProgress + Clone + Send + Sync,
{
    async fn get(&self, token: &SessionToken) -> Result<Option<SessionState<T>>, SessionStoreError> {
        self.sessions
            .get(token)
            .map(|session_and_expired| {
                let (session, expired) = session_and_expired.deref();

                if *expired {
                    return Err(SessionStoreError::Expired(token.clone()));
                }

                Ok(session.clone())
            })
            .transpose()
    }

    async fn write(&self, session: SessionState<T>, is_new: bool) -> Result<(), SessionStoreError> {
        // Get a mutable reference, so that we can both check token presence
        // and replace the session while maintaining a lock on the DashMap.
        let existing_session = self.sessions.get_mut(&session.token);

        if let Some(mut existing_session) = existing_session {
            if is_new {
                return Err(SessionStoreError::DuplicateToken(session.token));
            }

            *existing_session = (session, false);
        } else {
            self.sessions.insert(session.token.clone(), (session, false));
        }

        Ok(())
    }

    async fn cleanup(&self) -> Result<(), SessionStoreError> {
        let now = Self::now();
        let succeeded_cutoff = now - chrono::Duration::minutes(SUCCESSFUL_SESSION_DELETION_MINUTES.into());
        let failed_cutoff = now - chrono::Duration::minutes(FAILED_SESSION_DELETION_MINUTES.into());
        let expiry_cutoff = now - chrono::Duration::minutes(SESSION_EXPIRY_MINUTES.into());

        self.sessions.retain(|_, session_and_expired| {
            let (session, expired) = session_and_expired.deref();

            match (session.data.progress(), *expired) {
                // Remove all succeeded sessions that are older than SUCCESSFUL_SESSION_DELETION_MINUTES.
                (Progress::Finished { has_succeeded }, false) if has_succeeded => {
                    session.last_active >= succeeded_cutoff
                }
                // Remove all failed and expired sessions that are older than FAILED_SESSION_DELETION_MINUTES.
                (Progress::Finished { .. }, false) | (_, true) => session.last_active >= failed_cutoff,
                _ => true,
            }
        });

        // For all active sessions that are older than SESSION_EXPIRY_MINUTES,
        // update the last active time and set them to expired.
        self.sessions.iter_mut().for_each(|mut session_and_expired| {
            let (session, expired) = session_and_expired.deref_mut();

            if !*expired && matches!(session.data.progress(), Progress::Active) && session.last_active < expiry_cutoff {
                session.last_active = now;
                *expired = true;
            }
        });

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
