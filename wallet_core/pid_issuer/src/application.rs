use std::{fs, ops::Add, sync::Arc};

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
use p256::{ecdsa::SigningKey, pkcs8::DecodePrivateKey};
use serde::Serialize;
use tracing::{debug, info};

use nl_wallet_mdoc::{
    basic_sa_ext::{Entry, UnsignedMdoc},
    issuer::{self, MemorySessionStore, PrivateKey, SingleKeyRing},
    utils::x509::Certificate,
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
}

const PID_DOCTYPE: &str = "nl.voorbeeldwallet.test.pid";

pub async fn create_router(settings: Settings) -> Result<Router, userinfo_client::Error> {
    debug!("Discovering DigiD issuer...");

    let openid_client = Client::discover(settings.digid.issuer_url, settings.digid.client_id).await?;

    debug!("DigiD issuer discovered, starting HTTP server");

    let key = SingleKeyRing {
        doctype: PID_DOCTYPE.to_string(),
        issuance_key: PrivateKey::new(
            SigningKey::from_pkcs8_pem(&fs::read_to_string(settings.issuer_key.private_key)?).unwrap(),
            Certificate::from_pem(&fs::read_to_string(settings.issuer_key.certificate)?).unwrap(),
        ),
    };
    let application_state = Arc::new(ApplicationState {
        openid_client,
        jwe_decrypter: Client::decrypter_from_jwk_file(settings.digid.bsn_privkey)?,
        issuer: issuer::Server::new(settings.public_url.to_string(), key, MemorySessionStore::new()),
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
        .expect("processing mdoc message failed"); // TODO
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
    let attributes = pid_attributes(bsn);
    let service_engagement = state.issuer.new_session(attributes).expect("TODO");

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

fn pid_attributes(bsn: String) -> Vec<UnsignedMdoc> {
    vec![UnsignedMdoc {
        doc_type: PID_DOCTYPE.to_string(),
        count: 1,
        valid_from: Tdate::now(),
        valid_until: Utc::now().add(Days::new(365)).into(),
        attributes: IndexMap::from([(
            PID_DOCTYPE.to_string(),
            vec![Entry {
                name: "bsn".to_string(),
                value: Value::Text(bsn),
            }],
        )]),
    }]
}
