use std::{path::Path, sync::Arc};

use axum::{
    extract::State,
    headers::{authorization::Bearer, Authorization},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router, TypedHeader,
};
use futures::future::TryFutureExt;
use http::StatusCode;
use josekit::jwe::alg::rsaes::RsaesJweDecrypter;
use serde::Serialize;
use tracing::info;
use url::Url;

use crate::userinfo_client::{self, Client, UserInfoJWT};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("OIDC client error: {0}")]
    Client(#[from] userinfo_client::Error),
    #[error("no BSN found in response from OIDC server")]
    NoBSN,
}

// TODO: Implement proper error handling.
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", self)).into_response()
    }
}

#[derive(Serialize)]
pub struct BsnResponse {
    bsn: String,
}

struct ApplicationState {
    openid_client: Client,
    jwe_decrypter: RsaesJweDecrypter,
}

pub async fn create_router(
    issuer_url: Url,
    client_id: impl Into<String>,
    private_key_path: impl AsRef<Path>,
) -> Result<Router, userinfo_client::Error> {
    let application_state = Arc::new(ApplicationState {
        openid_client: Client::discover(issuer_url, client_id).await?,
        jwe_decrypter: Client::decrypter_from_jwk_file(private_key_path)?,
    });

    let x = post(extract_bsn_route);
    let app = Router::new().route("/extract_bsn", x).with_state(application_state);

    Ok(app)
}

async fn extract_bsn_route(
    State(state): State<Arc<ApplicationState>>,
    TypedHeader(authorization_header): TypedHeader<Authorization<Bearer>>,
) -> Result<Json<BsnResponse>, Error> {
    let access_token = authorization_header.token();

    let bsn = extract_bsn(&state.openid_client, &state.jwe_decrypter, access_token)
        .inspect_err(|error| info!("Error while extracting BSN: {}", error))
        .await?;

    let response = BsnResponse { bsn };

    Ok(Json(response))
}

async fn extract_bsn(
    client: &Client,
    jwe_decrypter: &RsaesJweDecrypter,
    access_token: impl AsRef<str>,
) -> Result<String, Error> {
    let userinfo_claims: UserInfoJWT = client
        .request_userinfo_decrypted_claims(access_token, jwe_decrypter)
        .await?;

    Client::bsn_from_claims(&userinfo_claims)?.ok_or(Error::NoBSN)
}
