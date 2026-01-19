use std::path::PathBuf;
use std::time::Duration;

use derive_more::Debug;
use serde::Deserialize;
use serde_with::DurationSeconds;
use serde_with::serde_as;

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Hsm {
    pub library_path: PathBuf,
    #[debug(skip)]
    pub user_pin: String,
    pub max_sessions: u8,

    #[serde(rename = "max_session_lifetime_in_sec")]
    #[serde_as(as = "DurationSeconds")]
    pub max_session_lifetime: Duration,
}
