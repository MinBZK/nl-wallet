use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use base64::prelude::Engine;
use chrono::Duration;
use crypto::utils::random_bytes;

/// TTL for PAR entries. Per [RFC 9126 §2.2], the `expires_in` value should be short.
///
/// [RFC 9126 §2.2]: https://www.rfc-editor.org/rfc/rfc9126#section-2.2
pub const PAR_TTL: Duration = Duration::seconds(60);

/// Generates a `request_uri` for use as a PAR reference, as specified by
/// <https://datatracker.ietf.org/doc/html/rfc9126#section-2.2-3>:
/// `urn:ietf:params:oauth:request_uri:<random-data>` where `<random-data>` is
/// 32 bytes of random data base64url-encoded.
pub fn generate_request_uri() -> String {
    let random_data = BASE64_URL_SAFE_NO_PAD.encode(random_bytes(32));
    format!("urn:ietf:params:oauth:request_uri:{random_data}")
}
