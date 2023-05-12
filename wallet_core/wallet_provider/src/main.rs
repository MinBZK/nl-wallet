use std::net::SocketAddr;
use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::Json, routing::post, Router};
use base64::{engine::general_purpose::STANDARD, Engine};
use p256::{ecdsa::SigningKey, pkcs8::DecodePrivateKey};
use tower_http::trace::TraceLayer;

use wallet_common::{
    account::{
        auth::{Certificate, Challenge, Registration},
        signed::SignedDouble,
    },
    utils::random_bytes,
};

use crate::account_server::AccountServer;

mod account_server;
mod settings;

struct AppState {
    account_server: AccountServer,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let settings = settings::Settings::new().unwrap();

    let account_server_privkey = settings.signing_private_key;

    let account_server = AccountServer::new(
        account_server_privkey.0.clone(),
        random_bytes(32),
        "stub_account_server".into(),
    )
    .unwrap();

    dbg!(STANDARD.encode(
        SigningKey::from_pkcs8_der(&account_server_privkey.0)
            .unwrap()
            .verifying_key()
            .to_encoded_point(false)
            .as_bytes()
    ));

    let app = app(account_server);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
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
