pub mod store;

use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use base64::prelude::Engine;
use crypto::utils::random_bytes;
pub use store::MemoryParStore;
pub use store::PAR_TTL;
pub use store::ParStore;

/// Generates a `request_uri` for use as a PAR reference, as specified by
/// <https://datatracker.ietf.org/doc/html/rfc9126#section-2.2-3>:
/// `urn:ietf:params:oauth:request_uri:<random-data>` where `<random-data>` is
/// 32 bytes of random data base64url-encoded.
pub fn generate_request_uri() -> String {
    let random_data = BASE64_URL_SAFE_NO_PAD.encode(random_bytes(32));
    format!("urn:ietf:params:oauth:request_uri:{random_data}")
}
