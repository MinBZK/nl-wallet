use const_decoder::Pem;
use const_decoder::decode;
use reqwest::Certificate;
use reqwest::ClientBuilder;
use utils::vec_nonempty;

use crate::client::HttpConfigError;
use crate::client::TlsPinningConfig;
use crate::reqwest::tls_reqwest_client_builder;
use crate::urls::BaseUrl;

// Source: https://github.com/httpmock/httpmock/raw/master/certs/ca.pem
const HTTPMOCK_ROOT_CA: &[u8] = &decode!(Pem, include_bytes!("../assets/httpmock-ca.pem"));

pub fn httpmock_reqwest_client_builder() -> ClientBuilder {
    tls_reqwest_client_builder([Certificate::from_der(HTTPMOCK_ROOT_CA).unwrap()])
}

impl TlsPinningConfig {
    pub fn try_new_httpmock(base_url: BaseUrl) -> Result<Self, HttpConfigError> {
        Self::try_new(base_url, vec_nonempty![HTTPMOCK_ROOT_CA.to_vec().try_into().unwrap()])
    }
}
