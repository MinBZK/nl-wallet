use std::net::IpAddr;
use std::sync::Arc;

use axum::extract::State;
use axum::routing::get;
use axum::routing::post;
use axum::Json;
use axum::Router;
use http::StatusCode;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::debug;
use tracing::info;

use wallet_common::http_error::HttpJsonError;

use crate::error::Error;
use crate::error::ErrorType;
use crate::gba::client::GbavClient;
use crate::gba::data::GbaResponse;
use crate::haal_centraal::Bsn;
use crate::haal_centraal::PersonQuery;
use crate::haal_centraal::PersonsResponse;

struct ApplicationState<T> {
    gbav_client: T,
}

pub async fn serve<T>(ip: IpAddr, port: u16, gbav_client: T) -> Result<(), Box<dyn std::error::Error>>
where
    T: GbavClient + Send + Sync + 'static,
{
    let listener = TcpListener::bind((ip, port)).await?;
    debug!("listening on {}:{}", ip, port);

    let app_state = Arc::new(ApplicationState { gbav_client });

    let app = Router::new()
        .nest("/", health_router())
        .nest(
            "/haalcentraal/api/brp",
            Router::new()
                .route("/personen", post(personen::<T>))
                .with_state(app_state),
        )
        .layer(TraceLayer::new_for_http());

    axum::serve(listener, app).await?;

    Ok(())
}

fn health_router() -> Router {
    Router::new().route("/health", get(|| async {}))
}

async fn personen<T>(
    State(state): State<Arc<ApplicationState<T>>>,
    Json(payload): Json<PersonQuery>,
) -> Result<(StatusCode, Json<PersonsResponse>), HttpJsonError<ErrorType>>
where
    T: GbavClient + Sync,
{
    info!("Received personen request");

    // We can safely unwrap here, because the brpproxy already guarantees there is at least one burgerservicenummer.
    let body = request_personen(&state.gbav_client, payload.bsn.first().unwrap())
        .await
        .inspect_err(|error| info!("error handling request: {:?}", error))?;

    info!("Sending personen response");

    Ok((StatusCode::OK, body.into()))
}

async fn request_personen<T>(gbav_client: &T, bsn: &Bsn) -> Result<PersonsResponse, Error>
where
    T: GbavClient,
{
    let response = gbav_client.vraag(bsn).await?;
    let gba_response = response.map_or(Ok(GbaResponse::empty()), |xml| GbaResponse::new(&xml))?;
    gba_response.as_error()?;

    let mut response = PersonsResponse::create(gba_response)?;
    response.filter_terminated_nationalities();

    Ok(response)
}
