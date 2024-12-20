use std::hash::Hash;
use std::hash::Hasher;
use std::path::Path;
use std::sync::LazyLock;
use std::time::Duration;

use derive_more::AsRef;
use http::header;
use http::HeaderMap;
use http::HeaderValue;
use mime::Mime;
use reqwest::Client;
use reqwest::Response;

use crate::http_error::APPLICATION_PROBLEM_JSON;

const CLIENT_REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
const CLIENT_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

/// Wrapper around a `reqwest::Certificate` implementing `PartialEq`, `Eq` and `Hash`. In addition, it implements
/// the necessary `From`/`TryFrom` implementations so that it can be (de)serialised using `serde_with`.
#[derive(Clone, AsRef)]
pub struct ReqwestTrustAnchor {
    #[as_ref([u8])]
    der_bytes: Vec<u8>,
    certificate: reqwest::Certificate,
}

impl ReqwestTrustAnchor {
    pub fn as_certificate(&self) -> &reqwest::Certificate {
        &self.certificate
    }

    pub fn into_certificate(self) -> reqwest::Certificate {
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
        let certificate = reqwest::Certificate::from_der(&value)?;
        Ok(Self {
            der_bytes: value,
            certificate,
        })
    }
}

pub trait ClientBuilder {
    fn builder(&self) -> reqwest::ClientBuilder;
}

pub trait JsonClientBuilder: ClientBuilder {
    fn json_builder(&self) -> reqwest::ClientBuilder {
        self.builder().default_headers(HeaderMap::from_iter([(
            header::ACCEPT,
            HeaderValue::from_static("application/json"),
        )]))
    }
}

#[trait_variant::make(Send)]
pub trait RequestBuilder {
    fn request(&self, method: reqwest::Method, path: impl AsRef<Path>) -> (reqwest::Client, reqwest::RequestBuilder);

    fn request_with_client(
        &self,
        client: &reqwest::Client,
        method: reqwest::Method,
        path: impl AsRef<Path>,
    ) -> reqwest::RequestBuilder;

    fn get(&self, path: impl AsRef<Path>) -> (reqwest::Client, reqwest::RequestBuilder) {
        self.request(reqwest::Method::GET, path)
    }

    fn get_with_client(&self, client: &reqwest::Client, path: impl AsRef<Path>) -> reqwest::RequestBuilder {
        self.request_with_client(client, reqwest::Method::GET, path)
    }

    fn post(&self, path: impl AsRef<Path>) -> (reqwest::Client, reqwest::RequestBuilder) {
        self.request(reqwest::Method::POST, path)
    }

    fn post_with_client(&self, client: &reqwest::Client, path: impl AsRef<Path>) -> reqwest::RequestBuilder {
        self.request_with_client(client, reqwest::Method::POST, path)
    }

    fn delete(&self, path: &impl AsRef<Path>) -> (reqwest::Client, reqwest::RequestBuilder) {
        self.request(reqwest::Method::DELETE, path)
    }

    fn delete_with_client(&self, client: &reqwest::Client, path: impl AsRef<Path>) -> reqwest::RequestBuilder {
        self.request_with_client(client, reqwest::Method::DELETE, path)
    }
}

pub trait ReqwestBuilder: ClientBuilder + RequestBuilder {}

pub trait JsonReqwestBuilder: JsonClientBuilder + RequestBuilder {}

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

pub fn default_reqwest_client_builder() -> reqwest::ClientBuilder {
    Client::builder()
        .timeout(CLIENT_REQUEST_TIMEOUT)
        .connect_timeout(CLIENT_CONNECT_TIMEOUT)
        .tls_built_in_root_certs(true)
}

/// Create a [`ClientBuilder`] that validates certificates signed with the supplied trust anchors (root certificates) as
/// well as the built-in root certificates.
pub fn trusted_reqwest_client_builder(trust_anchors: Vec<reqwest::Certificate>) -> reqwest::ClientBuilder {
    trust_anchors
        .into_iter()
        .fold(default_reqwest_client_builder(), |builder, root_ca| {
            builder.add_root_certificate(root_ca)
        })
}

/// Create a [`ClientBuilder`] that only validates certificates signed with the supplied trust anchors (root
/// certificates). The built-in root certificates are therefore disabled and the client will only work over https.
pub fn tls_pinned_client_builder(trust_anchors: Vec<reqwest::Certificate>) -> reqwest::ClientBuilder {
    trusted_reqwest_client_builder(trust_anchors)
        .https_only(true)
        .tls_built_in_root_certs(false)
}
