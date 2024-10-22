use std::{default::Default, env, path::PathBuf, result::Result as StdResult, sync::Arc};

use aes_gcm::Aes256Gcm;
use anyhow::anyhow;
use askama::Template;
use axum::{
    extract::{Request, State},
    middleware,
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{get, post},
    Form, Router,
};
use axum_csrf::{CsrfConfig, CsrfLayer, CsrfToken};
use http::{HeaderMap, StatusCode};
use serde::Deserialize;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{debug, level_filters::LevelFilter, warn};
use tracing_subscriber::EnvFilter;

use gba_hc_converter::{
    fetch::askama_axum,
    gba::{
        client::{GbavClient, HttpGbavClient},
        encryption::{clear_files_in_dir, count_files_in_dir, encrypt_bytes_to_dir, HmacSha256},
    },
    haal_centraal::Bsn,
    settings::{PreloadedSettings, RunMode, Settings},
};

// Cannot use #[tokio::main], see: https://docs.sentry.io/platforms/rust/#async-main-function
fn main() -> anyhow::Result<()> {
    let settings = Settings::new()?;

    // Retain [`ClientInitGuard`]
    let _guard = settings
        .sentry
        .as_ref()
        .map(|sentry| sentry.init(sentry::release_name!()));

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

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async { serve(settings).await })
}

#[derive(Debug)]
pub struct Error(anyhow::Error);

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Self(error)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        warn!("error result: {:?}", self);
        let result = ResultTemplate {
            msg: self.0.to_string(),
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
                .route("/clear", post(clear))
                .with_state(app_state)
                .layer(CsrfLayer::new(csrf_config))
                .layer(middleware::from_fn(check_auth))
                .layer(TraceLayer::new_for_http()),
        );

    axum::serve(listener, app).await?;

    Ok(())
}

async fn check_auth(headers: HeaderMap, request: Request, next: Next) -> StdResult<Response, StatusCode> {
    if !headers.get("Cert-Serial").is_some_and(|s| !s.is_empty()) {
        return Err(StatusCode::FORBIDDEN);
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
    Form(payload): Form<PreloadPayload>,
) -> Result<Response> {
    token.verify(&payload.authenticity_token).map_err(|err| anyhow!(err))?;

    if payload.bsn != payload.repeat_bsn {
        return Err(anyhow!("BSN's do not match"))?;
    }

    let bsn = Bsn::try_new(payload.bsn).map_err(|err| anyhow!(err))?;
    let path = &state.base_path.join(&state.preloaded_settings.xml_path);

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

    Ok(askama_axum::into_response(&result))
}

async fn clear(
    State(state): State<Arc<ApplicationState>>,
    token: CsrfToken,
    Form(payload): Form<ClearPayload>,
) -> Result<Response> {
    token.verify(&payload.authenticity_token).map_err(|err| anyhow!(err))?;

    if payload.confirmation_text != "clear all data" {
        return Err(anyhow!("Confirmation text is not correct"))?;
    }

    let path = &state.base_path.join(&state.preloaded_settings.xml_path);
    let count = clear_files_in_dir(path).await.map_err(|err| anyhow!(err))?;

    let result = ResultTemplate {
        msg: format!("Successfully cleared {count} items"),
    };

    Ok(askama_axum::into_response(&result))
}
