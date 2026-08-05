use std::error::Error;
use std::net::SocketAddr;
use std::path::PathBuf;

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
use http_utils::health::create_health_router;
use jwt::VerifiedJwt;
use status_lists::serve::StatusListRouteSource;
use status_lists::serve::create_serve_router;
use tokio::net::TcpListener;
use tracing::debug;
use tracing::info;
use utils::built_info::version_string;
use wallet_configuration::wallet_config::WalletConfiguration;

use super::settings::Settings;

pub async fn serve(settings: Settings) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(SocketAddr::new(settings.ip, settings.port)).await?;
    let crl_listener = TcpListener::bind(SocketAddr::new(settings.ip, settings.crl_port)).await?;
    let crl_file = settings.crl_file.clone();

    tokio::try_join!(
        serve_with_listener(listener, settings),
        serve_crl_with_listener(crl_listener, crl_file),
    )?;

    Ok(())
}

pub async fn serve_with_listener(listener: TcpListener, settings: Settings) -> Result<(), Box<dyn Error>> {
    info!("{}", version_string());
    info!("listening on {}", listener.local_addr()?);
    let listener = listener.into_std()?;

    let config_entity_tag = EntityTag::from_data(settings.wallet_config_jwt.jwt().serialization().as_bytes());
    let config_router = Router::new()
        .route("/wallet-config", get(configuration))
        .with_state((settings.wallet_config_jwt, config_entity_tag));

    let status_list_router = create_serve_router([StatusListRouteSource {
        path: "/wia",
        publish_dir: settings.wua_publish_dir,
        ttl: None,
    }])?;

    let app = Router::new()
        .merge(create_health_router([]))
        .merge(status_list_router)
        .nest("/config/v1", config_router);

    axum_server::from_tcp_rustls(listener, settings.tls_config.into_rustls_config()?)
        .expect("TCP listener should not be in blocking mode")
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn serve_crl_with_listener(listener: TcpListener, crl_file: PathBuf) -> Result<(), Box<dyn Error>> {
    info!("listening for CRL requests on {}", listener.local_addr()?);

    let app = Router::new().route("/wrpac.crl.der", get(crl)).with_state(crl_file);

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

async fn crl(State(crl_file): State<PathBuf>) -> Result<Response, StatusCode> {
    let bytes = tokio::fs::read(crl_file).await.map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })?;

    Ok((
        [(header::CONTENT_TYPE, HeaderValue::from_static("application/pkix-crl"))],
        bytes,
    )
        .into_response())
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

#[cfg(test)]
mod tests {
    use std::time::SystemTime;
    use std::time::UNIX_EPOCH;

    use axum::body::to_bytes;

    use super::*;

    fn temp_crl_file(name: &str) -> PathBuf {
        let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        std::env::temp_dir().join(format!("static-server-{name}-{unique}.crl.der"))
    }

    #[tokio::test]
    async fn serves_crl_with_pkix_content_type() {
        let crl_file = temp_crl_file("wrpac");
        let crl_bytes = b"example CRL";
        tokio::fs::write(&crl_file, crl_bytes).await.unwrap();

        let response = crl(State(crl_file.clone())).await.unwrap();

        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            HeaderValue::from_static("application/pkix-crl")
        );
        assert_eq!(
            to_bytes(response.into_body(), crl_bytes.len()).await.unwrap(),
            crl_bytes.as_slice()
        );
        tokio::fs::remove_file(crl_file).await.unwrap();
    }

    #[tokio::test]
    async fn returns_not_found_for_missing_crl() {
        let error = crl(State(temp_crl_file("missing"))).await.unwrap_err();

        assert_eq!(error, StatusCode::NOT_FOUND);
    }
}
