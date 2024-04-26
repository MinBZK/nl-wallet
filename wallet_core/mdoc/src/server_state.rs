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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    #[error("session with token {0} not found")]
    NotFound(SessionToken),
    #[error("session with token {0} is expired")]
    Expired(SessionToken),
    #[error("token {0} already exists")]
    DuplicateToken(SessionToken),
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
    fn get(&self, token: &SessionToken) -> impl Future<Output = Result<SessionState<T>, SessionStoreError>> + Send;
    fn write(
        &self,
        session: SessionState<T>,
        is_new: bool,
    ) -> impl Future<Output = Result<(), SessionStoreError>> + Send;
    fn cleanup(&self) -> impl Future<Output = Result<(), SessionStoreError>> + Send
    where
        T: HasProgress;

    fn start_cleanup_task(self: Arc<Self>, interval: Duration) -> JoinHandle<()>
    where
        T: HasProgress,
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

impl<T> SessionState<T> {
    pub fn new(token: SessionToken, data: T) -> SessionState<T> {
        SessionState {
            data,
            token,
            last_active: Utc::now(),
        }
    }
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
    async fn get(&self, token: &SessionToken) -> Result<SessionState<T>, SessionStoreError> {
        self.sessions
            .get(token)
            .ok_or_else(|| SessionStoreError::NotFound(token.clone()))
            .and_then(|session_and_expired| {
                let (session, expired) = session_and_expired.deref();

                if *expired {
                    return Err(SessionStoreError::Expired(token.clone()));
                }

                Ok(session.clone())
            })
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

#[cfg(any(test, feature = "test"))]
pub mod test {
    use std::fmt::Debug;

    use assert_matches::assert_matches;
    use parking_lot::RwLock;

    use super::*;

    // Helper trait that signifies that a type has a constructor that generates random data.
    pub trait RandomData {
        fn new_random() -> Self;
    }

    /// Test reading and writing to a `SessionStore` implementation.
    pub async fn test_session_store_get_write<T>(session_store: &impl SessionStore<T>)
    where
        T: Debug + Clone + RandomData + Eq,
    {
        // Generate a new random token and session state.
        let token = SessionToken::new_random();
        let session = SessionState::new(token.clone(), T::new_random());

        // The token should not be present in the store.
        assert_matches!(
            session_store.get(&token).await.expect_err("should return error"),
            SessionStoreError::NotFound(expired_token) if expired_token == token
        );

        // Writing the session state to the store and retrieving it should succeed.
        session_store
            .write(session.clone(), true)
            .await
            .expect("should succeed");

        let session_read = session_store.get(&token).await.expect("should succeed");

        assert_eq!(session_read.data, session.data);
        assert_eq!(session_read.token, token);
        assert_eq!(session_read.last_active, session.last_active);

        // Generate new session state for the same token.
        let updated_session = SessionState {
            data: T::new_random(),
            token: token.clone(),
            last_active: session.last_active + Duration::from_secs(1),
        };

        // Writing this as new data should return an error that indicates data for this token already exists.
        assert_matches!(
            session_store.write(updated_session.clone(), true).await.expect_err("should return error"),
            SessionStoreError::DuplicateToken(duplicate_token) if duplicate_token == token
        );

        // Writing this data as an update should succeed.
        session_store
            .write(updated_session.clone(), false)
            .await
            .expect("should succeed");

        let session_read = session_store.get(&token).await.expect("should succeed");

        assert_ne!(session_read.data, session.data);
        assert_eq!(session_read.data, updated_session.data);
        assert_eq!(session_read.token, token);
        assert_ne!(session_read.last_active, session.last_active);
        assert_eq!(session_read.last_active, updated_session.last_active);
    }

    pub async fn test_session_store_cleanup<T>(
        session_store: &impl SessionStore<T>,
        mock_now: &RwLock<Option<DateTime<Utc>>>,
        token: SessionToken,
        session_progress: Progress,
        max_time: chrono::Duration,
    ) -> Result<SessionState<T>, SessionStoreError>
    where
        T: HasProgress + From<Progress>,
    {
        // Mock the time.
        let t1 = Utc::now();
        mock_now.write().replace(t1);

        // Create new session state, this should be present in the store after cleanup, as the time has not advanced.
        let session = SessionState {
            data: T::from(session_progress),
            token: token.clone(),
            last_active: t1,
        };

        session_store.write(session, true).await.unwrap();
        session_store.cleanup().await.unwrap();

        let _ = session_store.get(&token).await.expect("should succeed");

        // Advance the time right up to the requested threshold, the session state should still be retrievable.
        let t2 = t1 + max_time;
        mock_now.write().replace(t2);

        session_store.cleanup().await.unwrap();

        let _ = session_store.get(&token).await.expect("should succeed");

        // Advance the time to just after the requested threshold and return the result of retrieving the session state.
        let t3 = t2 + chrono::Duration::milliseconds(1);
        mock_now.write().replace(t3);

        session_store.cleanup().await.unwrap();

        session_store.get(&token).await
    }

    pub async fn test_session_store_cleanup_expiration<T>(
        session_store: &impl SessionStore<T>,
        mock_now: &RwLock<Option<DateTime<Utc>>>,
    ) where
        T: Debug + HasProgress + From<Progress>,
    {
        // Retrieving active session state after SESSION_EXPIRY_MINUTES should result in an expired error.
        let token = SessionToken::new_random();
        let error = test_session_store_cleanup(
            session_store,
            mock_now,
            token.clone(),
            Progress::Active,
            chrono::Duration::minutes(SESSION_EXPIRY_MINUTES.into()),
        )
        .await
        .expect_err("should return error");

        assert_matches!(
            error,
            SessionStoreError::Expired(expired_token) if expired_token == token
        );

        // Advance the time right up to FAILED_SESSION_DELETION_MINUTES, the session state should still be expired.
        let t4 = mock_now.read().unwrap() + chrono::Duration::minutes(FAILED_SESSION_DELETION_MINUTES.into());
        mock_now.write().replace(t4);

        session_store.cleanup().await.unwrap();

        assert_matches!(
            session_store.get(&token).await.expect_err("should return error"),
            SessionStoreError::Expired(expired_token) if expired_token == token
        );

        // Advance the time just after FAILED_SESSION_DELETION_MINUTES and the session should no longer be present.
        let t5 = t4 + chrono::Duration::milliseconds(1);
        mock_now.write().replace(t5);

        session_store.cleanup().await.unwrap();

        assert_matches!(
            session_store.get(&token).await.expect_err("should return error"),
            SessionStoreError::NotFound(expired_token) if expired_token == token
        );
    }

    pub async fn test_session_store_cleanup_successful_deletion<T>(
        session_store: &impl SessionStore<T>,
        mock_now: &RwLock<Option<DateTime<Utc>>>,
    ) where
        T: Debug + HasProgress + From<Progress>,
    {
        // Retrieving succeeded session state after SUCCESSFUL_SESSION_DELETION_MINUTES
        // should result in the state no longer being present.
        let token = SessionToken::new_random();
        let error = test_session_store_cleanup(
            session_store,
            mock_now,
            token.clone(),
            Progress::Finished { has_succeeded: true },
            chrono::Duration::minutes(SUCCESSFUL_SESSION_DELETION_MINUTES.into()),
        )
        .await
        .expect_err("should return error");

        assert_matches!(
            error,
            SessionStoreError::NotFound(expired_token) if expired_token == token
        );
    }

    pub async fn test_session_store_cleanup_failed_deletion<T>(
        session_store: &impl SessionStore<T>,
        mock_now: &RwLock<Option<DateTime<Utc>>>,
    ) where
        T: Debug + HasProgress + From<Progress>,
    {
        // Retrieving failed session state after FAILED_SESSION_DELETION_MINUTES
        // should result in the state no longer being present.
        let token = SessionToken::new_random();
        let error = test_session_store_cleanup(
            session_store,
            mock_now,
            token.clone(),
            Progress::Finished { has_succeeded: false },
            chrono::Duration::minutes(FAILED_SESSION_DELETION_MINUTES.into()),
        )
        .await
        .expect_err("should return error");

        assert_matches!(
            error,
            SessionStoreError::NotFound(expired_token) if expired_token == token
        );
    }
}

#[cfg(test)]
mod tests {
    use once_cell::sync::Lazy;

    use wallet_common::utils;

    use self::test::RandomData;

    use super::*;

    /// A mock data type that adheres to all the trait bounds necessary for testing.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct MockSessionData {
        progress: Progress,
        data: Vec<u8>,
    }

    impl MockSessionData {
        fn new(progress: Progress) -> Self {
            Self {
                progress,
                data: utils::random_bytes(32),
            }
        }
    }

    impl From<Progress> for MockSessionData {
        fn from(value: Progress) -> Self {
            Self::new(value)
        }
    }

    impl HasProgress for MockSessionData {
        fn progress(&self) -> Progress {
            self.progress
        }
    }

    impl RandomData for MockSessionData {
        fn new_random() -> Self {
            Self::new(Progress::Active)
        }
    }

    #[tokio::test]
    async fn test_memory_session_get_write() {
        let session_store = MemorySessionStore::<MockSessionData>::new();
        test::test_session_store_get_write(&session_store).await;
    }

    #[tokio::test]
    async fn test_memory_session_store_cleanup() {
        let session_store = MemorySessionStore::<MockSessionData>::new();
        let mock_now = Lazy::force(&MEMORY_SESSION_STORE_NOW);

        test::test_session_store_cleanup_expiration(&session_store, mock_now).await;
        test::test_session_store_cleanup_successful_deletion(&session_store, mock_now).await;
        test::test_session_store_cleanup_failed_deletion(&session_store, mock_now).await;
    }
}
