use http::{header, StatusCode};
use indexmap::IndexMap;
use pem::Pem;
use quick_xml::{
    de::from_str,
    events::Event,
    name::{Namespace, ResolveResult::Bound},
    DeError, NsReader,
};
use reqwest::{tls, Certificate, Identity};
use serde::{Deserialize, Deserializer};

use wallet_common::{config::wallet_config::BaseUrl, reqwest::tls_pinned_client_builder};

use crate::gba;

const GBA_NAMESPACE: &[u8] = b"http://www.bprbzk.nl/GBA/LO3/version1.1";
const CATEGORIEVOORKOMENS_TAG: &[u8] = b"categorievoorkomens";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("networking error: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("XML deserialization error: {0}")]
    XmlDeserialization(#[from] quick_xml::de::DeError),
    #[error("XML error: {0}")]
    Xml(#[from] quick_xml::Error),
    #[error("Categorie {0} is mandatory but missing")]
    MissingCategorie(u8),
    #[error("Element number {0} is mandatory but missing")]
    MissingElement(String),
}

impl From<&Error> for StatusCode {
    fn from(value: &Error) -> Self {
        match value {
            gba::Error::Transport(_) => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::PRECONDITION_FAILED,
        }
    }
}

pub fn parse_xml(xml: &str) -> Result<Vec<Categorievoorkomen>, Error> {
    let mut reader = NsReader::from_str(xml);
    reader.trim_text(true);

    let mut categorievoorkomens = Vec::new();

    // Since quick_xml doesn't handle deserializing nested identical tags well, we'll first take the
    // 'categorievoorkomens' tag using event based parsing. After that, we can safely deserialize using Serde.
    loop {
        match reader.read_event() {
            Err(e) => return Err(Error::XmlDeserialization(DeError::InvalidXml(e))),
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                let (ns, local) = reader.resolve_element(e.name());
                if ns == Bound(Namespace(GBA_NAMESPACE)) && local.as_ref() == CATEGORIEVOORKOMENS_TAG {
                    let end = e.to_end();
                    let text = reader.read_text(end.name())?;
                    categorievoorkomens.push(text);
                }
            }
            _ => (),
        }
    }

    let result = categorievoorkomens
        .iter()
        .map(|categorievoorkomen| Ok(from_str(categorievoorkomen)?))
        .collect::<Result<Vec<Categorievoorkomen>, Error>>()?;

    Ok(result)
}

pub struct GbaResponse {
    pub bsn: String,
    pub categorievoorkomens: Vec<Categorievoorkomen>,
}

impl GbaResponse {
    pub fn new(bsn: String, xml: &str) -> Result<Self, Error> {
        Ok(Self {
            bsn,
            categorievoorkomens: parse_xml(xml)?,
        })
    }

    pub fn get_mandatory_voorkomen(&self, category_number: u8) -> Result<&Categorievoorkomen, Error> {
        self.categorievoorkomens
            .iter()
            .find(|voorkomen| voorkomen.categorienummer == category_number)
            .ok_or(Error::MissingCategorie(category_number))
    }

    pub fn get_voorkomens(&self, category_number: u8) -> Vec<&Categorievoorkomen> {
        self.categorievoorkomens
            .iter()
            .filter(|voorkomen| voorkomen.categorienummer == category_number)
            .collect()
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct Categorievoorkomen {
    pub categorienummer: u8,

    #[serde(deserialize_with = "unwrap_list")]
    pub elementen: Elementen,
}

#[derive(Clone, Debug)]
pub struct Elementen {
    pub map: IndexMap<String, String>,
}

impl Elementen {
    pub fn get_mandatory(&self, element_number: &str) -> Result<String, Error> {
        self.map
            .get(element_number)
            .cloned()
            .ok_or(Error::MissingElement(String::from(element_number)))
    }

    pub fn get_optional(&self, element_number: &str) -> Option<String> {
        self.map.get(element_number).cloned()
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct Item {
    nummer: String,
    waarde: String,
}

fn unwrap_list<'de, D>(deserializer: D) -> Result<Elementen, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Items {
        #[serde(default)]
        item: Vec<Item>,
    }

    let items = Items::deserialize(deserializer)?;
    let mut map = IndexMap::new();
    items.item.into_iter().for_each(|item| {
        map.insert(item.nummer, item.waarde);
    });
    Ok(Elementen { map })
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

    pub async fn vraag(&self, bsn: &str) -> Result<GbaResponse, Error> {
        let response = self
            .http_client
            .post(self.base_url.clone().into_inner())
            .basic_auth(self.username.clone(), Some(self.password.clone()))
            .header(header::CONTENT_TYPE, "application/xml;charset=UTF-8")
            .header(header::ACCEPT_CHARSET, "UTF-8")
            .body(VRAAG_REQUEST.replace("{{bsn}}", bsn))
            .send()
            .await?;

        let body = response.text().await?;
        let result = GbaResponse::new(String::from(bsn), &body)?;
        Ok(result)
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
                    <ns0:item>50120</ns0:item>
                    <ns0:item>50210</ns0:item>
                    <ns0:item>50230</ns0:item>
                    <ns0:item>50240</ns0:item>
                    <ns0:item>50310</ns0:item>
                    <ns0:item>50320</ns0:item>
                    <ns0:item>50330</ns0:item>
                    <ns0:item>50410</ns0:item>
                    <ns0:item>50610</ns0:item>
                    <ns0:item>50620</ns0:item>
                    <ns0:item>50630</ns0:item>
                    <ns0:item>50710</ns0:item>
                    <ns0:item>51510</ns0:item>
                    <ns0:item>58310</ns0:item>
                    <ns0:item>58320</ns0:item>
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
