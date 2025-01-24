use std::default::Default;
use std::env;
use std::path::PathBuf;
use std::result::Result as StdResult;
use std::sync::Arc;

use aes_gcm::Aes256Gcm;
use anyhow::anyhow;
use askama::Template;
use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::extract::Request;
use axum::extract::State;
use axum::middleware;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use axum::routing::post;
use axum::Form;
use axum::Router;
use axum_csrf::CsrfConfig;
use axum_csrf::CsrfLayer;
use axum_csrf::CsrfToken;
use http::request::Parts;
use http::StatusCode;
use nutype::nutype;
use serde::Deserialize;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing::warn;
use tracing_subscriber::EnvFilter;

use wallet_common::built_info::version_string;

use gba_hc_converter::fetch::askama_axum;
use gba_hc_converter::gba::client::GbavClient;
use gba_hc_converter::gba::client::HttpGbavClient;
use gba_hc_converter::gba::encryption::clear_files_in_dir;
use gba_hc_converter::gba::encryption::count_files_in_dir;
use gba_hc_converter::gba::encryption::encrypt_bytes_to_dir;
use gba_hc_converter::gba::encryption::HmacSha256;
use gba_hc_converter::haal_centraal::Bsn;
use gba_hc_converter::settings::PreloadedSettings;
use gba_hc_converter::settings::RunMode;
use gba_hc_converter::settings::Settings;

const CERT_SERIAL_HEADER: &str = "Cert-Serial";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = Settings::new()?;

    let builder = tracing_subscriber::fmt().with_env_filter(
        EnvFilter::builder()
            .with_default_directive(LevelFilter::INFO.into())
            .from_env_lossy(),
    );

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
        let result = ResultTemplate {
            msg: self.as_ref().to_string(),
        };
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
    info!("{}", version_string());

    info!("listening on {}:{}", settings.ip, settings.port);

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
                .route("/clear", post(clear))
                .with_state(app_state)
                .layer(CsrfLayer::new(csrf_config))
                .layer(middleware::from_fn(check_auth))
                .layer(TraceLayer::new_for_http()),
        );

    axum::serve(listener, app).await?;

    Ok(())
}

#[nutype(derive(Debug, Default), default = "unknown", validate(not_empty))]
struct CertSerial(String);

struct ExtractCertSerial(Option<CertSerial>);

#[async_trait]
impl<S> FromRequestParts<S> for ExtractCertSerial
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> StdResult<Self, Self::Rejection> {
        parts
            .headers
            .get(CERT_SERIAL_HEADER)
            .map(|header| {
                header
                    .to_str()
                    .map_err(anyhow::Error::from)
                    .and_then(|value| CertSerial::try_new(value).map_err(anyhow::Error::from))
            })
            .transpose()
            .map(ExtractCertSerial)
            .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))
    }
}

async fn check_auth(
    ExtractCertSerial(cert_serial): ExtractCertSerial,
    request: Request,
    next: Next,
) -> StdResult<Response, (StatusCode, &'static str)> {
    // This assumes an ingress/reverse proxy that uses mutual TLS and sets the `Cert-Serial` header with the value
    // from the client certificate. This is an extra safeguard against using this endpoint directly.
    if cert_serial.is_none_or(|s| s.into_inner().is_empty()) {
        return Err((StatusCode::FORBIDDEN, "client certificate missing"));
    }
    let response = next.run(request).await;
    Ok(response)
}

#[derive(Deserialize)]
struct PreloadPayload {
    bsn: String,
    repeat_bsn: String,
    authenticity_token: String,
}

#[derive(Deserialize, Debug)]
struct ClearPayload {
    confirmation_text: String,
    authenticity_token: String,
}

#[derive(Template, Default)]
#[template(path = "index.askama", escape = "html", ext = "html")]
struct IndexTemplate {
    authenticity_token: String,
    preloaded_count: u64,
}

#[derive(Template, Default)]
#[template(path = "result.askama", escape = "html", ext = "html")]
struct ResultTemplate {
    msg: String,
}

async fn index(State(state): State<Arc<ApplicationState>>, token: CsrfToken) -> Result<Response> {
    let path = &state.base_path.join(&state.preloaded_settings.xml_path);
    let preloaded_count = count_files_in_dir(path).await.map_err(|err| anyhow!(err))?;

    let result = IndexTemplate {
        authenticity_token: token.authenticity_token().map_err(|err| anyhow!(err))?,
        preloaded_count,
    };

    Ok(askama_axum::into_response_with_csrf(token, &result))
}

async fn fetch(
    State(state): State<Arc<ApplicationState>>,
    token: CsrfToken,
    ExtractCertSerial(cert_serial): ExtractCertSerial,
    Form(payload): Form<PreloadPayload>,
) -> Result<Response> {
    token.verify(&payload.authenticity_token).map_err(|err| anyhow!(err))?;

    if payload.bsn != payload.repeat_bsn {
        return Err(anyhow!("BSNs do not match"))?;
    }

    let bsn = Bsn::try_new(payload.bsn).map_err(|err| anyhow!(err))?;
    let path = &state.base_path.join(&state.preloaded_settings.xml_path);
    let serial = cert_serial.unwrap_or_default();

    info!("Preloading data using certificate having serial: {:?}", &serial);

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
        path,
        bsn.as_ref(),
    )
    .await
    .map_err(|err| anyhow!(err))?;

    let result = ResultTemplate {
        msg: String::from("Successfully preloaded"),
    };

    info!(
        "Successfully preloaded data using certificate having serial: {:?}",
        &serial
    );

    Ok(askama_axum::into_response(&result))
}

async fn clear(
    State(state): State<Arc<ApplicationState>>,
    token: CsrfToken,
    ExtractCertSerial(cert_serial): ExtractCertSerial,
    Form(payload): Form<ClearPayload>,
) -> Result<Response> {
    token.verify(&payload.authenticity_token).map_err(|err| anyhow!(err))?;

    if payload.confirmation_text != "clear all data" {
        return Err(anyhow!("Confirmation text is not correct"))?;
    }

    let serial = cert_serial.unwrap_or_default();

    info!(
        "Clearing all preloaded data using certificate having serial: {:?}",
        &serial
    );

    let path = &state.base_path.join(&state.preloaded_settings.xml_path);
    let count = clear_files_in_dir(path).await.map_err(|err| anyhow!(err))?;

    let result = ResultTemplate {
        msg: format!("Successfully cleared {count} items"),
    };

    info!("Cleared {count} items using certificate having serial: {:?}", &serial);

    Ok(askama_axum::into_response(&result))
}
