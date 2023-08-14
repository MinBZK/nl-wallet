use std::{fs, ops::Add, sync::Arc};

use anyhow::Result;
use axum::{
    body::Bytes,
    extract::{Path, State},
    headers::{authorization::Bearer, Authorization},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router, TypedHeader,
};
use chrono::{Days, Utc};
use ciborium::Value;
use futures::future::TryFutureExt;
use http::StatusCode;
use indexmap::IndexMap;
use josekit::jwe::alg::rsaes::RsaesJweDecrypter;
use tracing::{debug, info};

use nl_wallet_mdoc::{
    basic_sa_ext::{Entry, UnsignedMdoc},
    issuer::{self, MemorySessionStore, PrivateKey, SingleKeyRing},
    ServiceEngagement, Tdate,
};

use crate::{
    settings::Settings,
    userinfo_client::{self, Client, UserInfoJWT},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("OIDC client error: {0}")]
    Client(#[from] userinfo_client::Error),
    #[error("no BSN found in response from OIDC server")]
    NoBSN,
    #[error("starting mdoc session failed: {0}")]
    StartMdoc(#[source] nl_wallet_mdoc::Error),
    #[error("mdoc session error: {0}")]
    Mdoc(#[source] nl_wallet_mdoc::Error),
}

// TODO: Implement proper error handling.
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", self)).into_response()
    }
}

struct ApplicationState {
    openid_client: Client,
    jwe_decrypter: RsaesJweDecrypter,
    issuer: issuer::Server<SingleKeyRing, MemorySessionStore>,
    doctype: String,
}

pub async fn create_router(settings: Settings) -> Result<Router> {
    debug!("Discovering DigiD issuer...");

    let openid_client = Client::discover(settings.digid.issuer_url, settings.digid.client_id).await?;

    debug!("DigiD issuer discovered, starting HTTP server");

    let key = SingleKeyRing {
        doctype: settings.pid_doctype.clone(),
        issuance_key: PrivateKey::from_pem(
            &fs::read_to_string(settings.issuer_key.private_key)?,
            &fs::read_to_string(settings.issuer_key.certificate)?,
        )?,
    };
    let application_state = Arc::new(ApplicationState {
        openid_client,
        jwe_decrypter: Client::decrypter_from_jwk_file(settings.digid.bsn_privkey)?,
        issuer: issuer::Server::new(settings.public_url.to_string(), key, MemorySessionStore::new()),
        doctype: settings.pid_doctype.clone(),
    });

    let app = Router::new()
        .route("/mdoc/:session_token", post(mdoc_route))
        .route("/start", post(start_route))
        .with_state(application_state);

    Ok(app)
}

async fn mdoc_route(
    State(state): State<Arc<ApplicationState>>,
    Path(session_token): Path<String>,
    msg: Bytes,
) -> Result<Vec<u8>, Error> {
    let response = state
        .issuer
        .process_message(session_token.into(), &msg)
        .map_err(Error::StartMdoc)?;
    Ok(response)
}

async fn start_route(
    State(state): State<Arc<ApplicationState>>,
    TypedHeader(authorization_header): TypedHeader<Authorization<Bearer>>,
) -> Result<Json<ServiceEngagement>, Error> {
    // Using the access_token that the user specified, lookup the user's BSN at the OIDC issuer (DigiD bridge)
    let access_token = authorization_header.token();
    let bsn = request_bsn(&state.openid_client, &state.jwe_decrypter, access_token)
        .inspect_err(|error| info!("Error while extracting BSN: {}", error))
        .await?;

    // Start the session, and return the initial mdoc protocol message (containing the URL at which the wallet can
    // find us) to the wallet
    let attributes = pid_attributes(&state.doctype, &bsn);
    let service_engagement = state.issuer.new_session(attributes).map_err(Error::StartMdoc)?;

    Ok(Json(service_engagement))
}

async fn request_bsn(
    client: &Client,
    jwe_decrypter: &RsaesJweDecrypter,
    access_token: impl AsRef<str>,
) -> Result<String, Error> {
    let userinfo_claims: UserInfoJWT = client
        .request_userinfo_decrypted_claims(access_token, jwe_decrypter)
        .await?;

    Client::bsn_from_claims(&userinfo_claims)?.ok_or(Error::NoBSN)
}

fn pid_attributes(doctype: &str, bsn: &str) -> Vec<UnsignedMdoc> {
    vec![UnsignedMdoc {
        doc_type: doctype.to_string(),
        count: 1,
        valid_from: Tdate::now(),
        valid_until: Utc::now().add(Days::new(365)).into(),
        attributes: IndexMap::from([(
            doctype.to_string(),
            vec![Entry {
                name: "bsn".to_string(),
                value: Value::Text(bsn.to_string()),
            }],
        )]),
    }]
}
