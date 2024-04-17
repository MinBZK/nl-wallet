use std::{
    net::{IpAddr, SocketAddr, TcpListener},
    sync::Arc,
};

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use http::StatusCode;
use tracing::{debug, info};

use crate::{
    error::Error,
    gba::client::GbavClient,
    haal_centraal::{PersonQuery, PersonsResponse},
};

struct ApplicationState<T> {
    gbav_client: T,
}

pub async fn serve<T>(ip: IpAddr, port: u16, gbav_client: T) -> Result<(), Box<dyn std::error::Error>>
where
    T: GbavClient + Send + Sync + 'static,
{
    let socket = SocketAddr::new(ip, port);
    let listener = TcpListener::bind(socket)?;
    debug!("listening on {}", socket);

    let app_state = Arc::new(ApplicationState { gbav_client });

    let app = Router::new().nest("/", health_router()).nest(
        "/haalcentraal/api/brp",
        Router::new()
            .route("/personen", post(personen::<T>))
            .with_state(app_state),
    );

    axum::Server::from_tcp(listener)?.serve(app.into_make_service()).await?;

    Ok(())
}

fn health_router() -> Router {
    Router::new().route("/health", get(|| async {}))
}

async fn personen<T>(
    State(state): State<Arc<ApplicationState<T>>>,
    Json(payload): Json<PersonQuery>,
) -> Result<(StatusCode, Json<PersonsResponse>), Error>
where
    T: GbavClient + Sync,
{
    info!("Received personen request");

    // We can safely unwrap here, because the brpproxy already guarantees there is at least one burgerservicenummer.
    let gba_response = state.gbav_client.vraag(payload.bsn.first().unwrap()).await?;

    let mut body = PersonsResponse::create(gba_response)?;
    body.filter_terminated_nationalities();

    Ok((StatusCode::OK, body.into()))
}
