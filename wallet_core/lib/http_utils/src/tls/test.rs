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

    fn request_with_client(&self, client: &Client, method: Method, path: impl AsRef<Path>) -> reqwest::RequestBuilder {
        client.request(method, self.base_url.join(&path.as_ref().to_string_lossy()))
    }
}

impl JsonReqwestBuilder for HttpConfig {}

impl ReqwestBuilder for HttpConfig {}
