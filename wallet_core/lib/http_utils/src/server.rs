use std::io;

use axum_server::tls_rustls::RustlsConfig;
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

impl TlsServerConfig {
    pub async fn into_rustls_config(self) -> Result<RustlsConfig, io::Error> {
        RustlsConfig::from_der(vec![self.cert], self.key).await
    }
}
