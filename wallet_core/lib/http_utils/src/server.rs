//! This module defines TLS configuration, and a conversion function into [`RustlsConfig`],
//! that complies with the NCSC TLS policy: https://www.ncsc.nl/transport-layer-security-tls/richtlijnen2025-05
//!
//! # Section 3.3: safe TLS parameters
//!
//! Rustls enables by default only TLS 1.2 and 1.3.
//! Ciphers enabled by default by rustls for TLS 1.2:
//! - TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384
//! - TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256
//! - TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256
//! - TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384
//! - TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256
//! - TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256
//!
//! Ciphers enabled by default by rustls for TLS 1.3:
//! - Cipher suites:
//!     - TLS13_AES_256_GCM_SHA384
//!     - TLS13_AES_128_GCM_SHA256
//!     - TLS13_CHACHA20_POLY1305_SHA256
//! - Key exchange algorithms:
//!     - X25519:    ECDH over Curve25519
//!     - SECP256R1: ECDH over P-256
//!     - SECP384R1: ECDH over P-384
//!
//! All of this complies with what is allowed by the NCSC TLS policy section 3.3.
//!
//! # Section 3.4: TLS features
//!
//! | Feature                    | NCSC policy   | Implementation               |
//! |----------------------------|---------------|------------------------------|
//! | TLS compression            | not allowed   | not implemented by rustls    |
//! | Session tickets in TLS 1.2 | not allowed   | off by default in rustls     |
//! | Session IDs in TLS 1.2     | not allowed   | disabled below               |
//! | Renegotiation              | best disabled | not implemented by rustls    |
//! | 0-RTT in TLS 1.3           | not allowed   | off by default in rustls     |

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
    /// Based on [`RustlsConfig::from_der`], with disabling of session IDs added to it.
    pub fn into_rustls_config(self) -> Result<RustlsConfig, io::Error> {
        let cert = vec![CertificateDer::from(self.cert)];
        let key = PrivateKeyDer::try_from(self.key).map_err(io::Error::other)?;

        let mut config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert, key)
            .map_err(io::Error::other)?;

        config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        // Disable TLS 1.2 session IDs.
        config.session_storage = Arc::new(NoServerSessionStorage {});

        Ok(RustlsConfig::from_config(Arc::new(config)))
    }
}
