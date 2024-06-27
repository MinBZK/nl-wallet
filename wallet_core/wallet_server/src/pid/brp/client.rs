use serde::Serialize;
use url::ParseError;

use wallet_common::{config::wallet_config::BaseUrl, reqwest::default_reqwest_client_builder};

use crate::pid::brp::data::*;

#[derive(Debug, thiserror::Error)]
pub enum BrpError {
    #[error("networking error: {0}")]
    Networking(#[from] reqwest::Error),
    #[error("could not parse base URL: {0}")]
    BaseUrl(#[from] ParseError),
    #[error("could not deserialize JSON: {0}")]
    Deserialization(#[from] serde_json::Error),
}

pub trait BrpClient {
    async fn get_person_by_bsn(&self, bsn: &str) -> Result<BrpPersons, BrpError>;
}

pub struct HttpBrpClient {
    http_client: reqwest::Client,
    base_url: BaseUrl,
}

impl HttpBrpClient {
    pub fn new(base_url: BaseUrl) -> Self {
        Self {
            http_client: default_reqwest_client_builder()
                .build()
                .expect("Could not build reqwest HTTP client"),
            base_url,
        }
    }
}

#[derive(Serialize)]
struct GetByBsnRequest<'a> {
    r#type: &'a str,
    #[serde(rename = "burgerservicenummer")]
    bsn: &'a [&'a str],
    fields: &'a [&'a str],
}

impl GetByBsnRequest<'_> {
    const TYPE: &'static str = "RaadpleegMetBurgerservicenummer";
    const FIELDS: [&'static str; 17] = [
        "burgerservicenummer",
        "geslacht",
        "naam.voornamen",
        "naam.voorvoegsel",
        "naam.geslachtsnaam",
        "naam.aanduidingNaamgebruik",
        "geboorte",
        "verblijfplaats.verblijfadres.officieleStraatnaam",
        "verblijfplaats.verblijfadres.korteStraatnaam",
        "verblijfplaats.verblijfadres.huisnummer",
        "verblijfplaats.verblijfadres.postcode",
        "verblijfplaats.verblijfadres.woonplaats",
        "leeftijd",
        "partners.naam.voorvoegsel",
        "partners.naam.geslachtsnaam",
        "partners.soortVerbintenis.code",
        "partners.aangaanHuwelijkPartnerschap",
    ];
}

impl BrpClient for HttpBrpClient {
    async fn get_person_by_bsn(&self, bsn: &str) -> Result<BrpPersons, BrpError> {
        let url = self.base_url.join("haalcentraal/api/brp/personen");
        let request = self
            .http_client
            .post(url)
            .json(&GetByBsnRequest {
                r#type: GetByBsnRequest::TYPE,
                bsn: &[bsn],
                fields: &GetByBsnRequest::FIELDS,
            })
            .build()?;

        let response = self.http_client.execute(request).await?.error_for_status()?;
        let body = response.json().await?;

        Ok(body)
    }
}
