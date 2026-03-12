use std::hash::Hash;
use std::hash::Hasher;
use std::sync::LazyLock;
use std::time::Duration;

use derive_more::AsRef;
use http::header;
use http::HeaderMap;
use http::HeaderValue;
use http::Method;
use mime::Mime;
use mime::APPLICATION_JSON;
use reqwest::Certificate;
use reqwest::Client;
use reqwest::ClientBuilder;
use reqwest::RequestBuilder;
use reqwest::Response;
use url::Url;
use x509_parser::error::X509Error;
use x509_parser::prelude::FromDer;
use x509_parser::prelude::X509Certificate;

use crate::error::APPLICATION_PROBLEM_JSON;
use crate::urls::BaseUrl;

const CLIENT_REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
const CLIENT_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

/// Wrapper around a [`Certificate`] implementing `PartialEq`, `Eq` and `Hash`. In addition, it implements
/// the necessary `From`/`TryFrom` implementations so that it can be (de)serialised using `serde_with`.
#[derive(Clone, AsRef)]
pub struct ReqwestTrustAnchor {
    #[as_ref([u8])]
    der_bytes: Vec<u8>,
    certificate: Certificate,
}

impl ReqwestTrustAnchor {
    pub fn as_certificate(&self) -> &Certificate {
        &self.certificate
    }

    pub fn into_certificate(self) -> Certificate {
        self.certificate
    }
}

impl PartialEq for ReqwestTrustAnchor {
    fn eq(&self, other: &Self) -> bool {
        self.der_bytes == other.der_bytes
    }
}

impl Eq for ReqwestTrustAnchor {}

impl Hash for ReqwestTrustAnchor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.der_bytes.hash(state);
    }
}

impl TryFrom<Vec<u8>> for ReqwestTrustAnchor {
    type Error = ReqwestTrustAnchorError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        // Certificate::from_der does not parse the bytes when using `rustls`, so we explicitly parse it here to ensure
        // the bytes represent a valid X.509 certificate.
        let _ = X509Certificate::from_der(&value)?;
        let certificate = Certificate::from_der(&value)?;
        Ok(Self {
            der_bytes: value,
            certificate,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ReqwestTrustAnchorError {
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("certificate parsing error: {0}")]
    X509Parser(#[from] x509_parser::nom::Err<X509Error>),
}

pub trait IntoReqwestClient {
    fn try_into_custom_client<F>(self, builder_adapter: F) -> Result<ReqwestClient, reqwest::Error>
    where
        F: FnOnce(ClientBuilder) -> ClientBuilder;

    fn try_into_custom_json_client<F>(self, builder_adapter: F) -> Result<ReqwestClient, reqwest::Error>
    where
        F: FnOnce(ClientBuilder) -> ClientBuilder,
        Self: Sized,
    {
        self.try_into_custom_client(|builder| builder_adapter(client_builder_accept_json(builder)))
    }

    fn try_into_client(self) -> Result<ReqwestClient, reqwest::Error>
    where
        Self: Sized,
    {
        self.try_into_custom_client(std::convert::identity)
    }

    fn try_into_json_client(self) -> Result<ReqwestClient, reqwest::Error>
    where
        Self: Sized,
    {
        self.try_into_custom_json_client(std::convert::identity)
    }
}

#[derive(Debug, Clone)]
pub enum ReqwestClientUrl<'a> {
    Base,
    Absolute(Url),
    Relative(&'a str),
}

#[derive(Debug, Clone)]
pub struct ReqwestClient {
    client: Client,
    pub base_url: BaseUrl,
}

impl ReqwestClient {
    pub(crate) fn new(client: Client, base_url: BaseUrl) -> Self {
        Self { client, base_url }
    }

    async fn send_custom_request<F>(
        &self,
        method: Method,
        url: ReqwestClientUrl<'_>,
        request_adapter: F,
    ) -> Result<Response, reqwest::Error>
    where
        F: FnOnce(RequestBuilder) -> RequestBuilder,
    {
        let url = match url {
            ReqwestClientUrl::Base => self.base_url.as_ref().clone(),
            ReqwestClientUrl::Absolute(url) => url,
            ReqwestClientUrl::Relative(path) => self.base_url.clone().join(path),
        };

        let request = request_adapter(self.client.request(method, url)).build()?;
        let response = self.client.execute(request).await?;

        Ok(response)
    }

    async fn send_request(&self, method: Method, url: ReqwestClientUrl<'_>) -> Result<Response, reqwest::Error> {
        self.send_custom_request(method, url, std::convert::identity).await
    }

    pub async fn send_custom_get<F>(
        &self,
        url: ReqwestClientUrl<'_>,
        request_adapter: F,
    ) -> Result<Response, reqwest::Error>
    where
        F: FnOnce(RequestBuilder) -> RequestBuilder,
    {
        self.send_custom_request(Method::GET, url, request_adapter).await
    }

    pub async fn send_get(&self, url: ReqwestClientUrl<'_>) -> Result<Response, reqwest::Error> {
        self.send_request(Method::GET, url).await
    }

    pub async fn send_custom_post<F>(
        &self,
        url: ReqwestClientUrl<'_>,
        request_adapter: F,
    ) -> Result<Response, reqwest::Error>
    where
        F: FnOnce(RequestBuilder) -> RequestBuilder,
    {
        self.send_custom_request(Method::POST, url, request_adapter).await
    }

    pub async fn send_post(&self, url: ReqwestClientUrl<'_>) -> Result<Response, reqwest::Error> {
        self.send_request(Method::POST, url).await
    }
}

pub fn parse_content_type(response: &Response) -> Option<Mime> {
    response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|content_type| content_type.to_str().ok())
        .and_then(|content_type| content_type.parse().ok())
}

pub fn is_problem_json_response(response: &Response) -> bool {
    parse_content_type(response).as_ref() == Some(LazyLock::force(&APPLICATION_PROBLEM_JSON))
}

pub fn default_reqwest_client_builder() -> ClientBuilder {
    // Enable gzip compression by default, but explicitly disable any other compression algorithm,
    // to prevent these from being automatically enabled by `reqwest` feature flags.
    Client::builder()
        .timeout(CLIENT_REQUEST_TIMEOUT)
        .connect_timeout(CLIENT_CONNECT_TIMEOUT)
        .gzip(true)
        .no_brotli()
        .no_zstd()
        .no_deflate()
        .tls_built_in_root_certs(true)
}

/// Create a [`ClientBuilder`] that validates certificates signed with the supplied trust anchors (root certificates) as
/// well as the built-in root certificates.
pub fn trusted_reqwest_client_builder(trust_anchors: impl IntoIterator<Item = Certificate>) -> ClientBuilder {
    trust_anchors
        .into_iter()
        .fold(default_reqwest_client_builder(), |builder, root_ca| {
            builder.add_root_certificate(root_ca)
        })
}

/// Create a [`ClientBuilder`] that only validates certificates signed with the supplied trust anchors (root
/// certificates). The built-in root certificates are therefore disabled and the client will only work over https.
pub fn tls_pinned_client_builder(trust_anchors: impl IntoIterator<Item = Certificate>) -> ClientBuilder {
    trusted_reqwest_client_builder(trust_anchors)
        .https_only(true)
        .tls_built_in_root_certs(false)
}

pub fn client_builder_accept_json(builder: ClientBuilder) -> ClientBuilder {
    builder.default_headers(HeaderMap::from_iter([(
        header::ACCEPT,
        HeaderValue::from_static(APPLICATION_JSON.as_ref()),
    )]))
}

#[cfg(any(test, feature = "test"))]
pub mod test {
    use base64::prelude::BASE64_STANDARD;
    use base64::Engine;
    use utils::vec_nonempty;

    use crate::client::TlsPinningConfig;
    use crate::reqwest::ReqwestTrustAnchor;

    pub const TEST_CERTIFICATE_BASE64: &str = "MIIBUzCB+6ADAgECAhRGv/y0WtvIgkZodTBjwPMTvUcrujAKBggqhkjOPQQDAjAPMQ0wCwYDVQQDDAR0ZXN0MB4XDTI2MDMxMDE2MTUyM1oXDTI3MDMxMTE2MTUyM1owDzENMAsGA1UEAwwEdGVzdDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABIIT6QWW/A3sM0BDxZje7PvqheHP4qA1tEC2fSPBj+RbzOyUl6e39tB8nHZDFpk/UrRoLRYpJfa2PCebGeO+dBmjNTAzMB0GA1UdDgQWBBS/TIByJWDzfwSizPd6VRU/BQE4zTASBgNVHRMBAf8ECDAGAQH/AgEAMAoGCCqGSM49BAMCA0cAMEQCIGahEuSBxSDpgLIdGpSbVfqMKdLQ9c9ErbueoxFZZChbAiAGMCrYxBD4YRrhoiSdIndASo8RI9577oaKprb0KFa+Eg==";

    pub fn get_test_trust_anchor() -> ReqwestTrustAnchor {
        ReqwestTrustAnchor::try_from(BASE64_STANDARD.decode(TEST_CERTIFICATE_BASE64).unwrap()).unwrap()
    }

    pub fn get_tls_pinning_config_for_url(url: &str) -> TlsPinningConfig {
        TlsPinningConfig::try_new(url.parse().unwrap(), vec_nonempty![get_test_trust_anchor()]).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use base64::prelude::Engine;
    use base64::prelude::BASE64_STANDARD;
    use rstest::rstest;

    use crate::reqwest::test::TEST_CERTIFICATE_BASE64;

    use super::ReqwestTrustAnchor;
    use super::ReqwestTrustAnchorError;

    #[test]
    fn request_trust_anchor_from_bytes_success() {
        let der_bytes = BASE64_STANDARD.decode(TEST_CERTIFICATE_BASE64).expect("valid base64");

        let trust_anchor_result: Result<ReqwestTrustAnchor, ReqwestTrustAnchorError> = der_bytes.try_into();
        let _ = trust_anchor_result.unwrap();
    }

    #[rstest]
    #[case("")]
    #[case("SGVsbG8gV29ybGQh")]
    fn reqwest_trust_anchor_from_bytes_errors(#[case] input: &str) {
        let der_bytes = BASE64_STANDARD.decode(input).expect("valid base64");

        let trust_anchor_result: Result<ReqwestTrustAnchor, ReqwestTrustAnchorError> = der_bytes.try_into();

        // `unwrap` not possible because `ReqwestTrustAnchor` is not `Debug`
        let Err(error) = trust_anchor_result else {
            panic!("expected an error");
        };

        assert_matches!(error, ReqwestTrustAnchorError::X509Parser(_));
    }
}
