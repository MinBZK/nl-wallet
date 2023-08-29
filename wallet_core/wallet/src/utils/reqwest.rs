use std::time::Duration;

use http::{header, HeaderMap, HeaderValue};
use reqwest::Client;

const CLIENT_REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
const CLIENT_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

pub fn build_client() -> Client {
    Client::builder()
        .timeout(CLIENT_REQUEST_TIMEOUT)
        .connect_timeout(CLIENT_CONNECT_TIMEOUT)
        .default_headers(HeaderMap::from_iter([(
            header::ACCEPT,
            HeaderValue::from_static("application/json"),
        )]))
        .build()
        .expect("Could not build reqwest HTTP client")
}
