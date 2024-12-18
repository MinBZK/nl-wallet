use std::error::Error;
use std::path::Path;
use std::sync::LazyLock;
use std::time::Duration;

use base64::prelude::*;
use http::header;
use http::HeaderMap;
use http::HeaderValue;
use mime::Mime;
use reqwest::Certificate;
use reqwest::Client;
use reqwest::Response;
use serde::Deserialize;
use serde::Deserializer;

use crate::http_error::APPLICATION_PROBLEM_JSON;

const CLIENT_REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
const CLIENT_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

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

fn base64_to_certificate(encoded_certificate: String) -> Result<reqwest::Certificate, Box<dyn Error>> {
    let der_bytes = BASE64_STANDARD.decode(encoded_certificate)?;
    let certificate = reqwest::Certificate::from_der(&der_bytes)?;
    Ok(certificate)
}

pub fn deserialize_certificate<'de, D>(deserializer: D) -> Result<reqwest::Certificate, D::Error>
where
    D: Deserializer<'de>,
{
    let encoded_certificate = String::deserialize(deserializer).map_err(serde::de::Error::custom)?;
    let certificate = base64_to_certificate(encoded_certificate).map_err(serde::de::Error::custom)?;
    Ok(certificate)
}

pub fn deserialize_certificates<'de, D>(deserializer: D) -> Result<Vec<reqwest::Certificate>, D::Error>
where
    D: Deserializer<'de>,
{
    let encoded_certificates: Vec<String> = Vec::deserialize(deserializer).map_err(serde::de::Error::custom)?;
    let certificates = encoded_certificates
        .into_iter()
        .map(|encoded_certificate| {
            let certificate = base64_to_certificate(encoded_certificate).map_err(serde::de::Error::custom)?;
            Ok(certificate)
        })
        .collect::<Result<_, _>>()?;

    Ok(certificates)
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

pub fn default_reqwest_client_builder() -> reqwest::ClientBuilder {
    Client::builder()
        .timeout(CLIENT_REQUEST_TIMEOUT)
        .connect_timeout(CLIENT_CONNECT_TIMEOUT)
        .tls_built_in_root_certs(true)
}

/// Create a [`ClientBuilder`] that validates certificates signed with the supplied trust anchors (root certificates) as
/// well as the built-in root certificates.
pub fn trusted_reqwest_client_builder(trust_anchors: Vec<Certificate>) -> reqwest::ClientBuilder {
    trust_anchors
        .into_iter()
        .fold(default_reqwest_client_builder(), |builder, root_ca| {
            builder.add_root_certificate(root_ca)
        })
}

/// Create a [`ClientBuilder`] that only validates certificates signed with the supplied trust anchors (root
/// certificates). The built-in root certificates are therefore disabled and the client will only work over https.
pub fn tls_pinned_client_builder(trust_anchors: Vec<Certificate>) -> reqwest::ClientBuilder {
    trusted_reqwest_client_builder(trust_anchors)
        .https_only(true)
        .tls_built_in_root_certs(false)
}
