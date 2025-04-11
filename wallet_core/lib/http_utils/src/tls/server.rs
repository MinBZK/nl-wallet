use std::io;

use axum_server::tls_rustls::RustlsConfig;

use super::TlsServerConfig;

impl TlsServerConfig {
    pub async fn to_rustls_config(&self) -> Result<RustlsConfig, io::Error> {
        RustlsConfig::from_der(vec![self.cert.to_vec()], self.key.to_vec()).await
    }
}
