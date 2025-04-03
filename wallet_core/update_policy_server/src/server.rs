use std::net::SocketAddr;
use std::net::TcpListener;
use std::sync::Arc;

use anyhow::Result;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use axum::Json;
use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use etag::EntityTag;
use http::header;
use http::HeaderMap;
use http::HeaderValue;
use tracing::debug;
use tracing::info;

use http_utils::http::TlsServerConfig;
use wallet_common::built_info::version_string;
use wallet_common::generator::TimeGenerator;

use crate::config::UpdatePolicyConfig;
use crate::settings::Settings;

#[derive(Clone)]
struct ApplicationState {
    update_policy: UpdatePolicyConfig,
}

pub async fn serve(settings: Settings) -> Result<()> {
    let listener = TcpListener::bind(SocketAddr::new(settings.ip, settings.port))?;
    serve_with_listener(listener, settings).await
}

pub async fn serve_with_listener(listener: TcpListener, settings: Settings) -> Result<()> {
    info!("{}", version_string());
    info!("listening on {}", listener.local_addr()?);
    listener.set_nonblocking(true)?;

    let application_state = Arc::new(ApplicationState {
        update_policy: settings.update_policy,
    });

    let app = Router::new().merge(health_router()).nest(
        "/update/v1",
        Router::new()
            .route("/update-policy", get(get_policy))
            .with_state(application_state),
    );

    if let Some(TlsServerConfig { key, cert }) = settings.tls_config.clone() {
        let config = RustlsConfig::from_der(vec![cert], key).await?;
        axum_server::from_tcp_rustls(listener, config)
            .serve(app.into_make_service())
            .await?;
    } else {
        axum_server::from_tcp(listener).serve(app.into_make_service()).await?;
    }

    Ok(())
}

fn health_router() -> Router {
    Router::new().route("/health", get(|| async {}))
}

async fn get_policy(State(state): State<Arc<ApplicationState>>, headers: HeaderMap) -> Result<Response, StatusCode> {
    info!("Received update policy request");

    let policy = state.update_policy.clone().into_response(&TimeGenerator);
    let policy_entity_tag = EntityTag::from_data(&postcard::to_allocvec(&policy).unwrap());

    if let Some(etag) = headers.get(header::IF_NONE_MATCH) {
        let entity_tag = etag
            .to_str()
            .ok()
            .and_then(|etag| etag.parse().ok())
            .ok_or(StatusCode::BAD_REQUEST)?;

        // Comparing etags using the If-None-Match header uses the weak comparison algorithm.
        if policy_entity_tag.weak_eq(&entity_tag) {
            debug!("Policy is not modified");
            return Err(StatusCode::NOT_MODIFIED);
        }
    }

    let mut resp: Response = Json(policy).into_response();
    resp.headers_mut().append(
        header::ETAG,
        // We can safely unwrap here because we know for sure there are no non-ascii characters used.
        HeaderValue::from_str(&policy_entity_tag.to_string()).unwrap(),
    );

    Ok(resp)
}
