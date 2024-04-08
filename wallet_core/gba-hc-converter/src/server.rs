use std::net::{SocketAddr, TcpListener};

use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use http::{header, HeaderValue, StatusCode};
use tracing::{debug, info};

use wallet_common::http_error::{ErrorData, APPLICATION_PROBLEM_JSON};

use crate::{
    gba,
    gba::HttpGbavClient,
    haal_centraal,
    haal_centraal::{PersonenQuery, PersonenResponse},
};

use super::settings::Settings;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("GBA error: {0}")]
    Gba(#[from] gba::Error),
    #[error("Error converting GBA-V XML to Haal-Centraal JSON: {0}")]
    Conversion(#[from] haal_centraal::Error),
}

impl From<&Error> for StatusCode {
    fn from(value: &Error) -> Self {
        value.into()
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        info!("error handling request: {:?}", &self);

        let status_code: StatusCode = (&self).into();

        let error_data = ErrorData {
            typ: match self {
                Error::Gba(_) => "gba_error",
                Error::Conversion(_) => "conversion_error",
            },
            title: self.to_string(),
        };

        (
            status_code,
            [(
                header::CONTENT_TYPE,
                HeaderValue::from_static(APPLICATION_PROBLEM_JSON.as_ref()),
            )],
            Json(error_data),
        )
            .into_response()
    }
}

pub async fn serve(settings: Settings) -> Result<(), Box<dyn std::error::Error>> {
    let socket = SocketAddr::new(settings.ip, settings.port);
    let listener = TcpListener::bind(socket)?;
    debug!("listening on {}", socket);

    let app = Router::new().nest("/", health_router()).nest(
        "/haalcentraal/api/brp",
        Router::new().route("/personen", post(personen)).with_state(settings),
    );

    axum::Server::from_tcp(listener)?.serve(app.into_make_service()).await?;

    Ok(())
}

fn health_router() -> Router {
    Router::new().route("/health", get(|| async {}))
}

async fn personen(
    State(settings): State<Settings>,
    Json(payload): Json<PersonenQuery>,
) -> Result<(StatusCode, Json<PersonenResponse>), Error> {
    info!("Received personen request");

    let client = HttpGbavClient::new(
        settings.url,
        settings.username,
        settings.password,
        settings.trust_anchor,
        settings.client_cert,
        settings.client_cert_key,
    )?;

    // We can safely unwrap here, because the brpproxy already guaranteees there is at least one burgerservicenummer.
    let gba_response = client.vraag(payload.burgerservicenummer.first().unwrap()).await?;

    let mut body = PersonenResponse::create(gba_response)?;
    body.filter_terminated_nationalities();

    Ok((StatusCode::OK, body.into()))
}
