use std::path::Path;
use std::path::PathBuf;
use std::str;

use aes_gcm::Aes256Gcm;
use base64::prelude::*;
use http::header;
use pem::Pem;
use reqwest::Certificate;
use reqwest::Identity;
use tracing::info;

use http_utils::reqwest::tls_pinned_client_builder;
use http_utils::urls::BaseUrl;
use wallet_common::utils;

use crate::gba::encryption::decrypt_bytes_from_dir;
use crate::gba::encryption::HmacSha256;
use crate::gba::error::Error;
use crate::haal_centraal::Bsn;
use crate::settings::SymmetricKey;

#[trait_variant::make(Send)]
pub trait GbavClient {
    async fn vraag(&self, bsn: &Bsn) -> Result<Option<String>, Error>;
}

pub struct HttpGbavClient {
    http_client: reqwest::Client,
    base_url: BaseUrl,
    username: String,
    password: String,
    ca_api_key: Option<String>,
    vraag_request_template: String,
}

impl HttpGbavClient {
    const BRP_CREDENTIALS_HEADER_NAME: &'static str = "x-brp-credentials";

    pub async fn new(
        base_url: BaseUrl,
        username: String,
        password: String,
        trust_anchor: Certificate,
        client_cert: Vec<u8>,
        client_cert_key: Vec<u8>,
        ca_api_key: Option<String>,
    ) -> Result<Self, Error> {
        let cert = Pem::new("CERTIFICATE", client_cert);
        let key = Pem::new("PRIVATE KEY", client_cert_key);
        let cert_buf = pem::encode(&key) + &pem::encode(&cert);

        let vraag_request_template_path = utils::prefix_local_path("resources/remote/bsn_zoeken_template.xml".as_ref());
        let vraag_request_template = tokio::fs::read_to_string(vraag_request_template_path).await?;

        let http_client = tls_pinned_client_builder(vec![trust_anchor])
            .identity(Identity::from_pem(cert_buf.as_bytes())?)
            .build()
            .expect("Could not build reqwest HTTP client");

        let client = Self {
            http_client,
            base_url,
            username,
            password,
            ca_api_key,
            vraag_request_template,
        };

        Ok(client)
    }
}

impl GbavClient for HttpGbavClient {
    async fn vraag(&self, bsn: &Bsn) -> Result<Option<String>, Error> {
        info!("Sending GBA-V request to: {}", &self.base_url.clone().into_inner());

        let mut request_builder = self.http_client.post(self.base_url.clone().into_inner());

        if let Some(ca_api_key) = &self.ca_api_key {
            request_builder = request_builder
                .header(header::AUTHORIZATION, format!("CA {}", ca_api_key))
                .header(
                    HttpGbavClient::BRP_CREDENTIALS_HEADER_NAME,
                    format!(
                        "Basic {}",
                        BASE64_STANDARD.encode(format!("{}:{}", self.username.clone(), self.password.clone()))
                    ),
                );
        } else {
            request_builder = request_builder.basic_auth(self.username.clone(), Some(self.password.clone()));
        }

        let response = request_builder
            .header(header::CONTENT_TYPE, "application/xml;charset=UTF-8")
            .header(header::ACCEPT_CHARSET, "UTF-8")
            .body(self.vraag_request_template.replace("{{bsn}}", bsn.as_ref()))
            .send()
            .await?;

        info!("Received GBA-V response with status: {}", &response.status());

        let body = response.text().await?;
        Ok(Some(body))
    }
}

pub struct FileGbavClient<T> {
    base_path: PathBuf,
    decryption_key: SymmetricKey,
    hmac_key: SymmetricKey,
    client: T,
}

impl<T> FileGbavClient<T> {
    pub fn new(path: &Path, decryption_key: SymmetricKey, hmac_key: SymmetricKey, client: T) -> Self {
        let base_path = utils::prefix_local_path(path).into_owned();

        Self {
            base_path,
            decryption_key,
            hmac_key,
            client,
        }
    }
}

impl<T> GbavClient for FileGbavClient<T>
where
    T: GbavClient + Sync,
{
    async fn vraag(&self, bsn: &Bsn) -> Result<Option<String>, Error> {
        let decrypted = decrypt_bytes_from_dir(
            self.decryption_key.key::<Aes256Gcm>(),
            self.hmac_key.key::<HmacSha256>(),
            &self.base_path,
            bsn.as_ref(),
        )
        .await?;

        if let Some(bytes) = decrypted {
            let xml = String::from_utf8(bytes)?;
            info!("Using preloaded data");
            Ok(Some(xml))
        } else {
            info!("No preloaded data found");
            self.client.vraag(bsn).await
        }
    }
}

pub struct NoopGbavClient {}
impl GbavClient for NoopGbavClient {
    async fn vraag(&self, _bsn: &Bsn) -> Result<Option<String>, Error> {
        Ok(None)
    }
}
