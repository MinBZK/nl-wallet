use futures::TryFutureExt;
use reqwest::Response;
use serde::Serialize;
use url::ParseError;

use wallet_common::http_error::HttpJsonErrorBody;
use wallet_common::reqwest::default_reqwest_client_builder;
use wallet_common::reqwest::is_problem_json_response;
use wallet_common::urls::BaseUrl;

use crate::pid::brp::data::*;

#[derive(Debug, thiserror::Error)]
pub enum BrpError {
    #[error("networking error: {0}")]
    Networking(#[from] reqwest::Error),
    #[error("could not parse base URL: {0}")]
    BaseUrl(#[from] ParseError),
    #[error("could not deserialize JSON: {0}")]
    Deserialization(#[from] serde_json::Error),
    #[error("{0}")]
    Conversion(String),
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

    async fn error_for_response(response: Response) -> Result<Response, BrpError> {
        let status = response.status();

        if status.is_client_error() || status.is_server_error() {
            let error = if is_problem_json_response(&response) {
                let bytes = response.bytes().await?;
                let body = serde_json::from_slice::<HttpJsonErrorBody<String>>(&bytes)?;
                BrpError::Conversion(body.detail.unwrap_or(body.r#type))
            } else {
                BrpError::Networking(
                    response
                        .error_for_status()
                        .expect_err("it is already known there is an error"),
                )
            };

            return Err(error);
        }

        Ok(response)
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

        let response = self
            .http_client
            .execute(request)
            .map_err(BrpError::Networking)
            .and_then(|response| async { Self::error_for_response(response).await })
            .await?;

        let body = response.json().await?;

        Ok(body)
    }
}
