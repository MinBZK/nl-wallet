use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
use derive_more::AsRef;
use derive_more::Display;
use derive_more::From;
use derive_more::Into;
use serde::Deserialize;
use serde::Serialize;
use tokio::task::JoinHandle;
use tokio::time;
use tokio::time::MissedTickBehavior;
use tracing::warn;

use wallet_common::generator::Generator;
use wallet_common::generator::TimeGenerator;
use wallet_common::jwt::JwtCredentialClaims;
use wallet_common::jwt::VerifiedJwt;
use wallet_common::utils::random_string;
use wallet_common::utils::sha256;
use wallet_common::wte::WteClaims;

/// The cleanup task that removes stale sessions runs every so often.
pub const CLEANUP_INTERVAL_SECONDS: Duration = Duration::from_secs(120);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Progress {
    Active,
    Finished { has_succeeded: bool },
}

pub trait HasProgress {
    fn progress(&self) -> Progress;
}

pub trait Expirable {
    fn is_expired(&self) -> bool;
    fn expire(&mut self);
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
    #[error("error while serializing: {0}")]
    Serialize(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("error while deserializing: {0}")]
    Deserialize(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("something went wrong: {0}")]
    Other(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
}

#[trait_variant::make(Send)]
pub trait SessionStore<T>
where
    T: HasProgress + Expirable,
{
    async fn get(&self, token: &SessionToken) -> Result<Option<SessionState<T>>, SessionStoreError>;
    async fn write(&self, session: SessionState<T>, is_new: bool) -> Result<(), SessionStoreError>;
    async fn cleanup(&self) -> Result<(), SessionStoreError>;

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

/// Different timeout values that should be used by the [`SessionStore::cleanup()`] implementation.
#[derive(Debug, Clone, Copy)]
pub struct SessionStoreTimeouts {
    /// After this amount of inactivity, an active session should be expired.
    pub expiration: Duration,
    /// After this amount of time, a successfully completed session should be removed.
    pub successful_deletion: Duration,
    /// After this amount of time, a failed or expired session should be removed.
    pub failed_deletion: Duration,
}

#[derive(Debug)]
pub struct MemorySessionStore<T, G = TimeGenerator> {
    pub timeouts: SessionStoreTimeouts,
    time: G,
    // Store the session state and expired boolean as the value
    sessions: DashMap<SessionToken, SessionState<T>>,
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

impl Default for SessionStoreTimeouts {
    fn default() -> Self {
        Self {
            expiration: Duration::from_secs(30 * 60),
            successful_deletion: Duration::from_secs(5 * 60),
            failed_deletion: Duration::from_secs(4 * 60 * 60),
        }
    }
}

impl<T, G> MemorySessionStore<T, G> {
    pub fn new_with_time(timeouts: SessionStoreTimeouts, time: G) -> Self {
        MemorySessionStore {
            timeouts,
            time,
            sessions: DashMap::new(),
        }
    }
}

impl<T> MemorySessionStore<T> {
    pub fn new(timeouts: SessionStoreTimeouts) -> Self {
        Self::new_with_time(timeouts, TimeGenerator)
    }
}

impl<T> Default for MemorySessionStore<T, TimeGenerator> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T, G> SessionStore<T> for MemorySessionStore<T, G>
where
    T: HasProgress + Expirable + Clone + Send + Sync,
    G: Generator<DateTime<Utc>> + Send + Sync,
{
    async fn get(&self, token: &SessionToken) -> Result<Option<SessionState<T>>, SessionStoreError> {
        let session = self.sessions.get(token).map(|session| session.clone());

        Ok(session)
    }

    async fn write(&self, session: SessionState<T>, is_new: bool) -> Result<(), SessionStoreError> {
        // Get an `Entry` from the `HashMap`, so that we obtain a lock for that `SessionToken`.
        // This prevents a race condition between checking if the session is present and inserting it,
        // since we do not have a mutable reference on the `DashMap`.
        let entry = self.sessions.entry(session.token.clone());

        if matches!(entry, Entry::Occupied(_)) && is_new {
            return Err(SessionStoreError::DuplicateToken(session.token));
        }

        entry.insert(session);

        Ok(())
    }

    async fn cleanup(&self) -> Result<(), SessionStoreError> {
        let now = self.time.generate();
        let succeeded_cutoff = now - self.timeouts.successful_deletion;
        let failed_cutoff = now - self.timeouts.failed_deletion;
        let expiry_cutoff = now - self.timeouts.expiration;

        self.sessions.retain(|_, session| {
            match (session.data.progress(), session.data.is_expired()) {
                // Remove all succeeded sessions that are older than the "successful_deletion" timeout.
                (Progress::Finished { has_succeeded }, false) if has_succeeded => {
                    session.last_active >= succeeded_cutoff
                }
                // Remove all failed and expired sessions that are older than the "failed_deletion" timeout.
                (Progress::Finished { .. }, false) | (_, true) => session.last_active >= failed_cutoff,
                _ => true,
            }
        });

        // For all active sessions that are older than the "expiration" timeout,
        // update the last active time and set them to expired.
        self.sessions.iter_mut().for_each(|mut session| {
            if !session.data.is_expired()
                && matches!(session.data.progress(), Progress::Active)
                && session.last_active < expiry_cutoff
            {
                session.last_active = now;
                session.data.expire();
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, AsRef, From, Into, Display, Serialize, Deserialize)]
#[cfg_attr(test, from(String, &'static str))]
pub struct SessionToken(String);

impl SessionToken {
    pub fn new_random() -> Self {
        random_string(32).into()
    }
}

/// Allows detection of previously used WTEs, by keeping track of WTEs that have been used by wallets within
/// their validity time window (after which they may be cleaned up with `cleanup()`).
#[trait_variant::make(Send)]
pub trait WteTracker {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Return whether or not we have seen this WTE within its validity window, and track this WTE as seen.
    async fn track_wte(&self, wte: &VerifiedJwt<JwtCredentialClaims<WteClaims>>) -> Result<bool, Self::Error>;

    /// Cleanup expired WTEs from this tracker.
    async fn cleanup(&self) -> Result<(), Self::Error>;

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

#[derive(Debug, Clone, Default)]
pub struct MemoryWteTracker<G = TimeGenerator> {
    seen_wtes: DashMap<Vec<u8>, DateTime<Utc>>,
    time: G,
}

impl<G> MemoryWteTracker<G> {
    pub fn new_with_time(time_generator: G) -> Self {
        Self {
            seen_wtes: DashMap::new(),
            time: time_generator,
        }
    }
}

impl MemoryWteTracker {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<G> WteTracker for MemoryWteTracker<G>
where
    G: Generator<DateTime<Utc>> + Send + Sync,
{
    type Error = Infallible;

    async fn track_wte(&self, wte: &VerifiedJwt<JwtCredentialClaims<WteClaims>>) -> Result<bool, Self::Error> {
        let shasum = sha256(wte.jwt().0.as_bytes());

        // We don't have to check for expiry of the WTE, because its type guarantees that it has already been verified.
        if self.seen_wtes.contains_key(&shasum) {
            Ok(true)
        } else {
            self.seen_wtes.insert(shasum, wte.payload().contents.attributes.exp);
            Ok(false)
        }
    }

    async fn cleanup(&self) -> Result<(), Self::Error> {
        let now = self.time.generate();
        self.seen_wtes.retain(|_, exp| *exp > now);

        Ok(())
    }
}

#[cfg(any(test, feature = "test"))]
pub mod test {
    use std::fmt::Debug;

    use assert_matches::assert_matches;
    use p256::ecdsa::SigningKey;
    use parking_lot::RwLock;
    use rand_core::OsRng;

    use wallet_common::keys::mock_remote::MockRemoteKeyFactory;
    use wallet_common::wte::WTE_EXPIRY;

    use crate::issuance_session::mock_wte;
    use crate::issuer::WTE_JWT_VALIDATIONS;

    use super::*;

    // Helper trait that signifies that a type has a constructor that generates random data.
    pub trait RandomData {
        fn new_random() -> Self;
    }

    /// Test reading and writing to a `SessionStore` implementation.
    pub async fn test_session_store_get_write<T>(session_store: &impl SessionStore<T>)
    where
        T: Debug + Clone + HasProgress + Expirable + RandomData + Eq,
    {
        // Generate a new random token and session state.
        let token = SessionToken::new_random();
        let session = SessionState::new(token.clone(), T::new_random());

        // The token should not be present in the store.
        assert!(session_store.get(&token).await.expect("should succeed").is_none());

        // Writing the session state to the store and retrieving it should succeed.
        session_store
            .write(session.clone(), true)
            .await
            .expect("should succeed");

        let session_read = session_store
            .get(&token)
            .await
            .expect("should succeed")
            .expect("should return session");

        assert_eq!(session_read.data, session.data);
        assert_eq!(session_read.token, token);
        // The maximum precision for PostgreSQL is 1 microsecond.
        assert_eq!(
            session_read.last_active.timestamp_micros(),
            session.last_active.timestamp_micros()
        );

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

        let session_read = session_store
            .get(&token)
            .await
            .expect("should succeed")
            .expect("should return session");

        assert_ne!(session_read.data, session.data);
        assert_eq!(session_read.data, updated_session.data);
        assert_eq!(session_read.token, token);
        // The maximum precision for PostgreSQL is 1 microsecond.
        assert_ne!(
            session_read.last_active.timestamp_micros(),
            session.last_active.timestamp_micros()
        );
        assert_eq!(
            session_read.last_active.timestamp_micros(),
            updated_session.last_active.timestamp_micros()
        );
    }

    pub async fn test_session_store_cleanup<T>(
        session_store: &impl SessionStore<T>,
        mock_time: &RwLock<DateTime<Utc>>,
        token: SessionToken,
        session_progress: Progress,
        max_time: Duration,
    ) -> Result<Option<SessionState<T>>, SessionStoreError>
    where
        T: HasProgress + Expirable + From<Progress>,
    {
        // Get the mock time.
        let t1 = *mock_time.read();

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
        *mock_time.write() = t2;

        session_store.cleanup().await.unwrap();

        let _ = session_store.get(&token).await.expect("should succeed");

        // Advance the time to just after the requested threshold and return the result of retrieving the session state.
        let t3 = t2 + chrono::Duration::milliseconds(1);
        *mock_time.write() = t3;

        session_store.cleanup().await.unwrap();

        session_store.get(&token).await
    }

    pub async fn test_session_store_cleanup_expiration<T>(
        session_store: &impl SessionStore<T>,
        timeouts: &SessionStoreTimeouts,
        mock_time: &RwLock<DateTime<Utc>>,
    ) where
        T: HasProgress + Expirable + From<Progress>,
    {
        // Retrieving active session state after the "expiration" timeout should result in an expired error.
        let token = SessionToken::new_random();
        let session = test_session_store_cleanup(
            session_store,
            mock_time,
            token.clone(),
            Progress::Active,
            timeouts.expiration,
        )
        .await
        .expect("should succeed")
        .expect("should return session");

        assert!(session.data.is_expired());

        // Advance the time right up to the "failed_deletion" timeout, the session state should still be expired.
        let t4 = *mock_time.read() + timeouts.failed_deletion;
        *mock_time.write() = t4;

        session_store.cleanup().await.unwrap();

        let session = session_store
            .get(&token)
            .await
            .expect("should succeed")
            .expect("should return session");

        assert!(session.data.is_expired());

        // Advance the time just after the "failed_deletion" timeout and the session should no longer be present.
        let t5 = t4 + chrono::Duration::milliseconds(1);
        *mock_time.write() = t5;

        session_store.cleanup().await.unwrap();

        let session = session_store.get(&token).await.expect("should succeed");

        assert!(session.is_none());
    }

    pub async fn test_session_store_cleanup_successful_deletion<T>(
        session_store: &impl SessionStore<T>,
        timeouts: &SessionStoreTimeouts,
        mock_time: &RwLock<DateTime<Utc>>,
    ) where
        T: HasProgress + Expirable + From<Progress>,
    {
        // Retrieving succeeded session state after the "successful_deletion"
        // timeout should result in the state no longer being present.
        let token = SessionToken::new_random();
        let session = test_session_store_cleanup(
            session_store,
            mock_time,
            token.clone(),
            Progress::Finished { has_succeeded: true },
            timeouts.successful_deletion,
        )
        .await
        .expect("should succeed");

        assert!(session.is_none());
    }

    pub async fn test_session_store_cleanup_failed_deletion<T>(
        session_store: &impl SessionStore<T>,
        timeouts: &SessionStoreTimeouts,
        mock_time: &RwLock<DateTime<Utc>>,
    ) where
        T: Expirable + HasProgress + From<Progress>,
    {
        // Retrieving failed session state after the "failed_deletion"
        // timeout should result in the state no longer being present.
        let token = SessionToken::new_random();
        let session = test_session_store_cleanup(
            session_store,
            mock_time,
            token.clone(),
            Progress::Finished { has_succeeded: false },
            timeouts.failed_deletion,
        )
        .await
        .expect("should succeed");

        assert!(session.is_none());
    }

    pub async fn test_wte_tracker(wte_tracker: &impl WteTracker, mock_time: &RwLock<DateTime<Utc>>) {
        let key_factory = MockRemoteKeyFactory::default();
        let wte_signing_key = SigningKey::random(&mut OsRng);

        let wte = mock_wte(&key_factory, &wte_signing_key).await.jwt;

        let wte = VerifiedJwt::try_new(wte, &wte_signing_key.verifying_key().into(), &WTE_JWT_VALIDATIONS).unwrap();

        // Checking our WTE for the first time means we haven't seen it before
        assert!(!wte_tracker.track_wte(&wte).await.unwrap());

        // Now we have seen it
        assert!(wte_tracker.track_wte(&wte).await.unwrap());

        // Advance time past the expiry of the WTE and run the cleanup job
        let t2 = *mock_time.read() + WTE_EXPIRY * 2;
        *mock_time.write() = t2;
        wte_tracker.cleanup().await.unwrap();

        // The expired WTE has been removed by the cleanup job
        assert!(!wte_tracker.track_wte(&wte).await.unwrap());
    }
}

#[cfg(test)]
mod tests {
    use parking_lot::RwLock;

    use wallet_common::generator::mock::MockTimeGenerator;
    use wallet_common::utils;

    use self::test::RandomData;

    use super::*;

    /// A mock data type that adheres to all the trait bounds necessary for testing.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct MockSessionData {
        progress: Progress,
        is_expired: bool,
        data: Vec<u8>,
    }

    impl MockSessionData {
        fn new(progress: Progress) -> Self {
            Self {
                progress,
                is_expired: false,
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

    impl Expirable for MockSessionData {
        fn is_expired(&self) -> bool {
            self.is_expired
        }

        fn expire(&mut self) {
            self.is_expired = true;
        }
    }

    impl RandomData for MockSessionData {
        fn new_random() -> Self {
            Self::new(Progress::Active)
        }
    }

    #[tokio::test]
    async fn test_memory_session_store_get_write() {
        let session_store = MemorySessionStore::<MockSessionData, _>::default();
        test::test_session_store_get_write(&session_store).await;
    }

    fn memory_session_store_with_mock_time() -> (
        MemorySessionStore<MockSessionData, MockTimeGenerator>,
        Arc<RwLock<DateTime<Utc>>>,
    ) {
        let time_generator = MockTimeGenerator::default();
        let mock_time = Arc::clone(&time_generator.time);
        let session_store = MemorySessionStore::new_with_time(Default::default(), time_generator);

        (session_store, mock_time)
    }

    #[tokio::test]
    async fn test_memory_session_store_cleanup_expiration() {
        let (session_store, mock_time) = memory_session_store_with_mock_time();

        test::test_session_store_cleanup_expiration(&session_store, &session_store.timeouts, mock_time.as_ref()).await;
    }

    #[tokio::test]
    async fn test_memory_session_store_cleanup_successful_deletion() {
        let (session_store, mock_time) = memory_session_store_with_mock_time();

        test::test_session_store_cleanup_successful_deletion(
            &session_store,
            &session_store.timeouts,
            mock_time.as_ref(),
        )
        .await;
    }

    #[tokio::test]
    async fn test_memory_session_store_cleanup_failed_deletion() {
        let (session_store, mock_time) = memory_session_store_with_mock_time();

        test::test_session_store_cleanup_failed_deletion(&session_store, &session_store.timeouts, mock_time.as_ref())
            .await;
    }

    #[tokio::test]
    async fn test_memory_wte_tracker() {
        let time_generator = MockTimeGenerator::default();
        let mock_time = Arc::clone(&time_generator.time);

        test::test_wte_tracker(&MemoryWteTracker::new_with_time(time_generator), mock_time.as_ref()).await;
    }
}
