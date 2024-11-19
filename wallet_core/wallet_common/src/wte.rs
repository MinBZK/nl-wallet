use chrono::serde::ts_seconds;
use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

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
