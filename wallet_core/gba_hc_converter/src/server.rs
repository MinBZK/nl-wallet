use std::sync::Arc;

use axum::extract::State;
use axum::routing::get;
use axum::routing::post;
use axum::Json;
use axum::Router;
use http::StatusCode;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;

use http_utils::error::HttpJsonError;
use utils::built_info::version_string;

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

pub async fn serve<T>(listener: TcpListener, gbav_client: T) -> Result<(), Box<dyn std::error::Error>>
where
    T: GbavClient + Send + Sync + 'static,
{
    info! {"{}", version_string()}
    info!("listening on {}", listener.local_addr()?);

    let app_state = Arc::new(ApplicationState { gbav_client });

    let app = Router::new()
        .nest(
            "/haalcentraal/api/brp",
            Router::new()
                .route("/personen", post(personen::<T>))
                .with_state(app_state),
        )
        .layer(TraceLayer::new_for_http())
        .merge(health_router());

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
    info!("received personen request");

    // We can safely unwrap here, because the brpproxy already guarantees there is at least one burgerservicenummer.
    let body = request_personen(&state.gbav_client, payload.bsn.first().unwrap())
        .await
        .inspect_err(|error| info!("error handling request: {:?}", error))?;

    info!("sending personen response");

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
