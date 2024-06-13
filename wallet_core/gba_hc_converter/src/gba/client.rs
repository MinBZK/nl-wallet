use std::{env, path::PathBuf};

use http::header;
use pem::Pem;
use reqwest::{tls, Certificate, Identity};
use tracing::info;

use wallet_common::{config::wallet_config::BaseUrl, reqwest::tls_pinned_client_builder};

use crate::{
    gba::{data::GbaResponse, error::Error},
    haal_centraal::Bsn,
};

#[trait_variant::make(GbavClient: Send)]
pub trait GbavClientLocal {
    #[allow(dead_code)]
    async fn vraag(&self, bsn: &Bsn) -> Result<GbaResponse, Error>;
}

pub struct HttpGbavClient {
    http_client: reqwest::Client,
    base_url: BaseUrl,
    username: String,
    password: String,
}

impl HttpGbavClient {
    pub fn new(
        base_url: BaseUrl,
        username: String,
        password: String,
        trust_anchor: Certificate,
        client_cert: Vec<u8>,
        client_cert_key: Vec<u8>,
    ) -> Result<Self, Error> {
        let cert = Pem::new("CERTIFICATE", client_cert);
        let key = Pem::new("PRIVATE KEY", client_cert_key);
        let cert_buf = pem::encode(&key) + &pem::encode(&cert);

        let client = Self {
            http_client: tls_pinned_client_builder(vec![trust_anchor])
                // TLS_1_3 is currently not supported and version negotiation seems broken
                .max_tls_version(tls::Version::TLS_1_2)
                .identity(Identity::from_pem(cert_buf.as_bytes())?)
                .build()
                .expect("Could not build reqwest HTTP client"),
            base_url,
            username,
            password,
        };

        Ok(client)
    }
}

impl GbavClient for HttpGbavClient {
    async fn vraag(&self, bsn: &Bsn) -> Result<GbaResponse, Error> {
        info!("Sending GBA-V request to: {}", &self.base_url.clone().into_inner());

        let response = self
            .http_client
            .post(self.base_url.clone().into_inner())
            .basic_auth(self.username.clone(), Some(self.password.clone()))
            .header(header::CONTENT_TYPE, "application/xml;charset=UTF-8")
            .header(header::ACCEPT_CHARSET, "UTF-8")
            .body(VRAAG_REQUEST.replace("{{bsn}}", &bsn.to_string()))
            .send()
            .await?;

        info!("Received GBA-V response with status: {}", &response.status());

        let body = response.text().await?;
        let result = GbaResponse::new(&body)?;
        Ok(result)
    }
}

pub struct FileGbavClient<T> {
    base_path: PathBuf,
    client: T,
}

impl<T> FileGbavClient<T> {
    pub fn new(path: PathBuf, client: T) -> Self {
        let mut base_path = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap_or_default();
        base_path.push(path.as_path());
        Self { base_path, client }
    }
}

impl<T> GbavClient for FileGbavClient<T>
where
    T: GbavClient + Sync,
{
    async fn vraag(&self, bsn: &Bsn) -> Result<GbaResponse, Error> {
        let xml_file = self.base_path.join(format!("{bsn}.xml"));
        if xml_file.exists() {
            let xml = tokio::fs::read_to_string(xml_file).await?;
            let gba_response = GbaResponse::new(&xml)?;
            Ok(gba_response)
        } else {
            self.client.vraag(bsn).await
        }
    }
}

pub struct NoopGbavClient {}
impl GbavClient for NoopGbavClient {
    async fn vraag(&self, _bsn: &Bsn) -> Result<GbaResponse, Error> {
        Ok(GbaResponse::empty())
    }
}

const VRAAG_REQUEST: &str = r#"
<soap-env:Envelope xmlns:soap-env="http://schemas.xmlsoap.org/soap/envelope/">
    <soap-env:Body>
        <ns0:vraag xmlns:ns0="http://www.bprbzk.nl/GBA/LRDPlus/version1.1">
            <ns0:in0>
                <ns0:indicatieAdresvraag xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
                    xsi:nil="true" />
                <ns0:indicatieZoekenInHistorie xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
                    xsi:nil="true" />
                <ns0:masker>
                    <ns0:item>10120</ns0:item>
                    <ns0:item>10210</ns0:item>
                    <ns0:item>10230</ns0:item>
                    <ns0:item>10240</ns0:item>
                    <ns0:item>10310</ns0:item>
                    <ns0:item>10320</ns0:item>
                    <ns0:item>10330</ns0:item>
                    <ns0:item>10410</ns0:item>
                    <ns0:item>16110</ns0:item>
                    <ns0:item>18310</ns0:item>
                    <ns0:item>18320</ns0:item>
                    <ns0:item>40510</ns0:item>
                    <ns0:item>46310</ns0:item>
                    <ns0:item>46410</ns0:item>
                    <ns0:item>46510</ns0:item>
                    <ns0:item>48210</ns0:item>
                    <ns0:item>48220</ns0:item>
                    <ns0:item>48230</ns0:item>
                    <ns0:item>48310</ns0:item>
                    <ns0:item>48320</ns0:item>
                    <ns0:item>48510</ns0:item>
                    <ns0:item>50230</ns0:item>
                    <ns0:item>50240</ns0:item>
                    <ns0:item>50610</ns0:item>
                    <ns0:item>50710</ns0:item>
                    <ns0:item>51510</ns0:item>
                    <ns0:item>81110</ns0:item>
                    <ns0:item>81115</ns0:item>
                    <ns0:item>81120</ns0:item>
                    <ns0:item>81130</ns0:item>
                    <ns0:item>81140</ns0:item>
                    <ns0:item>81160</ns0:item>
                    <ns0:item>81170</ns0:item>
                    <ns0:item>87210</ns0:item>
                </ns0:masker>
                <ns0:parameters>
                    <ns0:item>
                        <ns0:rubrieknummer>10120</ns0:rubrieknummer>
                        <ns0:zoekwaarde>{{bsn}}</ns0:zoekwaarde>
                    </ns0:item>
                </ns0:parameters>
            </ns0:in0>
        </ns0:vraag>
    </soap-env:Body>
</soap-env:Envelope>
"#;
