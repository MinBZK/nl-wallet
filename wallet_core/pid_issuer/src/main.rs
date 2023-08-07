use std::{env, net::SocketAddr, path::PathBuf, sync::Arc};

use axum::{
    extract::State,
    headers::{authorization::Bearer, Authorization},
    response::IntoResponse,
    routing::post,
    Json, Router, TypedHeader,
};
use futures::future::TryFutureExt;
use josekit::jwe::alg::rsaes::RsaesJweDecrypter;
use reqwest::{StatusCode, Url};
use serde::Serialize;
use serde_json::json;
use tracing::info;

use pid_issuer::userinfo_client::{self, Client, UserInfoJWT};

// TODO: read from configuration
const DIGID_ISSUER_URL: &str = "https://example.com/digid-connector";

// TODO: read from configuration
// TODO: Use separate client ID for mock PID issuer.
const WALLET_CLIENT_ID: &str = "SSSS";

const SECRETS_DIR: &str = "secrets";
const JWK_PRIVATE_KEY_FILE: &str = "private_key.jwk";

struct ApplicationState {
    openid_client: Client,
    jwe_decrypter: RsaesJweDecrypter,
}

impl ApplicationState {
    async fn extract_bsn(&self, access_token: &str) -> Result<String, PidIssuerError> {
        let userinfo_claims: UserInfoJWT = self
            .openid_client
            .request_userinfo_decrypted_claims(access_token, &self.jwe_decrypter)
            .await?;
        let bsn = Client::bsn_from_claims(&userinfo_claims)?.ok_or(PidIssuerError::NoBSN)?;

        Ok(bsn)
    }
}

async fn serve() -> anyhow::Result<()> {
    let issuer_url = Url::parse(DIGID_ISSUER_URL).expect("Could not parse DIGID_ISSUER_URL");
    let secrets_path = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_default()
        .join(SECRETS_DIR);

    let application_state = Arc::new(ApplicationState {
        openid_client: Client::discover(issuer_url, WALLET_CLIENT_ID).await?,
        jwe_decrypter: Client::decrypter_from_jwk_file(secrets_path.join(JWK_PRIVATE_KEY_FILE))?,
    });

    let app = Router::new()
        .route("/extract_bsn", post(extract_bsn))
        .with_state(application_state);

    // TODO make ip and port configurable
    let addr = SocketAddr::from(([127, 0, 0, 1], 3003));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr).serve(app.into_make_service()).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize tracing
    tracing_subscriber::fmt::init();

    serve().await
}

#[derive(Debug, thiserror::Error)]
enum PidIssuerError {
    #[error("OIDC client error: {0}")]
    OidcClient(#[from] userinfo_client::Error),
    #[error("no BSN found in response from OIDC server")]
    NoBSN,
}

impl IntoResponse for PidIssuerError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::OidcClient(_) => {
                let body = Json(json!({
                    "error": "oidc_client_error",
                    "error_description": format!("{}", self),
                }));
                (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
            }
            Self::NoBSN => {
                let body = Json(json!({
                    "error": "no_bsn_error",
                    "error_description": format!("{}", self),
                }));
                (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
            }
        }
    }
}

async fn extract_bsn(
    State(state): State<Arc<ApplicationState>>,
    TypedHeader(authorization_header): TypedHeader<Authorization<Bearer>>,
) -> Result<Json<BsnResponse>, PidIssuerError> {
    let access_token = authorization_header.token();
    let bsn = state
        .extract_bsn(access_token)
        .inspect_err(|error| info!("Error while extracting BSN: {}", error))
        .await?;

    let response = BsnResponse { bsn };
    Ok(Json(response))
}

#[derive(Serialize)]
struct BsnResponse {
    bsn: String,
}
