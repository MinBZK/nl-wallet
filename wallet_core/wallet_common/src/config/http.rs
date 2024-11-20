#[cfg(feature = "axum")]
use std::io;

#[cfg(feature = "axum")]
use axum_server::tls_rustls::RustlsConfig;
use derive_more::Debug;
use reqwest::ClientBuilder;
use serde::Deserialize;
use serde::Serialize;
use serde_with::base64::Base64;
use serde_with::serde_as;

use crate::reqwest::tls_pinned_client_builder;
use crate::reqwest::ReqwestClient;
use crate::trust_anchor::DerTrustAnchor;
use crate::urls::BaseUrl;

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct TlsServerConfig {
    #[serde_as(as = "Base64")]
    pub cert: Vec<u8>,
    #[serde_as(as = "Base64")]
    pub key: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct TlsPinningConfig {
    pub base_url: BaseUrl,
    #[debug(skip)]
    pub trust_anchors: Vec<DerTrustAnchor>,
}

impl ReqwestClient for TlsPinningConfig {
    fn base_url(&self) -> &BaseUrl {
        &self.base_url
    }

    fn client_builder(&self) -> ClientBuilder {
        tls_pinned_client_builder(self.certificates())
    }
}

impl TlsPinningConfig {
    pub fn certificates(&self) -> Vec<reqwest::Certificate> {
        self.trust_anchors
            .iter()
            .map(|anchor| {
                reqwest::Certificate::from_der(&anchor.der_bytes)
                    .expect("DerTrustAnchor should be able to be converted to reqwest::Certificate")
            })
            .collect()
    }
}

#[cfg(feature = "axum")]
impl TlsServerConfig {
    pub async fn to_rustls_config(&self) -> Result<RustlsConfig, io::Error> {
        RustlsConfig::from_der(vec![self.cert.to_vec()], self.key.to_vec()).await
    }
}

#[cfg(any(test, feature = "test"))]
pub mod test {
    use crate::reqwest::ReqwestClient;
    use crate::urls::BaseUrl;

    pub struct HttpConfig {
        pub base_url: BaseUrl,
    }

    impl ReqwestClient for HttpConfig {
        fn base_url(&self) -> &BaseUrl {
            &self.base_url
        }

        fn client_builder(&self) -> reqwest::ClientBuilder {
            reqwest::ClientBuilder::new()
        }
    }
}
