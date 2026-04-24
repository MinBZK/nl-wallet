use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;

pub mod memory_store;
pub mod response;
pub mod store;

const C_NONCE_VALIDITY: Duration = Duration::from_mins(5);

pub fn earliest_nonce_validity_datetime(now: DateTime<Utc>) -> DateTime<Utc> {
    now - C_NONCE_VALIDITY
}

pub fn nonce_is_valid(created_date_time: DateTime<Utc>, now: DateTime<Utc>) -> bool {
    created_date_time >= earliest_nonce_validity_datetime(now)
}
