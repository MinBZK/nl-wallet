use std::error::Error;
use std::net::SocketAddr;
use std::net::TcpListener;

use axum::extract::State;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use etag::EntityTag;
use http::header;
use http::HeaderMap;
use http::HeaderValue;
use http::StatusCode;
use tracing::debug;
use tracing::info;

use utils::built_info::version_string;

use super::settings::Settings;

pub async fn serve(settings: Settings) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(SocketAddr::new(settings.ip, settings.port))?;
    serve_with_listener(listener, settings).await
}

pub async fn serve_with_listener(listener: TcpListener, settings: Settings) -> Result<(), Box<dyn Error>> {
    info!("{}", version_string());
    info!("listening on {}", listener.local_addr()?);
    listener.set_nonblocking(true)?;

    let app = Router::new().merge(health_router()).nest(
        "/config/v1",
        Router::new()
            .route("/wallet-config", get(configuration))
            .with_state(settings.wallet_config_jwt.into_bytes()),
    );

    axum_server::from_tcp_rustls(listener, settings.tls_config.to_rustls_config().await?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

fn health_router() -> Router {
    Router::new().route("/health", get(|| async {}))
}

async fn configuration(
    State(config_jwt): State<Vec<u8>>,
    headers: HeaderMap,
) -> std::result::Result<Response, StatusCode> {
    info!("received configuration request");

    let config_entity_tag = EntityTag::from_data(config_jwt.as_ref());

    if let Some(etag) = headers.get(header::IF_NONE_MATCH) {
        let entity_tag = etag
            .to_str()
            .ok()
            .and_then(|etag| etag.parse().ok())
            .ok_or(StatusCode::BAD_REQUEST)?;

        // Comparing etags using the If-None-Match header uses the weak comparison algorithm.
        if config_entity_tag.weak_eq(&entity_tag) {
            debug!("configuration is not modified");
            return Err(StatusCode::NOT_MODIFIED);
        }
    }

    let mut resp: Response = config_jwt.into_response();
    resp.headers_mut().append(
        header::ETAG,
        // We can safely unwrap here because we know for sure there are no non-ascii characters used.
        HeaderValue::from_str(&config_entity_tag.to_string()).unwrap(),
    );

    info!("replying with the configuration");
    Ok(resp)
}
