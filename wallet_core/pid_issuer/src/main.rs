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
use openid::Client;
use reqwest::{StatusCode, Url};
use serde::Serialize;
use serde_json::json;
use tracing::info;

use pid_issuer::userinfo::{
    bsn_from_claims, decrypter_from_jwk_file, ClientUserInfoExtension, UserInfoError, UserInfoJWT,
};

// TODO: read from configuration
const DIGID_ISSUER_URL: &str = "https://example.com/digid-connector";

// TODO: read the following values from configuration, and align with digid-connector configuration
const WALLET_CLIENT_ID: &str = "SSSS";
const WALLET_CLIENT_REDIRECT_URI: &str = "walletdebuginteraction://wallet.edi.rijksoverheid.nl/authentication";

const SECRETS_DIR: &str = "secrets";
const JWK_PRIVATE_KEY_FILE: &str = "private_key.jwk";

async fn create_openid_client() -> anyhow::Result<Client> {
    let issuer_url = Url::parse(DIGID_ISSUER_URL)?;

    let openid_client = Client::discover_with_client(
        reqwest::Client::new(),
        WALLET_CLIENT_ID.to_string(),
        None,
        Some(WALLET_CLIENT_REDIRECT_URI.to_string()),
        issuer_url,
    )
    .await?;

    Ok(openid_client)
}

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
        let bsn = bsn_from_claims(&userinfo_claims)?.ok_or(PidIssuerError::NoBSN)?;

        Ok(bsn)
    }
}

async fn serve() -> anyhow::Result<()> {
    let secrets_path = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_default()
        .join(SECRETS_DIR);

    let application_state = Arc::new(ApplicationState {
        openid_client: create_openid_client().await?,
        jwe_decrypter: decrypter_from_jwk_file(secrets_path.join(JWK_PRIVATE_KEY_FILE))?,
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
    OidcClient(#[from] UserInfoError),
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
