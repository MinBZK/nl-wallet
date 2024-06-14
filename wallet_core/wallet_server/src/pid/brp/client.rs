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
    async fn get_person_by_bsn(&self, bsn: String) -> Result<BrpPersons, BrpError>;
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
struct GetByBsnRequest {
    r#type: String,

    #[serde(rename = "burgerservicenummer")]
    bsn: Vec<String>,
    fields: Vec<String>,
}

impl GetByBsnRequest {
    fn create(bsn: String) -> GetByBsnRequest {
        GetByBsnRequest {
            r#type: String::from("RaadpleegMetBurgerservicenummer"),
            bsn: vec![bsn],
            fields: vec![
                String::from("burgerservicenummer"),
                String::from("geslacht"),
                String::from("naam.voornamen"),
                String::from("naam.voorvoegsel"),
                String::from("naam.geslachtsnaam"),
                String::from("geboorte"),
                String::from("verblijfplaats.verblijfadres.officieleStraatnaam"),
                String::from("verblijfplaats.verblijfadres.korteStraatnaam"),
                String::from("verblijfplaats.verblijfadres.huisnummer"),
                String::from("verblijfplaats.verblijfadres.postcode"),
                String::from("verblijfplaats.verblijfadres.woonplaats"),
                String::from("leeftijd"),
                String::from("partners.soortVerbintenis.code"),
                String::from("partners.naam.voornamen"),
                String::from("partners.naam.voorvoegsel"),
                String::from("partners.naam.geslachtsnaam"),
                String::from("nationaliteiten.nationaliteit.omschrijving"),
            ],
        }
    }
}

impl BrpClient for HttpBrpClient {
    async fn get_person_by_bsn(&self, bsn: String) -> Result<BrpPersons, BrpError> {
        let url = self.base_url.join("haalcentraal/api/brp/personen");
        let request = self.http_client.post(url).json(&GetByBsnRequest::create(bsn)).build()?;

        let response = self.http_client.execute(request).await?.error_for_status()?;
        let body = response.json().await?;

        Ok(body)
    }
}
