use std::{env, path::PathBuf, result::Result as StdResult, sync::Arc};

use aes_gcm::Aes256Gcm;
use anyhow::anyhow;
use askama::Template;
use axum::{
    extract::{Request, State},
    middleware,
    middleware::Next,
    response::{IntoResponse, Response},
    routing::get,
    Form, Router,
};
use axum_csrf::{CsrfConfig, CsrfLayer, CsrfToken};
use http::{HeaderMap, StatusCode};
use nutype::nutype;
use serde::Deserialize;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{debug, level_filters::LevelFilter, warn};
use tracing_subscriber::EnvFilter;

use gba_hc_converter::{
    fetch::askama_axum,
    gba::{
        client::{GbavClient, HttpGbavClient},
        encryption::{encrypt_bytes_to_dir, HmacSha256},
    },
    haal_centraal::Bsn,
    settings::{PreloadedSettings, RunMode, Settings},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let builder = tracing_subscriber::fmt().with_env_filter(
        EnvFilter::builder()
            .with_default_directive(LevelFilter::INFO.into())
            .from_env_lossy(),
    );

    let settings = Settings::new()?;
    if settings.structured_logging {
        builder.json().init();
    } else {
        builder.init();
    }

    serve(settings).await
}

#[nutype(derive(Debug, From, AsRef))]
pub struct Error(anyhow::Error);

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        warn!("error result: {:?}", self);
        let result = IndexTemplate::from_error(self.as_ref().to_string());
        let mut response = askama_axum::into_response(&result);
        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
        response
    }
}

type Result<T> = StdResult<T, Error>;

struct ApplicationState {
    base_path: PathBuf,
    http_client: HttpGbavClient,
    preloaded_settings: PreloadedSettings,
}

async fn serve(settings: Settings) -> anyhow::Result<()> {
    let listener = TcpListener::bind((settings.ip, settings.port)).await?;
    debug!("listening on {}:{}", settings.ip, settings.port);

    let RunMode::All {
        gbav: gbav_settings,
        preloaded: preloaded_settings,
    } = settings.run_mode
    else {
        return Err(anyhow!("Only Runmode::All is allowed"));
    };

    let csrf_config = CsrfConfig::default();
    let base_path = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap_or_default();
    let http_client = HttpGbavClient::from_settings(gbav_settings).await?;

    let app_state = Arc::new(ApplicationState {
        base_path,
        http_client,
        preloaded_settings,
    });

    let app = Router::new()
        .nest("/health", Router::new().route("/", get(|| async {})))
        .nest(
            "/",
            Router::new()
                .route("/", get(index).post(fetch))
                .with_state(app_state)
                .layer(CsrfLayer::new(csrf_config))
                .layer(middleware::from_fn(check_auth))
                .layer(TraceLayer::new_for_http()),
        );

    axum::serve(listener, app).await?;

    Ok(())
}

async fn check_auth(headers: HeaderMap, request: Request, next: Next) -> StdResult<Response, StatusCode> {
    // This assumes an ingress/reverse proxy that uses mutual TLS and sets the `Cert-Serial` header with the value
    // from the client certificate. This is an extra safeguard against using this endpoint directly.
    if !headers.get("Cert-Serial").is_some_and(|s| !s.is_empty()) {
        return Err(StatusCode::FORBIDDEN);
    }
    let response = next.run(request).await;
    Ok(response)
}

#[derive(Deserialize, Debug)]
struct Payload {
    bsn: String,
    authenticity_token: String,
}

#[derive(Template)]
#[template(path = "index.askama", escape = "html", ext = "html")]
struct IndexTemplate {
    authenticity_token: Option<String>,
    msg: Option<String>,
    error: Option<String>,
}

impl IndexTemplate {
    fn new(authenticity_token: String) -> Self {
        IndexTemplate {
            authenticity_token: Some(authenticity_token),
            error: None,
            msg: None,
        }
    }

    fn from_error(err: String) -> Self {
        IndexTemplate {
            authenticity_token: None,
            error: Some(err),
            msg: None,
        }
    }

    fn from_msg(msg: String) -> Self {
        IndexTemplate {
            authenticity_token: None,
            msg: Some(msg),
            error: None,
        }
    }
}

async fn index(token: CsrfToken) -> Result<Response> {
    let result = IndexTemplate::new(token.authenticity_token().map_err(|err| anyhow!(err))?);
    Ok(askama_axum::into_response_with_csrf(token, &result))
}

async fn fetch(
    State(state): State<Arc<ApplicationState>>,
    token: CsrfToken,
    Form(payload): Form<Payload>,
) -> Result<Response> {
    token.verify(&payload.authenticity_token).map_err(|err| anyhow!(err))?;

    let bsn = Bsn::try_new(payload.bsn).map_err(|err| anyhow!(err))?;

    let xml = state
        .http_client
        .vraag(&bsn)
        .await
        .map_err(|err| anyhow!(err))?
        .ok_or(anyhow!("No GBA-V results found for the supplied BSN"))?;

    encrypt_bytes_to_dir(
        state.preloaded_settings.encryption_key.key::<Aes256Gcm>(),
        state.preloaded_settings.hmac_key.key::<HmacSha256>(),
        xml.as_bytes(),
        &state.base_path.join(&state.preloaded_settings.xml_path),
        bsn.as_ref(),
    )
    .await
    .map_err(|err| anyhow!(err))?;

    let result = IndexTemplate::from_msg(String::from("Ok"));

    Ok(askama_axum::into_response(&result))
}
