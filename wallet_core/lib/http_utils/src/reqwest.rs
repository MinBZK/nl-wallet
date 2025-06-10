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
    type Error = reqwest::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let certificate = Certificate::from_der(&value)?;
        Ok(Self {
            der_bytes: value,
            certificate,
        })
    }
}

pub trait IntoPinnedReqwestClient {
    fn try_into_custom_client<F>(self, builder_adapter: F) -> Result<PinnedReqwestClient, reqwest::Error>
    where
        F: FnOnce(ClientBuilder) -> ClientBuilder;

    fn try_into_custom_json_client<F>(self, builder_adapter: F) -> Result<PinnedReqwestClient, reqwest::Error>
    where
        F: FnOnce(ClientBuilder) -> ClientBuilder,
        Self: Sized,
    {
        self.try_into_custom_client(|builder| {
            let builder = builder.default_headers(HeaderMap::from_iter([(
                header::ACCEPT,
                HeaderValue::from_static(APPLICATION_JSON.as_ref()),
            )]));

            builder_adapter(builder)
        })
    }

    fn try_into_client(self) -> Result<PinnedReqwestClient, reqwest::Error>
    where
        Self: Sized,
    {
        self.try_into_custom_client(std::convert::identity)
    }

    fn try_into_json_client(self) -> Result<PinnedReqwestClient, reqwest::Error>
    where
        Self: Sized,
    {
        self.try_into_custom_json_client(std::convert::identity)
    }
}

#[derive(Debug, Clone)]
pub enum ReqwestClientUrl<'a> {
    Absolute(Url),
    Relative(&'a str),
}

#[derive(Debug, Clone)]
pub struct PinnedReqwestClient {
    client: Client,
    pub base_url: BaseUrl,
}

impl PinnedReqwestClient {
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
    Client::builder()
        .timeout(CLIENT_REQUEST_TIMEOUT)
        .connect_timeout(CLIENT_CONNECT_TIMEOUT)
        .tls_built_in_root_certs(true)
}

/// Create a [`ClientBuilder`] that validates certificates signed with the supplied trust anchors (root certificates) as
/// well as the built-in root certificates.
pub fn trusted_reqwest_client_builder(trust_anchors: Vec<Certificate>) -> ClientBuilder {
    trust_anchors
        .into_iter()
        .fold(default_reqwest_client_builder(), |builder, root_ca| {
            builder.add_root_certificate(root_ca)
        })
}

/// Create a [`ClientBuilder`] that only validates certificates signed with the supplied trust anchors (root
/// certificates). The built-in root certificates are therefore disabled and the client will only work over https.
pub fn tls_pinned_client_builder(trust_anchors: Vec<Certificate>) -> ClientBuilder {
    trusted_reqwest_client_builder(trust_anchors)
        .https_only(true)
        .tls_built_in_root_certs(false)
}
