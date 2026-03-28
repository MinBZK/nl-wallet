use std::time::Duration;

use crypto::utils::random_string;

pub mod memory_store;
pub mod response;
pub mod store;

const C_NONCE_LENGTH: usize = 32;
pub const C_NONCE_VALIDITY: Duration = Duration::from_mins(5);

pub fn generate_nonce() -> String {
    random_string(C_NONCE_LENGTH)
}
