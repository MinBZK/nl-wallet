#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "server")]
pub mod server;
#[cfg(feature = "insecure_http_client")]
pub mod test;

use serde::Deserialize;
use serde_with::base64::Base64;
use serde_with::serde_as;

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct TlsServerConfig {
    #[serde_as(as = "Base64")]
    pub cert: Vec<u8>,
    #[serde_as(as = "Base64")]
    pub key: Vec<u8>,
}
