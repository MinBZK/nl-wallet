use std::net::SocketAddr;
use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::Json, routing::post, Router};

use tower_http::trace::TraceLayer;

use base64::{engine::general_purpose::STANDARD, Engine};
use p256::{ecdsa::SigningKey, pkcs8::EncodePrivateKey};
use rand::rngs::OsRng;

use rust_core::utils::random_bytes;
use rust_core::wallet::signed::SignedDouble;
use rust_core::wp::instructions::Registration;
use rust_core::wp::{AccountServer, Certificate, Challenge};

struct AppState {
    account_server: AccountServer,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let account_server = new_account_server();
    dbg!(STANDARD.encode(&account_server.pubkey));

    let shared_state = Arc::new(AppState { account_server });

    let api_routes = Router::new()
        .route("/enroll", post(enroll))
        .route("/createwallet", post(create_wallet))
        .layer(TraceLayer::new_for_http())
        .with_state(shared_state);

    let app = Router::new().nest("/api/v1", api_routes);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn new_account_server() -> AccountServer {
    let as_privkey = SigningKey::random(&mut OsRng);
    AccountServer::new(
        as_privkey.to_pkcs8_der().unwrap().as_bytes().to_vec(),
        random_bytes(32),
        "test_account_server".to_owned(),
    )
    .unwrap()
}

async fn enroll(State(state): State<Arc<AppState>>) -> (StatusCode, Json<Challenge>) {
    let challenge = state.account_server.registration_challenge().unwrap();
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
    let cert = state.account_server.register(payload).unwrap();
    (StatusCode::CREATED, Json(Certificate { certificate: cert }))
}
