use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
#[cfg(feature = "axum")]
use std::io;
use std::path::Path;

#[cfg(feature = "axum")]
use axum_server::tls_rustls::RustlsConfig;
use derive_more::Debug;
use http::Method;
use reqwest::Client;
use serde::Deserialize;
use serde::Serialize;
use serde_with::base64::Base64;
use serde_with::serde_as;

use crate::reqwest::tls_pinned_client_builder;
use crate::reqwest::ClientBuilder;
use crate::reqwest::JsonClientBuilder;
use crate::reqwest::JsonReqwestBuilder;
use crate::reqwest::RequestBuilder;
use crate::reqwest::ReqwestBuilder;
use crate::reqwest::ReqwestTrustAnchor;
use crate::urls::BaseUrl;

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct TlsServerConfig {
    #[serde_as(as = "Base64")]
    pub cert: Vec<u8>,
    #[serde_as(as = "Base64")]
    pub key: Vec<u8>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TlsPinningConfigHash(u64);

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct TlsPinningConfig {
    pub base_url: BaseUrl,
    #[debug(skip)]
    #[serde_as(as = "Vec<Base64>")]
    pub trust_anchors: Vec<ReqwestTrustAnchor>,
}

impl TlsPinningConfig {
    pub fn to_hash(&self) -> TlsPinningConfigHash {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        TlsPinningConfigHash(hasher.finish())
    }
}

impl ClientBuilder for TlsPinningConfig {
    fn builder(&self) -> reqwest::ClientBuilder {
        tls_pinned_client_builder(self.certificates())
    }
}

impl JsonClientBuilder for TlsPinningConfig {}

impl RequestBuilder for TlsPinningConfig {
    fn request(&self, method: Method, path: impl AsRef<Path>) -> (reqwest::Client, reqwest::RequestBuilder) {
        let client = self.client();
        let request = self.request_with_client(&client, method, &path);
        (client, request)
    }

    fn request_with_client(&self, client: &Client, method: Method, path: impl AsRef<Path>) -> reqwest::RequestBuilder {
        client.request(method, self.base_url.join(&path.as_ref().to_string_lossy()))
    }
}

impl ReqwestBuilder for TlsPinningConfig {}

impl JsonReqwestBuilder for TlsPinningConfig {}

impl TlsPinningConfig {
    fn client(&self) -> reqwest::Client {
        self.builder()
            .build()
            .expect("should be able to build reqwest HTTP client")
    }

    pub fn certificates(&self) -> Vec<reqwest::Certificate> {
        self.trust_anchors
            .iter()
            .map(|anchor| anchor.as_certificate().clone())
            .collect()
    }
}

#[cfg(feature = "axum")]
impl TlsServerConfig {
    pub async fn to_rustls_config(&self) -> Result<RustlsConfig, io::Error> {
        RustlsConfig::from_der(vec![self.cert.to_vec()], self.key.to_vec()).await
    }
}

#[cfg(feature = "insecure_http_client")]
pub mod test {
    use std::path::Path;

    use http::Method;
    use reqwest::Client;

    use crate::reqwest::ClientBuilder;
    use crate::reqwest::JsonClientBuilder;
    use crate::reqwest::JsonReqwestBuilder;
    use crate::reqwest::RequestBuilder;
    use crate::reqwest::ReqwestBuilder;
    use crate::urls::BaseUrl;

    pub struct HttpConfig {
        pub base_url: BaseUrl,
    }

    impl ClientBuilder for HttpConfig {
        fn builder(&self) -> reqwest::ClientBuilder {
            reqwest::ClientBuilder::new()
        }
    }

    impl JsonClientBuilder for HttpConfig {}

    impl RequestBuilder for HttpConfig {
        fn request(&self, method: Method, path: impl AsRef<Path>) -> (reqwest::Client, reqwest::RequestBuilder) {
            let client = self
                .builder()
                .build()
                .expect("should be able to build reqwest HTTP client");
            let request = self.request_with_client(&client, method, &path);
            (client, request)
        }

        fn request_with_client(
            &self,
            client: &Client,
            method: Method,
            path: impl AsRef<Path>,
        ) -> reqwest::RequestBuilder {
            client.request(method, self.base_url.join(&path.as_ref().to_string_lossy()))
        }
    }

    impl JsonReqwestBuilder for HttpConfig {}

    impl ReqwestBuilder for HttpConfig {}
}
