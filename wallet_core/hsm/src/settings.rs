use std::path::PathBuf;
use std::time::Duration;

use serde::Deserialize;
use serde_with::serde_as;
use serde_with::DurationSeconds;

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct Hsm {
    pub library_path: PathBuf,
    pub user_pin: String,
    pub max_sessions: u8,

    #[serde(rename = "max_session_lifetime_in_sec")]
    #[serde_as(as = "DurationSeconds")]
    pub max_session_lifetime: Duration,
}
