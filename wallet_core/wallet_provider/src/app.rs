use axum::{
    routing::post,
    {extract::State, http::StatusCode, response::Json, Router},
};
use std::sync::Arc;
use tower_http::trace::TraceLayer;

use wallet_common::account::{
    auth::{Certificate, Challenge, Registration},
    signed::SignedDouble,
};

use crate::app_dependencies::AppDependencies;

pub fn router(dependencies: Arc<AppDependencies>) -> Router {
    Router::new().nest(
        "/api/v1",
        Router::new()
            .route("/enroll", post(enroll))
            .route("/createwallet", post(create_wallet))
            .layer(TraceLayer::new_for_http())
            .with_state(dependencies),
    )
}

async fn enroll(State(state): State<Arc<AppDependencies>>) -> (StatusCode, Json<Challenge>) {
    let challenge = state.account_server.registration_challenge().unwrap(); // todo: error handling
    (
        StatusCode::OK,
        Json(Challenge {
            challenge: challenge.into(),
        }),
    )
}

async fn create_wallet(
    State(state): State<Arc<AppDependencies>>,
    Json(payload): Json<SignedDouble<Registration>>,
) -> (StatusCode, Json<Certificate>) {
    let cert = state.account_server.register(state.as_ref(), payload).await.unwrap(); // todo: error handling
    (StatusCode::CREATED, Json(Certificate { certificate: cert }))
}
