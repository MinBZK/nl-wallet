use std::{
    env,
    fs::File,
    io::BufReader,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};

use axum::{
    extract::State,
    headers::{authorization::Bearer, Authorization},
    response::IntoResponse,
    routing::post,
    Json, Router, TypedHeader,
};
use josekit::jwk::Jwk;
use openid::{biscuit::ClaimsSet, error::ClientError, Client, Empty, Jws};
use reqwest::{StatusCode, Url};
use serde::Serialize;
use serde_json::json;

use pid_issuer::userinfo::{AttributeMap, UserinfoExtensions};

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

fn read_jwk_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Jwk> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `User`.
    let jwk = serde_json::from_reader(reader)?;

    Ok(jwk)
}

struct ApplicationState {
    openid_client: Client,
    private_key: Jwk,
}

impl ApplicationState {
    async fn extract_bsn(&self, access_token: &str) -> Result<String, PidIssuerError> {
        let userinfo = self.openid_client.invoke_userinfo_endpoint(access_token).await?;
        let userinfo_claims: Jws<ClaimsSet<AttributeMap>, Empty> = self
            .openid_client
            .jwe_decrypt_claims(userinfo.as_str().unwrap(), &self.private_key)?;
        let bsn = self.openid_client.extract_bsn(&userinfo_claims)?.unwrap();
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
        private_key: read_jwk_from_file(secrets_path.join(JWK_PRIVATE_KEY_FILE))?,
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

enum PidIssuerError {
    OidcClientError(ClientError),
    Anyhow(anyhow::Error),
}

impl From<ClientError> for PidIssuerError {
    fn from(source: ClientError) -> Self {
        Self::OidcClientError(source)
    }
}

impl From<anyhow::Error> for PidIssuerError {
    fn from(source: anyhow::Error) -> Self {
        Self::Anyhow(source)
    }
}

impl IntoResponse for PidIssuerError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::OidcClientError(e) => {
                let body = Json(json!({
                    "error": "oidc_client_error",
                    "error_description": format!("{}", e),
                }));
                (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
            }
            Self::Anyhow(e) => {
                let body = Json(json!({
                    "error": "generic",
                    "error_description": format!("{}", e),
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
    let bsn = state.extract_bsn(access_token).await?;

    let response = BsnResponse { bsn };
    Ok(Json(response))
}

#[derive(Serialize)]
struct BsnResponse {
    bsn: String,
}
