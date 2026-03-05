use std::io;
use std::sync::Arc;

use axum_server::tls_rustls::RustlsConfig;
use rustls::ServerConfig;
use rustls::server::NoServerSessionStorage;
use rustls_pki_types::CertificateDer;
use rustls_pki_types::PrivateKeyDer;
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
    /// Convert this instance into a [`RustlsConfig`].
    /// Based on [`RustlsConfig::from_der`]
    pub fn into_rustls_config(self) -> Result<RustlsConfig, io::Error> {
        let cert = vec![CertificateDer::from(self.cert)];
        let key = PrivateKeyDer::try_from(self.key).map_err(io::Error::other)?;

        let mut config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert, key)
            .map_err(io::Error::other)?;

        config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        // Disable TLS 1.2 session IDs as mandated by the NCSC TLS policy (session tickets are off by default in rustls)
        // See https://www.ncsc.nl/transport-layer-security-tls/richtlijnen2025-05
        config.session_storage = Arc::new(NoServerSessionStorage {});

        Ok(RustlsConfig::from_config(Arc::new(config)))
    }
}
