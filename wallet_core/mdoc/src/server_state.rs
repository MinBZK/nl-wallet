use std::{sync::Arc, time::Duration};

use chrono::Local;
use dashmap::DashMap;
use tokio::{task::JoinHandle, time};

use crate::{
    issuer::SessionState,
    issuer_shared::{IssuanceError, SessionToken},
    Error, Result,
};

pub trait SessionStore {
    type Data: Clone;

    fn get(&self, id: &SessionToken) -> Result<Self::Data>;
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
            data,
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

    fn get(&self, token: &SessionToken) -> Result<SessionState<T>> {
        let data = self
            .sessions
            .get(token)
            .ok_or_else(|| Error::from(IssuanceError::UnknownSessionId(token.clone())))?
            .clone();
        Ok(data)
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
