use std::net::SocketAddr;
use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::Json, routing::post, Router};
use base64::{engine::general_purpose::STANDARD, Engine};
use tower_http::trace::TraceLayer;

use wallet_provider::account_server::AccountServer;
use wallet_shared::account::{instructions::Registration, signed::SignedDouble, Certificate, Challenge};

struct AppState {
    account_server: AccountServer,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let account_server = AccountServer::new_stub();
    dbg!(STANDARD.encode(&account_server.pubkey));

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
