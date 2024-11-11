use chrono::{serde::ts_seconds, DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WteClaims {
    #[serde(with = "ts_seconds")]
    pub exp: DateTime<Utc>,
}

pub static WTE_EXPIRY: Duration = Duration::minutes(5);

impl WteClaims {
    pub fn new() -> Self {
        Self {
            exp: Utc::now() + WTE_EXPIRY,
        }
    }
}

impl Default for WteClaims {
    fn default() -> Self {
        Self::new()
    }
}
