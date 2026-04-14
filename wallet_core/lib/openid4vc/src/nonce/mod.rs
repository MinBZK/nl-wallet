use std::time::Duration;

pub mod memory_store;
pub mod response;
pub mod store;

pub const C_NONCE_VALIDITY: Duration = Duration::from_mins(5);
