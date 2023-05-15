use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::Json, routing::post, Router};
use tower_http::trace::TraceLayer;

use wallet_common::account::{
    auth::{Certificate, Challenge, Registration},
    signed::SignedDouble,
};

use crate::account_server::AccountServer;

struct AppState {
    account_server: AccountServer,
}

pub fn router(account_server: AccountServer) -> Router {
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
