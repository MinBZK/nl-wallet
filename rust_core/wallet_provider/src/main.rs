// Prevent dead code warnings because AccountServer::new_stub is not used in the bin.
// TODO: remove this when the wallet_provider is not a dependency of the wallet.
#![allow(dead_code)]

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::Json, routing::post, Router};
use base64::{engine::general_purpose::STANDARD, Engine};
use p256::{
    ecdsa::SigningKey,
    pkcs8::{EncodePrivateKey, EncodePublicKey},
};
use rand::rngs::OsRng;
use tower_http::trace::TraceLayer;

use wallet_shared::{
    account::{instructions::Registration, signed::SignedDouble, Certificate, Challenge},
    utils::random_bytes,
};

use crate::account_server::AccountServer;

mod account_server;

struct AppState {
    account_server: AccountServer,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let account_server_privkey = SigningKey::random(&mut OsRng);
    let account_server = AccountServer::new(
        account_server_privkey.to_pkcs8_der().unwrap().as_bytes().to_vec(),
        random_bytes(32),
        "stub_account_server".into(),
    )
    .unwrap();

    dbg!(STANDARD.encode(
        account_server_privkey
            .verifying_key()
            .to_public_key_der()
            .unwrap()
            .as_bytes()
    ));

    let app = app(account_server);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
}

fn app(account_server: AccountServer) -> Router {
    let shared_state = Arc::new(AppState { account_server });

    Router::new().nest(
        "/api/v1",
        Router::new()
            .route("/enroll", post(enroll))
            .route("/createwallet", post(create_wallet))
            .layer(TraceLayer::new_for_http())
            .with_state(shared_state),
    )
}

async fn enroll(State(state): State<Arc<AppState>>) -> (StatusCode, Json<Challenge>) {
    let challenge = state.account_server.registration_challenge().unwrap(); // todo: error handling
    (
        StatusCode::OK,
        Json(Challenge {
            challenge: challenge.into(),
        }),
    )
}

async fn create_wallet(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SignedDouble<Registration>>,
) -> (StatusCode, Json<Certificate>) {
    let cert = state.account_server.register(payload).unwrap(); // todo: error handling
    (StatusCode::CREATED, Json(Certificate { certificate: cert }))
}
