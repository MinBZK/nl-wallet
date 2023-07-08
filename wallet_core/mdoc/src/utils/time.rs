#[cfg(not(any(test, feature = "mock")))]
pub fn now() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
}

#[cfg(any(test, feature = "mock"))]
pub use mock_time::now;
#[cfg(any(test, feature = "mock"))]
pub mod mock_time {
    use std::sync::RwLock;

    use chrono::{DateTime, Utc};

    static MOCK_TIME: RwLock<Option<DateTime<Utc>>> = RwLock::new(None);

    pub fn now() -> DateTime<Utc> {
        MOCK_TIME.read().unwrap().unwrap_or_else(Utc::now)
    }

    pub fn set_mock_time(time: DateTime<Utc>) {
        *MOCK_TIME.write().unwrap() = Some(time);
    }

    pub fn clear_mock_time() {
        *MOCK_TIME.write().unwrap() = None;
    }
}
