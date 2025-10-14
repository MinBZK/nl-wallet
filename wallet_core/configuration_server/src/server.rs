use std::error::Error;
use std::net::SocketAddr;
use std::net::TcpListener;

use axum::Router;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use etag::EntityTag;
use http::HeaderMap;
use http::HeaderValue;
use http::StatusCode;
use http::header;
use tracing::debug;
use tracing::info;

use jwt::VerifiedJwt;
use utils::built_info::version_string;
use wallet_configuration::wallet_config::WalletConfiguration;

use super::settings::Settings;

pub async fn serve(settings: Settings) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(SocketAddr::new(settings.ip, settings.port))?;
    serve_with_listener(listener, settings).await
}

pub async fn serve_with_listener(listener: TcpListener, settings: Settings) -> Result<(), Box<dyn Error>> {
    info!("{}", version_string());
    info!("listening on {}", listener.local_addr()?);
    listener.set_nonblocking(true)?;

    let config_entity_tag = EntityTag::from_data(settings.wallet_config_jwt.jwt().serialization().as_bytes());
    let app = Router::new().merge(health_router()).nest(
        "/config/v1",
        Router::new()
            .route("/wallet-config", get(configuration))
            .with_state((settings.wallet_config_jwt, config_entity_tag)),
    );

    axum_server::from_tcp_rustls(listener, settings.tls_config.into_rustls_config().await?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

fn health_router() -> Router {
    Router::new().route("/health", get(|| async {}))
}

async fn configuration(
    State((config_jwt, config_entity_tag)): State<(VerifiedJwt<WalletConfiguration>, EntityTag)>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    info!("received configuration request");

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

    let mut resp: Response = config_jwt.to_string().into_response();
    resp.headers_mut().append(
        header::ETAG,
        // We can safely unwrap here because we know for sure there are no non-ascii characters used.
        HeaderValue::from_str(&config_entity_tag.to_string()).unwrap(),
    );

    info!("replying with the configuration");
    Ok(resp)
}
